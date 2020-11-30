//! Structs, data types, and functions for purtel: Phips userland runtime task execution library.

mod types;

use crate::PurtelTaskState::{WAITING, DISPATCHED};
use std::thread;
use std::sync::mpsc::channel;
use crate::PurtelParamUsageKind::{READ, WRITE};
use crate::types::{TaskId, TaskDependencies, TaskExecutionLevel};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PurtelTaskState {
    WAITING,
    DISPATCHED,
}

pub struct PurtelTask {
    // This is an option because this memory
    // is taken from the purtel task to prepare
    // execution
    closure: Option<Box<dyn FnOnce() -> () + Send>>,
    state: PurtelTaskState,
}

impl PurtelTask {

    pub fn new(closure: Box<dyn FnOnce() -> () + Send>) -> Self {
        Self {
            closure: Some(closure),
            state: WAITING,
        }
    }

    pub fn take_task(&mut self) -> Box<dyn FnOnce() -> () + Send> {
        if self.state != WAITING { panic!("Task is not in WAITING state!") }
        self.state = DISPATCHED;
        self.closure.take().expect("Must have value")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PurtelParamUsageKind {
    READ,
    WRITE,
}

#[derive(Debug)]
pub struct PurtelParamUsage {
    identifier: String,
    kind: PurtelParamUsageKind,
}

impl PurtelParamUsage {
    pub fn new(identifier: &str, kind: PurtelParamUsageKind) -> Self {
        Self {
            identifier: identifier.to_owned(),
            kind
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn kind(&self) -> PurtelParamUsageKind {
        self.kind
    }
}

/// Struct that contains all tasks shat shall be executed by Purtel. It needs meta-data
/// about the relation of the dependencies.
pub struct PurtelExecutor {
    param_usage_desc: Option<Vec<Vec<PurtelParamUsage>>>,
    exe_order: Option<Vec<TaskExecutionLevel>>,
    tasks: Vec<PurtelTask>,
}

impl PurtelExecutor {

    /// Constructor. Takes the closures/actual tasks and a description
    /// how each parameter per task is used. With this data `PurtelExecutor`
    /// can calculate an optimized execution order.
    pub fn new(tasks: Vec<PurtelTask>,
               param_usage_desc: Vec<Vec<PurtelParamUsage>>) -> Self {

        // Validate dependencies
        assert_eq!(tasks.len(), param_usage_desc.len(), "You must specify param usage for every task!");

        eprintln!("got the following parameter usage description");
        dbg!(&param_usage_desc);

        Self {
            exe_order: None,
            param_usage_desc: Some(param_usage_desc),
            tasks,
        }
    }

    /// Helper function for `calc_task_dependencies()`. Asserts there are no duplicates
    /// (to prevent human error for example). This means every task can list a parameter only
    /// once. This also ensures that if a parameter is declared as write it is not also
    /// declared as read.
    fn assert_no_duplicates(param_usages: &Vec<Vec<PurtelParamUsage>>) {
        for (task_i, param_usage) in param_usages.iter().enumerate() {
            // We check that each parameter ID is contained only once
            for (p_i, p_desc) in param_usage.iter().enumerate() {
                let p_id = &p_desc.identifier;
                for p_j in (p_i + 1)..param_usage.len() {
                    let p_j_id = &param_usage[p_j].identifier;
                    if p_id == p_j_id {
                        panic!("Task {} declares usage for parameter '{}' multiple times, that's illegal!", task_i, p_id);
                    }
                }
            }
        }
    }

    /// Calculates the dependencies for each task id/index on other task ids/indices. This is done
    /// by an analysis of the parameter usage per task index. A dependency to a previous task exists
    /// iff:
    ///   - a task with a lower id has write access to the same parameter (Read after Write), or
    ///   - a task with a lower id has read access to a parameter that this tasks
    ///     needs right access for (Write after Read, Write After Write)
    /// The resulting vector is a vector per task (index) that contains all task indices that must
    /// be finished before the task can run.
    ///
    /// We don't do a simplification for the transitivity of dependencies
    /// (i.e.: A <- B, B <- C => Deps(C) = {A, B}) to reduce the complexity of the algorithm.
    /// The overhead is (probably even for thousands of tasks?) negligible.
    ///
    /// Tasks with the same count of dependencies can never be dependent on each other.
    fn calc_task_dependencies(param_usages: &Vec<Vec<PurtelParamUsage>>) -> Vec<TaskDependencies> {
        // checks if parameter usage is properly defined
        PurtelExecutor::assert_no_duplicates(&param_usages);

        let mut all_dependencies = vec![];
        // for each tasks
        for task_i in 0..param_usages.len() {
            // dependencies of the current task
            // these are the indices of all tasks for the current
            // task that current dask is dependent of
            let mut task_dependencies = vec![];

            // for each param per task
            for task_i_param_i in 0..param_usages[task_i].len() {
                // we check if a dependence to a prevous task exists
                let param = &param_usages[task_i][task_i_param_i];

                // check all params that previous tasks use
                for prev_task_i in 0..task_i {
                    // for each param of previous tasks
                    for prev_task_i_param_i in 0..param_usages[prev_task_i].len() {
                        let prev_param = &param_usages[prev_task_i][prev_task_i_param_i];
                        // true if: a previous tasks uses the same parameter
                        let already_in_deps = task_dependencies.contains(&prev_task_i);
                        if param.identifier == prev_param.identifier && !already_in_deps {
                            // Dependency exists iff:
                            // - prev usage is write
                            // - current usage is write and prev usage is read
                            let write_after_write = param.kind() == WRITE && prev_param.kind() == WRITE;
                            let write_after_read = param.kind() == WRITE && prev_param.kind() == READ;
                            // let read_after_read = param.kind() == READ && prev_param.kind() == READ;
                            let read_after_write = param.kind() == READ && prev_param.kind() == WRITE;

                            if write_after_read || write_after_write || read_after_write {
                                // task "task_i" has dependency to task "prev_task_i"
                                task_dependencies.push(prev_task_i);
                            }
                        }
                    }
                }
            }
            all_dependencies.push(task_dependencies)
        }
        all_dependencies
    }

    /// Execution order is a Vector of Vector of task indices. The
    /// primary vector describes in how many iterations several tasks
    /// are bundles and executed in parallel ("execution level").
    /// All threads/tasks of one iterations step must be finished before
    /// the next iteration can start. Different iteration-levels describe
    /// dependent tasks to previous iterations while the indices in the
    /// inner vector are independent from each other. Tasks in the same
    /// execution level can never be dependent on each other. But they can
    /// have concurrent read to the same data.
    ///  * `task_deps: Vec<Vec<usize>>`: Vector with all dependencies per
    ///                                  task id. A dependency is a task id
    ///                                  that can only be less than the current task_id
    fn calc_execution_levels(task_deps: Vec<TaskDependencies>) -> Vec<TaskExecutionLevel> {
        let mut execution_levels: Vec<Vec<usize>> = vec![];

        // Vector that maps from index (task id) to Option. The option describes whether
        // this tasks was already assigned to an execution level.
        let mut tasks_assigned_map = task_deps.iter()
            .map(|_| Some(()))
            .collect::<Vec<Option<()>>>();

        // Convenient lambda that checks if all tasks are assigned yet.
        let all_tasks_assigned = |tasks_assigned_map: &Vec<Option<()>>| {
            tasks_assigned_map.iter()
                .filter(|x| x.is_some())
                .count() == 0
        };

        // Convenient lambda that returns a vector with all unassigned task ids.
        let get_unassigned_task_ids = |tasks_assigned_map: &Vec<Option<()>>| {
            tasks_assigned_map.iter()
                // do enumerate first to get the proper index!!
                .enumerate()
                .filter(|(_, assigned_opt)| assigned_opt.is_some())
                .map(|(task_id, _)| task_id)
                .collect::<Vec<TaskId>>()
        };

        while !all_tasks_assigned(&tasks_assigned_map) {
            debug_assert!(execution_levels.len() <= task_deps.len(), "Their can't be more execution levels that task dependencies!");

            // holds task IDs that can be executed on current level
            let mut curr_exe_level = vec![];

            // try for each task if we can assign it to the current level
            for task_id in get_unassigned_task_ids(&tasks_assigned_map) {
                let task_can_be_assigned = PurtelExecutor::all_deps_already_assigned(
                    task_id,
                    &execution_levels,
                    &task_deps,
                );
                if task_can_be_assigned {
                    tasks_assigned_map[task_id] = None;
                    curr_exe_level.push(task_id);
                }
            }


            if curr_exe_level.is_empty() {
                panic!("No tasks in current level! Deadlock or algorithm error?");
            }
            execution_levels.push(curr_exe_level);
        }

        execution_levels
    }

    /// Helper function for `calculate_exe_order` that checks if all tasks that the specified
    /// task is dependent from are already assigned to previous execution levels.
    fn all_deps_already_assigned(id: TaskId,
                                 execution_levels: &Vec<TaskExecutionLevel>,
                                 task_deps: &Vec<TaskDependencies>) -> bool {
        // ids of all tasks that this task is dependent from
        let task_deps = &task_deps[id];

        // not necessary because in this case the loop won't be active
        // and the last "true"-expression just returns true :)
        /*if task_deps.len() == 0 {
            return true;
        }*/

        // we check for each dependency (task id) if we can find
        // it in a previous execution level
        for dep_task_id in task_deps {
            let mut found = false;
            // at this point execution_levels only contains "complete" levels,
            // e.g. verified data; working set is not part of the vector yet; because of
            // this "-1" is not necessary at upper bound
            for tasks_of_level_i in 0..execution_levels.len() {
                let tasks_of_level = &execution_levels[tasks_of_level_i];
                if tasks_of_level.contains(dep_task_id) {
                    found = true;
                    break;
                }
            }

            // we couldn't find a dependency in a previous iteration level yet
            if !found {
                return false;
            }
        }

        true
    }

    /*/// Help function for `calculate_exe_order()`. Tells whether all tasks are assigned
    /// already to a level. If true, `calculate_exe_order()` can stop its work and return.
    fn assigned_all_tasks_to_level(execution_levels: &Vec<Vec<usize>>, num_tasks: usize) -> bool {
        let all_task_ids = execution_levels.iter()
            .flat_map(|vec| vec.iter())
            .map(|x| *x) // &usize to usize
            .collect::<Vec<usize>>();
        for i in 0..num_tasks {
            if !all_task_ids.contains(&i) {
                return false;
            }
        }
        true
    }*/

    /// Calculates an optimized order in which the tasks shall be executed.
    pub fn calc_and_verify_exe_order(&mut self) {
        // here we calculate which task id is dependent on what task ids
        let deps = PurtelExecutor::calc_task_dependencies(
            // take() to free memory; memory for footprint of thousands of tasks
            // may be big otherwise; also we do not need this any more
            &self.param_usage_desc.take().expect("calc_and_verify_exe_order() should only be called once!")
        );

        dbg!("found following dependencies");
        dbg!(&deps);

        // calculate an optimized execution order
        let exe_order = PurtelExecutor::calc_execution_levels(deps);
        self.exe_order = Some(exe_order);

        // this should only fail if my algorithm does weird things
        // check if not more levels than tasks exists
        debug_assert_eq!(0, self.exe_order.iter().filter(|vec| vec.is_empty()).count(), "Empty execution levels are invalid!");
    }

    /// Executes the tasks in an optimal order in a parallelized way.
    /// You *must* call `calc_and_verify_exe_order()` first.
    pub fn execute(mut self) {
        assert!(self.exe_order.is_some(), "Call calc_and_verify_exe_order() first!");

        dbg!("execute all tasks in the following order");
        dbg!(self.exe_order.as_ref().unwrap());

        for task_ids in self.exe_order.unwrap() {
            let mut handles = vec![];
            for task_id in &task_ids {
                // Channel needed to safely transfer heap data (Box<>) into a thread
                let (sender, receiver) = channel();
                let task = self.tasks[*task_id].take_task();
                sender.send(task).expect("Transfer of closure into thread must work");
                let h = thread::spawn(move || {
                    eprintln!("thread spawned!");
                    let closure = receiver.recv().expect("Must receive closure/task!");
                    closure();
                    eprintln!("thread stopped!");
                });
                handles.push(h);
            }

            // synchronously wait for all threads of current iteration level to finish and succeed!
            handles.into_iter().for_each(|h| {
                h.join().expect("Thread must succeed!");
                // doesn't work, rust compiler complains move errors...
                // self.tasks[task_i].finish();
            });
        }
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::PurtelParamUsageKind::{READ, WRITE};

    #[test]
    pub fn test_calc_dependencies_simple() {
        let param_usages = vec![
            // for first task
            vec![PurtelParamUsage::new("data1", READ)],
            // for second task
            vec![PurtelParamUsage::new("data1", WRITE),
                 PurtelParamUsage::new("data2", WRITE)],
            // for third task
            vec![PurtelParamUsage::new("data1", READ)],
            // for fourth task
            vec![PurtelParamUsage::new("data2", READ)],
        ];

        let dependencies = PurtelExecutor::calc_task_dependencies(&param_usages);
        assert_eq!(4, dependencies.len(), "Must generate dependencies for each task!");

        // first task has no dependencies;
        // PS: don't get confused, task indices start at 0; second task has index 1
        assert_eq!(dependencies[0], vec![], "first task has no dependencies");
        // second task is dependent to first task
        assert_eq!(dependencies[1], vec![0], "second task is dependent on first; Write After Read");
        // third and fourth task are dependent to second task
        assert_eq!(dependencies[2], vec![1], "third task is dependent on second; Read after Write");
        assert_eq!(dependencies[3], vec![1], "fourth task is dependent on second; Read after Write");
    }

    #[test]
    pub fn test_calc_dependencies_complex() {
        let param_usages = vec![
            // param usage of first task
            vec![PurtelParamUsage::new("data1", WRITE),
                 PurtelParamUsage::new("data2", WRITE)],
            // param usage of second task
            vec![PurtelParamUsage::new("data1", WRITE),
                 PurtelParamUsage::new("data2", WRITE)],
            // param usage of third task
            vec![PurtelParamUsage::new("data1", WRITE),
                 PurtelParamUsage::new("data2", WRITE)],
            // param usage of fourth task
            vec![PurtelParamUsage::new("data1", WRITE),
                 PurtelParamUsage::new("data2", WRITE)],
        ];

        let dependencies = PurtelExecutor::calc_task_dependencies(&param_usages);
        assert_eq!(4, dependencies.len(), "Must generate dependencies for each task!");

        // first task has no dependencies;
        // PS: don't get confused, task indices start at 0; second task has index 1
        assert_eq!(dependencies[0], vec![], "first task has no dependencies");
        assert_eq!(dependencies[1], vec![0], "second task is dependent on first; Write After Write");
        assert_eq!(dependencies[2], vec![0, 1], "third task is dependent on second, and first; Write after Write");
        assert_eq!(dependencies[3], vec![0, 1, 2], "fourth task is dependent on third, second, and first; Write after Write");
    }

    #[test]
    pub fn test_calc_execution_levels() {
        let param_usages = vec![
            // for first task
            vec![PurtelParamUsage::new("data1", READ)],
            // for second task
            vec![PurtelParamUsage::new("data1", WRITE),
                 PurtelParamUsage::new("data2", WRITE)],
            // for third task
            vec![PurtelParamUsage::new("data1", READ)],
            // for fourth task
            vec![PurtelParamUsage::new("data2", READ)],
        ];

        let deps = PurtelExecutor::calc_task_dependencies(&param_usages);
        let order = PurtelExecutor::calc_execution_levels(deps);
        assert_eq!(3, order.len(), "should only need 3 execution levels");

        // in first iteration only task 1 can run
        assert_eq!(vec![0], order[0], "in first iteration only first task can run");
        // in second iteration only task 2 can run
        assert_eq!(vec![1], order[1], "in second iteration only second task can run");
        // in third iteration only task 3 can run
        assert_eq!(vec![2, 3], order[2], "in third iteration only third and fourth task can run");
    }

    #[test]
    pub fn test_calc_execution_levels_complex() {
        let deps = vec![
            vec![], // first task
            vec![], // second task
            vec![0],
            vec![1],
            vec![],
            vec![],
            vec![2,0,3,1,4,5], // seventh task; order is irrelevant
        ];
        let execution_levels = PurtelExecutor::calc_execution_levels(deps);
        assert_eq!(3, execution_levels.len(), "should only need 3 execution levels");

        // first iteration/execution level
        assert_eq!(vec![0, 1, 4, 5], execution_levels[0]);
        assert_eq!(vec![2, 3], execution_levels[1]);
        assert_eq!(vec![6], execution_levels[2]);
    }

    #[test]
    #[should_panic]
    pub fn test_assert_no_duplicates_panic() {
        let param_usages = vec![
            vec![
                PurtelParamUsage::new("data1", READ),
                PurtelParamUsage::new("data1", WRITE)
            ],
        ];
        PurtelExecutor::assert_no_duplicates(&param_usages);
    }

}



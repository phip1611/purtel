//! Defines types used in Purtel to manage task dependencies, execution orders etc.

/// Each task in Purtel is referenced through its ID. The ID is the index
/// inside the vector with all closures (the actual tasks).
pub type TaskId = usize;

/// Defines the dependencies of a task. A task is dependent on `n` other task IDs.
/// A task can only be dependent on tasks with an ID `< curr_task_id`. In other words
/// a task can only be dependent on tasks that are defined "higher" (at lower index)
/// in the vector with all closures.
pub type TaskDependencies = Vec<TaskId>;

/// Defines which task IDs should execute per iteration level. All tasks inside the
/// same execution level are independent from each other and only dependent to tasks
/// in a previous execution level. There can't be more execution levels than there
/// are tasks.
pub type TaskExecutionLevel = Vec<TaskId>;

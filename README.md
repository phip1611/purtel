# Purtel - Phips userland runtime task execution library.

Prototype for a Task-parallel runtime systems in Rust. Focus is on code generation during compile time.


This repository contains my Rust prototype for my INF-D-960 project at TU Dresden.
The presentation was on Fri, 2020-12-04 @ 13:00.

## About

Name: Philipp Schuster\
Supervisor: Hannes Weisbach\
Supervisor 2: Michael Roitzsch

### Topic: Task-parallel runtime systems & programming models

This prototype is a simple demonstration to make "#pragma"-like code annotations in Rust code
with the goal to generate code. The annotations are similar to OpenMP, but Purtel is different. 
It uses a similar way of code annotations - of course in the Rust way. The focus of this project 
is a simple task definition and parallel execution. So far, tasks are static and the whole order 
can be calculated once during startup. There is no support for dynamic creating tasks - but this
could be added in the future. The main contribution is to present the audience a way for code 
generation in Rust during compile time. 

This project only builds with the nightly channel of Rust (1.50.0-nightly works, 1.48.0-stable doesn't work).
**(Actually it's not the lib but the bin that uses the lib that requires nightly.)**

## Task model
- each task is a closure (lambda) in Rust without return type and without parameters
- each task manages it's shared data by itself via `Arc<RwLock<T>>`
- all shared state shall be accessed via `Arc<RwLock<T>>`
    - `Arc`: atomic reference count inside each thread
    - `RwLock`: ReadWrite-Lock -> n readers or 1 writer
    - `T`: actual data
    - otherwise multiple threads would spawn but only run after each other 
      (if a Mutex would be used)
- each task describes what parameters it uses and how (read or write)
  (**This is the code that gets generated**)
- each task has a unique ID. A Task can only be dependent on tasks with a smaller ID.
  (on previous tasks)
    - circular dependencies are not possible this way
- a previous task (dependency of a task) modifies data that is also behind `Arc<RwLock<T>>`
- a dependency exist between tasks iff:
    - Read after Write
    - Write after Write
    - Write after Read
- a dependency does not exist between tasks iff:
    - Read after Read
- so far all dependencies of a previous task are also dependencies of a task
  (transitive inheritance)

## Guarantees
- if all tasks follow my task model and can run in sequentially order and terminate,
  then also the parallelized execution will terminate and be correct.

## Examples
In `src/bin` are two binaries. One binary contains all boilerplate code that is needed.
The other one uses purtel code annotations to generate this boilerplate code.

### Short code snippet
```rust
    ...
    #[purtel_tasks] {
        // consumes var "data1" read only
        // we move the var into the closure
        // (and later into a thread)
        let data1_t = data1.clone();
        #[purtel_task(read = "data1")] {}
        let task1 = move || {
            let _data1 = data1_t.try_read().unwrap();
            println!("task 1 is running");
            // we simulate an expensive task
            sleep(Duration::from_secs(1));
        };
    ...
     // Blocking
    let mut executor = PurtelExecutor::new(closures, param_usages);
    executor.calc_and_verify_exe_order();
    executor.execute();
```


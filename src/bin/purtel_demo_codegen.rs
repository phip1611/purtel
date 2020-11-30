//! purtel demo with code generation of parameter usage.

//! We need rust nightly because of to use procedural macros on expressions.
//! 1) `#![feature(stmt_expr_attributes)]`
//! 2) `#![feature(proc_macro_hygiene)]`
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]

use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use purtel::{PurtelExecutor, PurtelTask};
use purtel::{purtel_task, purtel_tasks};

fn main() {
    // All params for the tasks
    let data1 = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let data2 = Arc::new(RwLock::new(vec![42]));


    // this will analyze all purtel task metadata,
    // UNWRAP(!) the inner block and finally execute everything
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

        let data1_t = data1.clone();
        let data2_t = data2.clone();
        // consumes data1 and data2 read + write
        #[purtel_task(write = "data1, data2")] {}
        let task2 = move || {
            let _data1 = data1_t.try_write().unwrap();
            let _data2 = data2_t.try_write().unwrap();
            println!("task 2 is running");
            sleep(Duration::from_secs(1));
        };

        let data1_t = data1.clone();
        // consumes data1 read only
        #[purtel_task(read = "data1")] {}
        let task3 = move || {
            let _data1 = data1_t.try_read().unwrap();
            println!("task 3 is running");
            sleep(Duration::from_secs(1));
        };

        let data2_t = data2.clone();
        // consumes data2 read only
        #[purtel_task(read = "data2")] {}
        let task4 = move || {
            let _data2 = data2_t.try_read().unwrap();
            println!("task 4 is running");
            sleep(Duration::from_secs(1));
        };

        let data2_t = data2.clone();
        // consumes data2 read only
        #[purtel_task(read = "data2")] {}
        let task5 = move || {
            let _data2 = data2_t.try_read().unwrap();
            println!("task 5 is running");
            sleep(Duration::from_secs(1));
        };

        let data2_t = data2.clone();
        // consumes data2 read only
        #[purtel_task(read = "data2")] {}
        let task6 = move || {
            let _data2 = data2_t.try_read().unwrap();
            println!("task 6 is running");
            sleep(Duration::from_secs(1));
        };
        let data2_t = data2.clone();
        // consumes data2 read only
        #[purtel_task(read = "data2")] {}
        let task7 = move || {
            let _data2 = data2_t.try_read().unwrap();
            println!("task 7 is running");
            sleep(Duration::from_secs(1));
        };
        let data2_t = data2.clone();
        // consumes data2 read only
        #[purtel_task(read = "data2")] {}
        let task8 = move || {
            let _data2 = data2_t.try_read().unwrap();
            println!("task 8 is running");
            sleep(Duration::from_secs(1));
        };

        let data2_t = data2.clone();
        // consumes data2 read only
        #[purtel_task(read = "data2")] {}
        let task9 = move || {
            let _data2 = data2_t.try_read().unwrap();
            println!("task 9 is running");
            sleep(Duration::from_secs(1));
        };
    };

    // this can be easily constructed by a regular macro in future
    let closures = vec![
        PurtelTask::new(Box::from(task1)),
        PurtelTask::new(Box::from(task2)),
        PurtelTask::new(Box::from(task3)),
        PurtelTask::new(Box::from(task4)),
        PurtelTask::new(Box::from(task5)),
        PurtelTask::new(Box::from(task6)),
        PurtelTask::new(Box::from(task7)),
        PurtelTask::new(Box::from(task8)),
        PurtelTask::new(Box::from(task9)),
    ];

    // Blocking
    let mut executor = PurtelExecutor::new(closures, param_usages);
    executor.calc_and_verify_exe_order();
    executor.execute();
}



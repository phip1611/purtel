//! purtel demo with manual declaration of parameter usage.

use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use purtel::{PurtelExecutor, PurtelTask, PurtelParamUsage};
use purtel::PurtelParamUsageKind::{READ, WRITE};

fn main() {
    // All params for the tasks
    let data1 = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let data2 = Arc::new(RwLock::new(vec![42]));

    let data1_t = data1.clone();
    // consumes data1 read only
    let task1 = move || {
        let _data1 = data1_t.try_read().unwrap();
        println!("task 1 is running");
        sleep(Duration::from_secs(1));
    };

    let data1_t = data1.clone();
    let data2_t = data2.clone();
    // consumes data1 and data2 read + write
    let task2 = move || {
        let _data1 = data1_t.try_write().unwrap();
        let _data2 = data2_t.try_write().unwrap();
        println!("task 2 is running");
        sleep(Duration::from_secs(1));
    };

    let data1_t = data1.clone();
    // consumes data1 read only
    let task3 = move || {
        let _data1 = data1_t.try_read().unwrap();
        println!("task 3 is running");
        sleep(Duration::from_secs(1));
    };

    let data2_t = data2.clone();
    // consumes data2 read only
    let task4 = move || {
        let _data2 = data2_t.try_read().unwrap();
        println!("task 4 is running");
        sleep(Duration::from_secs(1));
    };


    let data2_t = data2.clone();
    // consumes data2 read only
    
    let task5 = move || {
        let _data2 = data2_t.try_read().unwrap();
        println!("task 5 is running");
        sleep(Duration::from_secs(1));
    };

    let data2_t = data2.clone();
    // consumes data2 read only
    let task6 = move || {
        let _data2 = data2_t.try_read().unwrap();
        println!("task 6 is running");
        sleep(Duration::from_secs(1));
    };
    let data2_t = data2.clone();
    // consumes data2 read only
    let task7 = move || {
        let _data2 = data2_t.try_read().unwrap();
        println!("task 7 is running");
        sleep(Duration::from_secs(1));
    };
    let data2_t = data2.clone();
    // consumes data2 read only
    let task8 = move || {
        let _data2 = data2_t.try_read().unwrap();
        println!("task 8 is running");
        sleep(Duration::from_secs(1));
    };

    let data2_t = data2.clone();
    // consumes data2 read only
    let task9 = move || {
        let _data2 = data2_t.try_read().unwrap();
        println!("task 9 is running");
        sleep(Duration::from_secs(1));
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

    // This get's generated when using the codegen procedural macros.
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
        vec![PurtelParamUsage::new("data2", READ)],
        vec![PurtelParamUsage::new("data2", READ)],
        vec![PurtelParamUsage::new("data2", READ)],
        vec![PurtelParamUsage::new("data2", READ)],
        // ninth task
        vec![PurtelParamUsage::new("data2", READ)],
    ];
    // Generation end

    // Blocking
    let mut executor = PurtelExecutor::new(closures, param_usages);
    executor.calc_and_verify_exe_order();
    executor.execute();
}



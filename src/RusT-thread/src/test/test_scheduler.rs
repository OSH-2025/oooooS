use crate::rtthread::scheduler::*;
use crate::rtthread::thread::{RtThread, rt_thread_create};
use crate::rtdef::ThreadState;
use cortex_m_semihosting::hprintln;



pub extern "C" fn thread1_enter(arg: usize) -> () {
    let mut i = arg;
    hprintln!("thread1: {}", i);
    loop {
        i += 1;
        hprintln!("thread1: {}", i);
        if i > 10000 {
            rt_schedule();
            break;
        }
    }
}

pub extern "C" fn thread2_enter(arg: usize) -> () {
    let mut i = arg;
    hprintln!("thread2: {}", i);
    loop {
        i += 1;
        hprintln!("thread2: {}", i);
    }
}
// test1： 线程插入与删除
pub fn test_insert_thread() {
    // insert thread
    let thread1 = rt_thread_create("thread1",thread1_enter as usize, 1024,4,1000);
    insert_thread(thread1.clone());
    let thread2 = rt_thread_create("thread2",thread1_enter as usize, 1024,4,1000);
    insert_thread(thread2);
    let thread3 = rt_thread_create("thread3",thread1_enter as usize, 1024,11,1000);
    insert_thread(thread3);
    
    look_at_priority_table();

    // remove thread
    remove_thread(thread1);

    look_at_priority_table();

    // pop thread
    let thread4 = pop_thread(4);
    hprintln!("thread4: {:?}", thread4);
    look_at_priority_table();



    
}

fn look_at_priority_table(){
    output_priority_table();
    let highest_priority = get_highest_priority();
    hprintln!("highest_priority: {}", highest_priority);
    let highest_priority_thread = get_highest_priority_thread();
    hprintln!("highest_priority_thread: {:?}", highest_priority_thread);
}

// test2: 调度开始
pub fn test_schedule_start() {
    // insert thread
    let thread1 = rt_thread_create("thread1",thread1_enter as usize, 1024,1,1000);
    insert_thread(thread1.clone());
    let thread2 = rt_thread_create("thread2",thread2_enter as usize, 1024,1,1000);
    insert_thread(thread2);
    let thread3 = rt_thread_create("thread3",thread2_enter as usize, 1024,11,1000);
    insert_thread(thread3);

    // start schedule
    rt_schedule_start();

}


pub fn test_schedule(){
    let thread1 = rt_thread_create("thread1",thread1_enter as usize, 1024,1,1000);
    insert_thread(thread1.clone());
    let thread2 = rt_thread_create("thread2",thread2_enter as usize, 1024,1,1000);
    insert_thread(thread2);
    let thread3 = rt_thread_create("thread3",thread2_enter as usize, 1024,11,1000);
    insert_thread(thread3);

    hprintln!("A");
    // start schedule
    rt_schedule_start();

    // schedule
    rt_schedule();

}
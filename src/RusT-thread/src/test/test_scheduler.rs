use crate::rtthread::scheduler::*;
use crate::rtthread::thread::*;
use crate::rtdef::ThreadState;
use cortex_m_semihosting::hprintln;



pub extern "C" fn thread_enter(arg: usize) -> () {
    let mut i = arg;
    hprintln!("thread1: {}", i);

}

// test1： 线程插入与删除
pub fn test_insert_thread() {
    hprintln!("in test1:");
    let thread = rt_thread_create("thread1",thread_enter as usize, 1024,1,1000);
    hprintln!("after create thread1:");
    insert_thread(thread);

    let thread2 = rt_thread_create("thread2",thread_enter as usize, 1024,2,1000);
    insert_thread(thread2);
    hprintln!("after insert:");
    output_priority_table();
}




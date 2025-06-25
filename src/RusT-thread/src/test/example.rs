//! 示例代码
//! 
//! 示例代码的目的是展示如何使用rtthread_rt库

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use cortex_m_semihosting::hprintln;

pub extern "C" fn example_thread_1(arg: usize) -> () {
    hprintln!("example_thread_1...");
    let mut start_tick = rt_tick_get();
    hprintln!("start_tick: {:?}", start_tick);
    loop {
        if rt_tick_get() - start_tick > 100 {
            hprintln!("example_thread_1...");
            start_tick = rt_tick_get();
        }
    }
}

pub extern "C" fn example_thread_2(arg: usize) -> () {
    hprintln!("example_thread_2...");
    let mut start_tick = rt_tick_get();
    hprintln!("start_tick: {:?}", start_tick);
    loop {
        // hprintln!("rt_tick_get(): {:?}", rt_tick_get());
        if rt_tick_get() - start_tick > 100 {
            hprintln!("example_thread_2...");
            start_tick = rt_tick_get();
        }
    }
}

pub fn run_example() {
    // hprintln!("run_example...");
    // hprintln!("rt_thread_create...");
    let thread_1 = rt_thread_create("example_thread_1", example_thread_1 as usize, 2*1024, 10, 1000);
    // hprintln!("thread_1: {:?}", thread_1);
    let thread_2 = rt_thread_create("example_thread_2", example_thread_2 as usize, 2*1024, 10, 1000);
    // hprintln!("thread_2: {:?}", thread_2);

    let level = rt_hw_interrupt_disable();
    rt_thread_startup(thread_1);
    rt_thread_startup(thread_2);
    rt_hw_interrupt_enable(level);

}





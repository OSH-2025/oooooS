//! 示例代码
//! 
//! 示例代码的目的是展示如何使用rtthread_rt库

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use cortex_m_semihosting::hprintln;

pub extern "C" fn example_thread_1(arg: usize) -> () {
    hprintln!("example_thread_1...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("start_tick: {:?}", start_tick);
    loop {
        if rt_tick_get() - start_tick > 100 {
            hprintln!("example_thread_1...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 6000 {
            // rt_thread_suspend(rt_thread_self().unwrap());
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
}

pub extern "C" fn example_thread_2(arg: usize) -> () {
    hprintln!("example_thread_2...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("start_tick: {:?}", start_tick);
    loop {
        // hprintln!("rt_tick_get(): {:?}", rt_tick_get());
        if rt_tick_get() - start_tick > 100 {
            hprintln!("example_thread_2...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 10000 {
            // rt_thread_suspend(rt_thread_self().unwrap());
            if rt_thread_self().unwrap().inner.exclusive_access().current_priority == 10 {
                rt_thread_set_priority(rt_thread_self().unwrap(), 14);
            }
        }
    }
}

pub extern "C" fn example_thread_3(arg: usize) -> () {
    hprintln!("example_thread_3...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("start_tick: {:?}", start_tick);
    loop {
        if rt_tick_get() - start_tick > 100 {
            hprintln!("example_thread_3...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 10000 {
            // rt_thread_suspend(rt_thread_self().unwrap());
            if rt_thread_self().unwrap().inner.exclusive_access().current_priority == 10 {
                rt_thread_set_priority(rt_thread_self().unwrap(), 14);
            }
        }
    }
}

pub extern "C" fn example_thread_4(arg: usize) -> () {
    hprintln!("example_thread_4...");
    let mut start_tick = rt_tick_get();
    hprintln!("start_tick: {:?}", start_tick);
    loop {
        if rt_tick_get() - start_tick > 100 {
            hprintln!("example_thread_4...");
            start_tick = rt_tick_get();
        }
    }
}

pub fn run_example() {
    // hprintln!("run_example...");
    let thread_1 = rt_thread_create("example_thread_1", example_thread_1 as usize, 2*1024, 10, 1000);
    let thread_2 = rt_thread_create("example_thread_2", example_thread_2 as usize, 2*1024, 10, 1000);
    let thread_3 = rt_thread_create("example_thread_3", example_thread_3 as usize, 2*1024, 10, 1000);
    let thread_4 = rt_thread_create("example_thread_4", example_thread_4 as usize, 2*1024, 14, 1000);

    let level = rt_hw_interrupt_disable();
    rt_thread_startup(thread_1);
    rt_thread_startup(thread_2.clone());
    rt_thread_sleep(thread_2.clone(), 10000);
    rt_thread_startup(thread_3);
    rt_thread_startup(thread_4);
    rt_hw_interrupt_enable(level);

}





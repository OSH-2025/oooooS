//! 示例代码
//! 
//! 示例代码的目的是展示如何使用rtthread_rt库

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use cortex_m_semihosting::hprintln;

/// 测试线程挂起与删除
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

/// 测试线程优先级变化 + 睡眠
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
        if rt_tick_get() - tic > 20000 {
            // rt_thread_suspend(rt_thread_self().unwrap());
            if rt_thread_self().unwrap().inner.exclusive_access().current_priority == 10 {
                rt_thread_set_priority(rt_thread_self().unwrap(), 14);
            }
        }
    }
}

/// 测试线程优先级变化
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
        if rt_tick_get() - tic > 20000 {
            // rt_thread_suspend(rt_thread_self().unwrap());
            if rt_thread_self().unwrap().inner.exclusive_access().current_priority == 10 {
                rt_thread_set_priority(rt_thread_self().unwrap(), 14);
            }
        }
    }
}

/// 一个简单的线程
pub extern "C" fn example_thread_4(arg: usize) -> () {
    hprintln!("example_thread_4...");
    hprintln!("example_thread_4: level: {}", rt_hw_get_interrupt_level());
    let mut start_tick = rt_tick_get();
    hprintln!("start_tick: {:?}", start_tick);
    rt_thread_sleep(rt_thread_self().unwrap(), 1000);
    loop {
        if rt_tick_get() - start_tick > 100 {
            hprintln!("example_thread_4...");
            start_tick = rt_tick_get();
        }
    }
}

/// 测试线程创建与启动
/// [在抢占优先级+RR调度下]
/// 期望现象：
/// 1. 最开始，1、3交替运行（2睡眠，4优先级不够）
/// 2. 之后，1被挂起，3独占CPU
/// 3. 然后，2被唤醒，2，3交替运行（3先降低优先级，可能有一段时间2独占CPU）
/// 4. 最后，3，4优先级降低至与4相同，2，3，4交替运行
pub fn run_example() {
    hprintln!("run_example: level: {}", rt_hw_get_interrupt_level());
    // hprintln!("run_example...");
    let thread_1 = rt_thread_create("example_thread_1", example_thread_1 as usize, 2*1024, 10, 1000);
    let thread_2 = rt_thread_create("example_thread_2", example_thread_2 as usize, 2*1024, 17, 1000);
    let thread_3 = rt_thread_create("example_thread_3", example_thread_3 as usize, 2*1024, 10, 1000);
    let thread_4 = rt_thread_create("example_thread_4", example_thread_4 as usize, 2*1024, 14, 1000);

    let level = rt_hw_interrupt_disable();
    hprintln!("run_example after disable: level: {}", level);
    rt_thread_startup(thread_1);
    rt_thread_startup(thread_2.clone());
    rt_thread_sleep(thread_2.clone(), 10000);
    rt_thread_startup(thread_3);
    rt_thread_startup(thread_4);
    rt_hw_interrupt_enable(level);

}





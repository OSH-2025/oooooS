//! 时钟模块
//! 
//! 定义了时钟相关的函数和变量

#![warn(unused_imports)]

use lazy_static::lazy_static;
use crate::rtthread_rt::kservice::RTIntrFreeCell;
use crate::rtthread_rt::rtconfig::RT_TICK_PER_SECOND;
use crate::rtthread_rt::hardware::irq::{rt_hw_interrupt_disable, rt_hw_interrupt_enable};
use crate::rtthread_rt::timer::timer::rt_timer_check;
use crate::rtthread_rt::rtdef::RT_THREAD_STAT_YIELD;
use crate::rtthread_rt::thread::*;
use cortex_m_semihosting::hprintln;

//调试
// use cortex_m_semihosting::hprintln;

pub const RT_WAITING_FOREVER: u32 = 0xFFFFFFFF;

/// 定义全局变量：当前时钟，使用RTIntrFreeCell包裹，实现中断安全
lazy_static! {
    static ref RT_TICK: RTIntrFreeCell<u32> = unsafe { RTIntrFreeCell::new(0) };
}

/// 获取当前时钟周期
pub fn rt_tick_get() -> u32 {
    *RT_TICK.exclusive_access()
}

/// 设置当前时钟周期
pub fn rt_tick_set(tick: u32) {
    *RT_TICK.exclusive_access() = tick;
}

/// 时钟中断处理函数
pub fn rt_tick_increase() {

    let level = rt_hw_interrupt_disable();
    *RT_TICK.exclusive_access() +=1 ;

    if let Some(thread) = rt_thread_self() {
        thread.inner.exclusive_access().remaining_tick -= 1;
        
        if thread.inner.exclusive_access().remaining_tick == 0 {
            // hprintln!("rt_tick_increase: yield");
            // hprintln!("thread: {:?}", thread.clone());
            rt_thread_yield();
        }
    }


    rt_hw_interrupt_enable(level);
    // 检查定时器
    rt_timer_check();
    //调试，输出当前时钟周期 
    // hprintln!("Current tick: {}", rt_tick_get());
}

/// 将毫秒转换为时钟周期
pub fn rt_tick_from_millisecond(ms: i32) -> u32 {
    if ms < 0 {
        RT_WAITING_FOREVER
    } else {
        let tick = RT_TICK_PER_SECOND * (ms as u32 / 1000);
        let tick = tick + (RT_TICK_PER_SECOND * (ms as u32 % 1000) + 999) / 1000;
        tick
    }
}

/// 获取自启动以来经过的毫秒数
pub fn rt_tick_get_millisecond() -> u32 {
    if 1000 % RT_TICK_PER_SECOND == 0 {
        rt_tick_get() * (1000 / RT_TICK_PER_SECOND)
    } else {
        // 错误情况，直接返回0
        0
    }
}
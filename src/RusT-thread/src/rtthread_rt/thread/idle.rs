//! 空闲线程
//! 
//! 定义了空闲线程的入口函数和初始化函数

#![warn(unused_imports)]

use cortex_m::asm;
use cortex_m_semihosting::hprintln;

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::rtconfig;
use crate::rtthread_rt::thread::thread_priority_table;
use crate::rtthread_rt::rtdef::ThreadState;

/// 空闲线程入口函数
/// 用户可以在这里实现自己的空闲线程逻辑（默认是空循环）
pub extern "C" fn idle_entry(arg: usize) -> () {
    hprintln!("idle_entry...");
    loop{
        hprintln!("idle_entry loop...");
        asm::nop;
        rt_thread_yield();
    }
    hprintln!("idle_entry finished.");
}
/// 初始化空闲线程
/// 即创建一个空闲线程，并将其插入到就绪队列中
pub fn init_idle(){
    // hprintln!("Initializing idle...");
    let idle = rt_thread_create("idle", idle_entry as usize, 1024, (rtconfig::RT_THREAD_PRIORITY_MAX - 1) as u8, 1000);
    idle.inner.exclusive_access().stat = ThreadState::Ready;
    thread_priority_table::insert_thread(idle.clone());
    // hprintln!("Idle initialized.");
}
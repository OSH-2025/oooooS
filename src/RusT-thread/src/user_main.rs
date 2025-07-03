//! 用户主线程入口
//! 
//! 用户可以在这里实现自己的主线程逻辑
//! 主线程的优先级可在rtconfig.rs中配置，默认是0
//! 主线程的栈大小可在rtconfig.rs中配置，默认是256
//! 主线程的入口函数是main_entry，参数是usize类型
//! 用户主线程的入口函数必须以extern "C" 声明，参数是usize类型
//! 用户主线程的入口函数必须以pub extern "C" 声明，参数是usize类型

use crate::test::comprehensive_example;
use crate::test::example_mfq;
use crate::test::example;
use crate::test::performance_test;
use crate::test::switch_time_test;
use crate::test::test_interruput_latency;
use crate::rtthread_rt::thread::rt_thread_yield;
use crate::rtthread_rt::timer::rt_tick_get;

use cortex_m_semihosting::hprintln;
use cortex_m::asm;

// 用户主线程入口
pub extern "C" fn main_entry(arg: usize) -> () {
    hprintln!("main_entry...");
    // example::run_example();
    // comprehensive_example::run_comprehensive_demo();
    // example_mfq::run_example();
    // performance_test::run_performance_test();
    // switch_time_test::test_thread_switch_time();
    // test_interruput_latency::run_precise_interrupt_latency_test();
    // 用户主线程入口
    let mut tick = rt_tick_get();
    loop{
        if rt_tick_get() - tick > 10000 {
            hprintln!("main_entry loop... at tick: {}", rt_tick_get());
            tick = rt_tick_get();
        }
        asm::nop;
    }
}

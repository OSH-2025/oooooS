//! 线程创建时间测试
//! 
//! 使用SysTick计数器测量线程创建所需的时间
//! 通过批量创建多个线程并计算平均值，获得更准确的测量结果

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_semihosting::hprintln;
use crate::rtthread_rt::timer::clock::*;
use crate::rtthread_rt::thread::rt_thread_create;

// 空闲线程入口函数
fn idle_entry() {
    loop {
        cortex_m::asm::nop();
    }
}

// 测试线程创建时间
pub fn test_thread_creation_time(thread_count: usize) {
    hprintln!("\n======= 线程创建时间测试 =======");
    
    // 测试参数
    let stack_size = 128;  // 线程栈大小
    let priority = 10;     // 线程优先级
    let tick = 10;         // 时间片大小
    
    // 记录开始时间
    let start_ticks = rt_tick_get();
    
    // 批量创建线程
    for i in 0..thread_count {
        let thread = rt_thread_create(
            "test",
            idle_entry as usize,
            stack_size,
            priority,
            tick
        );
    }
    
    // 记录结束时间
    let end_ticks = rt_tick_get();
    let total_ticks = end_ticks - start_ticks;
    
    // 计算每个线程的平均创建时间
    let avg_ticks_per_thread = total_ticks as f32 / thread_count as f32;
    
    let avg_time_us = rt_tick_to_us(total_ticks) as f32 / thread_count as f32;
    
    hprintln!("创建线程数量: {} 个", thread_count);
    hprintln!("总时钟周期: {} tick", total_ticks);
    hprintln!("平均每个线程创建时间: {:.2} tick", avg_ticks_per_thread);
    hprintln!("平均每个线程创建时间: {:.2} 微秒", avg_time_us);
    hprintln!("==============================\n");
}

// 运行测试
pub fn run_thread_creation_benchmark() {
    // 测试不同数量线程的创建时间
    test_thread_creation_time(50);  // 50个线程
    
    hprintln!("线程创建时间测试完成！");
} 
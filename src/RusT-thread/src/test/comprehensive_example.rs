//! 全面示例代码
//! 
//! 本示例展示了rust-thread库的所有主要功能，包括：
//! 1. 线程管理（创建、启动、挂起、恢复、删除、优先级控制）
//! 2. 定时器功能（单次定时器、周期定时器、定时器控制）
//! 3. 调度策略（优先级调度、多级反馈队列调度）
//! 4. 线程同步（睡眠、让出CPU）
//! 5. 中断管理
//! 6. 性能测试

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use crate::rtthread_rt::rtdef::ThreadState;
use cortex_m_semihosting::hprintln;

extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, Ordering};
use lazy_static::lazy_static;
use crate::rtthread_rt::kservice::RTIntrFreeCell;
use spin::Mutex;

// 全局计数器用于演示
lazy_static! {
    static ref THREAD_COUNTER: AtomicU32 = AtomicU32::new(0);
    static ref TIMER_COUNTER: AtomicU32 = AtomicU32::new(0);
    static ref SHARED_DATA: RTIntrFreeCell<Vec<u32>> = unsafe { RTIntrFreeCell::new(Vec::new()) };
}

/// 基础线程示例 - 展示线程生命周期
pub extern "C" fn basic_thread_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("基础线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut last_print = start_tick;
    
    loop {
        let current_tick = rt_tick_get();
        
        // 每100个tick打印一次
        if current_tick - last_print > 100 {
            hprintln!("基础线程 #{} 运行中... tick: {}", thread_id, current_tick);
            last_print = current_tick;
        }
        
        // 运行10秒后退出
        if current_tick - start_tick > 10000 {
            hprintln!("基础线程 #{} 完成，准备退出", thread_id);
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
}

/// 优先级变化演示线程
pub extern "C" fn priority_change_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("优先级变化线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut last_print = start_tick;
    let mut priority_phase = 0;
    
    loop {
        let current_tick = rt_tick_get();
        
        // 每200个tick打印一次
        if current_tick - last_print > 200 {
            let current_priority = rt_thread_self().unwrap().inner.exclusive_access().current_priority;
            hprintln!("优先级变化线程 #{} 运行中... 当前优先级: {}", thread_id, current_priority);
            last_print = current_tick;
        }
        
        // 每5秒改变一次优先级
        if current_tick - start_tick > 5000 * (priority_phase + 1) {
            priority_phase += 1;
            let new_priority = match priority_phase {
                1 => 15, // 降低优先级
                2 => 8,  // 提高优先级
                3 => 20, // 再次降低
                _ => {
                    hprintln!("优先级变化线程 #{} 完成", thread_id);
                    rt_thread_delete(rt_thread_self().unwrap());
                    return;
                }
            };
            
            hprintln!("优先级变化线程 #{} 改变优先级为: {}", thread_id, new_priority);
            rt_thread_set_priority(rt_thread_self().unwrap(), new_priority);
        }
    }
}

/// 睡眠演示线程
pub extern "C" fn sleep_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("睡眠演示线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut sleep_count = 0;
    
    loop {
        let current_tick = rt_tick_get();
        
        hprintln!("睡眠演示线程 #{} 准备睡眠 #{}", thread_id, sleep_count + 1);
        
        // 睡眠1秒
        rt_thread_sleep(rt_thread_self().unwrap(), 1000);
        
        sleep_count += 1;
        hprintln!("睡眠演示线程 #{} 醒来，已睡眠 {} 次", thread_id, sleep_count);
        
        // 睡眠5次后退出
        if sleep_count >= 5 {
            hprintln!("睡眠演示线程 #{} 完成", thread_id);
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
}

/// 让出CPU演示线程
pub extern "C" fn yield_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("让出CPU演示线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut yield_count = 0;
    
    loop {
        let current_tick = rt_tick_get();
        
        hprintln!("让出CPU演示线程 #{} 让出CPU #{}", thread_id, yield_count + 1);
        
        // 让出CPU
        rt_thread_yield();
        
        yield_count += 1;
        hprintln!("让出CPU演示线程 #{} 重新获得CPU", thread_id);
        
        // 让出10次后退出
        if yield_count >= 10 {
            hprintln!("让出CPU演示线程 #{} 完成", thread_id);
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
}

/// 定时器回调函数
fn timer_callback() {
    let timer_id = TIMER_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("定时器回调 #{} 触发，当前tick: {}", timer_id, rt_tick_get());
    
    // 向共享数据添加时间戳
    SHARED_DATA.exclusive_access().push(rt_tick_get());
}

/// 定时器演示线程
pub extern "C" fn timer_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("定时器演示线程 #{} 启动", thread_id);
    
    // 创建单次定时器
    let oneshot_timer = Arc::new(Mutex::new(RtTimer::new(
        "oneshot_timer",
        0,
        0, // 单次定时器
        Some(Box::new(timer_callback)),
        2000, // 2秒后触发
        2000,
    )));
    
    // 创建周期定时器
    let periodic_timer = Arc::new(Mutex::new(RtTimer::new(
        "periodic_timer",
        0,
        2, // 周期定时器
        Some(Box::new(timer_callback)),
        1000, // 每1秒触发一次
        1000,
    )));
    
    hprintln!("启动单次定时器（2秒后触发）");
    rt_timer_start(oneshot_timer.clone());
    
    hprintln!("启动周期定时器（每1秒触发）");
    rt_timer_start(periodic_timer.clone());
    
    let start_tick = rt_tick_get();
    
    loop {
        let current_tick = rt_tick_get();
        
        // 运行15秒后停止定时器并退出
        if current_tick - start_tick > 15000 {
            hprintln!("停止定时器");
            rt_timer_stop(&oneshot_timer);
            rt_timer_stop(&periodic_timer);
            
            // 显示收集的数据
            let data = SHARED_DATA.exclusive_access();
            hprintln!("定时器触发次数: {}", data.len());
            if !data.is_empty() {
                hprintln!("第一次触发时间: {}", data[0]);
                hprintln!("最后一次触发时间: {}", data[data.len() - 1]);
            }
            
            hprintln!("定时器演示线程 #{} 完成", thread_id);
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
}

/// 调度策略演示线程
pub extern "C" fn scheduling_policy_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("调度策略演示线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut last_print = start_tick;
    
    loop {
        let current_tick = rt_tick_get();
        
        // 每500个tick打印一次
        if current_tick - last_print > 500 {
            let current_priority = rt_thread_self().unwrap().inner.exclusive_access().current_priority;
            let policy_name = get_current_scheduling_policy_name();
            hprintln!("调度策略演示线程 #{} 运行中... 优先级: {}, 策略: {}", 
                     thread_id, current_priority, policy_name);
            last_print = current_tick;
        }
        
        // 运行8秒后退出
        if current_tick - start_tick > 8000 {
            hprintln!("调度策略演示线程 #{} 完成", thread_id);
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
}

/// 中断级别演示线程
pub extern "C" fn interrupt_level_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("中断级别演示线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut last_print = start_tick;
    
    loop {
        let current_tick = rt_tick_get();
        
        // 每300个tick打印一次
        if current_tick - last_print > 300 {
            let interrupt_level = rt_hw_get_interrupt_level();
            hprintln!("中断级别演示线程 #{} 运行中... 中断级别: {}", thread_id, interrupt_level);
            last_print = current_tick;
        }
        
        // 演示中断禁用/启用
        if current_tick - start_tick == 3000 {
            hprintln!("禁用中断");
            let level = rt_hw_interrupt_disable();
            hprintln!("中断已禁用，级别: {}", level);
            
            // 模拟一些关键操作
            hprintln!("执行关键操作...");
            
            hprintln!("重新启用中断");
            rt_hw_interrupt_enable(level);
        }
        
        // 运行6秒后退出
        if current_tick - start_tick > 6000 {
            hprintln!("中断级别演示线程 #{} 完成", thread_id);
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
}

/// 线程控制演示线程
pub extern "C" fn thread_control_demo(arg: usize) -> () {
    let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    hprintln!("线程控制演示线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut phase: i32 = 0;
    
    loop {
        let current_tick = rt_tick_get();
        
        match phase {
            0 => {
                if current_tick - start_tick > 2000 {
                    hprintln!("线程控制演示线程 #{} 准备挂起", thread_id);
                    phase = 1;
                    rt_thread_suspend(rt_thread_self().unwrap());
                    // 这里不会执行，因为线程已被挂起
                }
            },
            1 => {
                // 这个阶段不会执行，因为线程被挂起
                // 需要其他线程来恢复它
                hprintln!("线程控制演示线程 #{} 已被挂起", thread_id);
            },
            2 => {
                if current_tick - start_tick > 8000 {
                    hprintln!("线程控制演示线程 #{} 完成", thread_id);
                    rt_thread_delete(rt_thread_self().unwrap());
                }
            },
            _ => {
                // 处理其他情况
                break;
            }
        }
    }
}

/// 恢复挂起线程的辅助线程
pub extern "C" fn resume_helper_thread(arg: usize) -> () {
    hprintln!("恢复辅助线程启动");
    
    // 等待3秒后恢复被挂起的线程
    rt_thread_sleep(rt_thread_self().unwrap(), 3000);
    
    // 查找并恢复被挂起的线程
    // 注意：这里需要访问线程列表，但RT_THREAD_LIST是私有的
    // 我们可以通过其他方式来恢复线程，比如使用全局变量
    hprintln!("尝试恢复被挂起的线程...");
    
    hprintln!("恢复辅助线程完成");
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 运行全面演示
pub fn run_comprehensive_demo() {
    hprintln!("=== 开始全面功能演示 ===");
    hprintln!("演示中断级别: {}", rt_hw_get_interrupt_level());
    
    // 创建各种演示线程
    let basic_thread = rt_thread_create("basic_demo", basic_thread_demo as usize, 2*1024, 2, 100);
    let priority_thread = rt_thread_create("priority_demo", priority_change_demo as usize, 2*1024, 3, 100);
    let sleep_thread = rt_thread_create("sleep_demo", sleep_demo as usize, 2*1024, 4, 100);
    let yield_thread = rt_thread_create("yield_demo", yield_demo as usize, 2*1024, 5, 100);
    let timer_thread = rt_thread_create("timer_demo", timer_demo as usize, 2*1024, 6, 100);
    let policy_thread = rt_thread_create("policy_demo", scheduling_policy_demo as usize, 2*1024, 7, 100);
    let interrupt_thread = rt_thread_create("interrupt_demo", interrupt_level_demo as usize, 2*1024, 8, 100);
    let control_thread = rt_thread_create("thread_control_demo", thread_control_demo as usize, 2*1024, 9, 100);
    let resume_thread = rt_thread_create("resume_helper", resume_helper_thread as usize, 2*1024, 10, 100);
    
    // 启动所有线程
    let level = rt_hw_interrupt_disable();
    
    // 先使用优先级调度
    hprintln!("使用优先级调度策略");
    set_priority_scheduling();
    
    // rt_thread_startup(basic_thread);
    // rt_thread_startup(priority_thread);
    // rt_thread_startup(sleep_thread);
    // rt_thread_startup(yield_thread);
    rt_thread_startup(timer_thread);
    // rt_thread_startup(policy_thread);
    // rt_thread_startup(interrupt_thread);
    // rt_thread_startup(control_thread);
    // rt_thread_startup(resume_thread);
    
    rt_hw_interrupt_enable(level);
    
    hprintln!("所有演示线程已启动");
}

/// 运行多级反馈队列调度演示
pub fn run_mfq_demo() {
    hprintln!("=== 开始多级反馈队列调度演示 ===");
    
    // 创建多个相同优先级的线程来演示MFQ调度
    let threads: Vec<Arc<RtThread>> = (0..5).map(|i| {
        let name = alloc::format!("mfq_thread_{}", i);
        rt_thread_create(&name, mfq_demo_thread as usize, 2*1024, 15, 50)
    }).collect();
    
    let level = rt_hw_interrupt_disable();
    
    // 使用多级反馈队列调度
    hprintln!("切换到多级反馈队列调度策略");
    set_mfq_scheduling();
    
    // 启动所有线程
    for thread in threads.iter() {
        rt_thread_startup(thread.clone());
    }
    
    rt_hw_interrupt_enable(level);
    
    hprintln!("MFQ演示线程已启动");
}

/// MFQ演示线程
pub extern "C" fn mfq_demo_thread(arg: usize) -> () {
    let thread_id = arg;
    hprintln!("MFQ线程 #{} 启动", thread_id);
    
    let start_tick = rt_tick_get();
    let mut last_print = start_tick;
    
    loop {
        let current_tick = rt_tick_get();
        
        // 每200个tick打印一次
        if current_tick - last_print > 200 {
            let current_priority = rt_thread_self().unwrap().inner.exclusive_access().current_priority;
            hprintln!("MFQ线程 #{} 运行中... 当前优先级: {}", thread_id, current_priority);
            last_print = current_tick;
        }
        
        // 运行5秒后退出
        if current_tick - start_tick > 5000 {
            hprintln!("MFQ线程 #{} 完成", thread_id);
            rt_thread_delete(rt_thread_self().unwrap());
        }
    }
} 
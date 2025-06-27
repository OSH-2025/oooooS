use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use cortex_m_semihosting::hprintln;

extern crate alloc;
use core::str;
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use lazy_static::lazy_static;
use crate::rtthread_rt::kservice::RTIntrFreeCell;


// 用于线程切换测试的全局变量
lazy_static! {
    static ref SWITCH_COUNT: AtomicU32 = AtomicU32::new(0);
    static ref PARTNER_THREAD: RTIntrFreeCell<Option<Arc<RtThread>>> = unsafe { RTIntrFreeCell::new(None) };
    static ref START_TIME: AtomicU32 = AtomicU32::new(0);
    static ref END_TIME: AtomicU32 = AtomicU32::new(0);
    static ref SWITCH_COMPLETED: AtomicU32 = AtomicU32::new(0);
}

/// 用于测试线程切换时间的测试线程入口函数 - 修改版
pub extern "C" fn thread_switch_test_entry() -> () {
    let thread = rt_thread_self().unwrap();
    let name = thread.thread_name();
    
    hprintln!("{} 已启动", name);
    
    // 等待伙伴线程赋值完成
    while PARTNER_THREAD.exclusive_access().is_none() {
        rt_thread_yield();
    }
    
    // 第一个线程负责初始化计时
    if name == "switch1" {
        // 记录开始时间
        START_TIME.store(rt_tick_get(), Ordering::SeqCst);
        // 恢复伙伴线程
        if let Some(partner) = PARTNER_THREAD.exclusive_access().clone() {
            // 增加切换计数
            SWITCH_COUNT.fetch_add(1, Ordering::SeqCst);
            rt_thread_resume(partner);
            // 挂起自己
            rt_thread_suspend(thread.clone());
        }
    }
    
    // 切换循环
    const TOTAL_SWITCHES: u32 = 50; // 总切换次数
    
    while SWITCH_COUNT.load(Ordering::SeqCst) < TOTAL_SWITCHES {
        // 增加切换计数
        let count = SWITCH_COUNT.fetch_add(1, Ordering::SeqCst);
        
        // 最后一次切换时记录结束时间
        if count == TOTAL_SWITCHES - 1 {
            END_TIME.store(rt_tick_get(), Ordering::SeqCst);
            SWITCH_COMPLETED.store(1, Ordering::SeqCst);
        }
        
        // 恢复伙伴线程
        if let Some(partner) = PARTNER_THREAD.exclusive_access().clone() {
            rt_thread_resume(partner);
            // 如果已经完成所有切换，则不再挂起自己
            if SWITCH_COMPLETED.load(Ordering::SeqCst) == 0 {
                // 挂起自己
                rt_thread_suspend(thread.clone());
            }
        }
    }
    
    // 第一个线程负责输出结果
    if name == "switch1" && SWITCH_COMPLETED.load(Ordering::SeqCst) == 1 {
        let start = START_TIME.load(Ordering::SeqCst);
        let end = END_TIME.load(Ordering::SeqCst);
        let total_time = end - start;
        let switches = SWITCH_COUNT.load(Ordering::SeqCst);
        let avg_time = total_time as f32 / switches as f32;
        
        hprintln!("线程切换测试结果:");
        hprintln!("  总切换次数: {}", switches);
        hprintln!("  总耗时: {} tick ({} ms)", total_time, rt_tick_to_ms(total_time));
        hprintln!("  平均切换时间: {:.2} tick ({:.2} ms)", avg_time, rt_tick_to_ms(total_time) as f32 / switches as f32);
    }
    
    rt_thread_yield(); // 确保结果能输出
}

/// 测试线程切换时间
pub fn test_thread_switch_time() {
    hprintln!("开始测试线程切换时间...");
    
    // 重置测试状态
    SWITCH_COUNT.store(0, Ordering::SeqCst);
    SWITCH_COMPLETED.store(0, Ordering::SeqCst);
    *PARTNER_THREAD.exclusive_access() = None;
    
    // 1. 测试相同优先级线程之间的协作切换
    hprintln!("测试: 相同优先级线程协作切换 (50次)");
    
    // 创建两个优先级相同的线程
    let thread1 = rt_thread_create(
        "switch1",
        thread_switch_test_entry as usize,
        2*1024,
        10,  // 相同优先级
        100   
    );
    
    let thread2 = rt_thread_create(
        "switch2",
        thread_switch_test_entry as usize,
        2*1024,
        10,  // 相同优先级
        100
    );
    
    // 设置伙伴线程
    *PARTNER_THREAD.exclusive_access() = Some(thread2.clone());
    
    // 先启动线程1
    rt_thread_startup(thread1.clone());
    
    // 再启动线程2，但设置为挂起状态
    rt_thread_startup(thread2.clone());
    rt_thread_suspend(thread2.clone());
    
    // 等待测试完成
    while SWITCH_COMPLETED.load(Ordering::SeqCst) == 0 {
        rt_thread_sleep(rt_thread_self().unwrap(), 100);
    }
    
    // 再等待一段时间确保结果输出
    rt_thread_sleep(rt_thread_self().unwrap(), 500);
    
    hprintln!("线程切换时间测试完成");
}
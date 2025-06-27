use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use cortex_m_semihosting::hprintln;

extern crate alloc;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicU8;
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use lazy_static::lazy_static;
use crate::rtthread_rt::kservice::RTIntrFreeCell;



// 用于线程切换测试的全局变量
lazy_static! {
    static ref SWITCH_COUNT: AtomicU32 = AtomicU32::new(0);
    static ref THREAD_1: RTIntrFreeCell<Option<Arc<RtThread>>> = unsafe { RTIntrFreeCell::new(None) };
    static ref THREAD_2: RTIntrFreeCell<Option<Arc<RtThread>>> = unsafe { RTIntrFreeCell::new(None) };
    static ref START_TIME: AtomicU32 = AtomicU32::new(0);
    static ref END_TIME: AtomicU32 = AtomicU32::new(0);
    static ref SWITCH_COMPLETED: AtomicBool = AtomicBool::new(false);
}

const switch_nums: u32 = 500;

/// 线程1入口函数
pub extern "C" fn thread1() -> () {
    let thread = rt_thread_self().unwrap();
    let name = thread.thread_name();
    
    hprintln!("{} 已启动", name);
    
    // 等待线程2初始化完成
    while THREAD_2.exclusive_access().is_none() {
        rt_thread_yield();
    }
    
    // 记录开始时间
    START_TIME.store(rt_tick_get(), Ordering::SeqCst);
    
    let mut count = 0;
    while count < switch_nums {
        if count % 2 == 0 { // 线程1负责偶数次切换
            // 增加切换计数
            count = SWITCH_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

            // 切换到线程2
            if let Some(other) = THREAD_2.exclusive_access().clone() {
                rt_thread_resume(other);
                rt_thread_suspend(thread.clone());
            }
            
        } else {
            // 如果是奇数次切换，等待线程2切换回来
            count = SWITCH_COUNT.load(Ordering::SeqCst);
        }
    }
    if let Some(other) = THREAD_2.exclusive_access().clone() {
        rt_thread_delete(other);
    }

    // 测试完成后输出结果
    let start = START_TIME.load(Ordering::SeqCst);
    hprintln!("start: {}", start);
    let end = END_TIME.load(Ordering::SeqCst);
    hprintln!("end； {}", end);
    let total_time = end - start;
    let switches = SWITCH_COUNT.load(Ordering::SeqCst);
    let avg_time = total_time as f32 / count as f32;
    
    hprintln!("线程切换测试结果:");
    hprintln!("  总切换次数: {}", switches);
    hprintln!("  总耗时: {} tick ({} ms)", total_time, rt_tick_to_ms(total_time));
    hprintln!("  平均切换时间: {:.2} tick ({:.2} ms)", avg_time, rt_tick_to_ms(total_time) as f32 / switches as f32);
    
}

/// 线程2入口函数
pub extern "C" fn thread2() -> () {
    let thread = rt_thread_self().unwrap();
    let name = thread.thread_name();
    
    hprintln!("{} 已启动", name);
    
    let mut count = 0;
    while count < switch_nums {
        if count % 2 == 1 { // 线程2负责奇数次切换
            // 增加切换计数
            count = SWITCH_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
            
            // 如果达到最大切换次数，记录结束时间
            if count >= switch_nums {
                END_TIME.store(rt_tick_get(), Ordering::SeqCst);
                SWITCH_COMPLETED.store(true, Ordering::SeqCst);
            }

            // 切换到线程1
            if let Some(other) = THREAD_1.exclusive_access().clone() {
                rt_thread_resume(other);
                rt_thread_suspend(thread.clone());
            }

        } else {
            // 如果是偶数次切换，等待线程1切换过来
            count = SWITCH_COUNT.load(Ordering::SeqCst);
        }
    }

    rt_thread_delete(rt_thread_self().unwrap());
}

/// 测试线程切换时间
pub fn test_thread_switch_time() {
    hprintln!("开始测试线程切换时间...");
    
    // 重置测试状态
    SWITCH_COUNT.store(0, Ordering::SeqCst);
    SWITCH_COMPLETED.store(false, Ordering::SeqCst);
    *THREAD_1.exclusive_access() = None;
    *THREAD_2.exclusive_access() = None;
    
    // 测试相同优先级线程之间的直接切换
    hprintln!("测试: 相同优先级线程直接切换 (50次)");
    
    // 创建两个优先级相同的线程
    let thread1 = rt_thread_create(
        "switch1",
        thread1 as usize,
        2*1024,
        10,  // 相同优先级
        100   
    );
    
    let thread2 = rt_thread_create(
        "switch2",
        thread2 as usize,
        2*1024,
        10,  // 相同优先级
        100
    );
    
    // 保存线程引用到全局变量
    *THREAD_1.exclusive_access() = Some(thread1.clone());
    *THREAD_2.exclusive_access() = Some(thread2.clone());
    
    // 启动线程
    rt_thread_startup(thread1);
    rt_thread_startup(thread2);
    
    // // 等待测试完成
    // while !SWITCH_COMPLETED.load(Ordering::SeqCst) {
    //     rt_thread_sleep(rt_thread_self().unwrap(), 100);
    // }
    
    // // 再等待一段时间确保结果输出
    // rt_thread_sleep(rt_thread_self().unwrap(), 500);

    // rt_thread_suspend(rt_thread_self().unwrap());
    
    hprintln!("线程切换时间测试完成");
}




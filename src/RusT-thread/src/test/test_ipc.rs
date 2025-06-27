//! IPC 测试代码
//! 
//! 测试IPC基础功能，包括线程挂起、唤醒等
extern crate alloc;

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use crate::rtthread_rt::ipc::*;
use cortex_m_semihosting::hprintln;
use alloc::string::String;

/// 测试线程1：测试IPC挂起功能
pub extern "C" fn test_ipc_thread_1(arg: usize) -> () {
    hprintln!("test_ipc_thread_1 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_1 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_1 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 3000 {
            hprintln!("test_ipc_thread_1 准备挂起...");
            // 获取IPC对象并挂起当前线程        
            let ipc = rt_ipc_init("test_sem", 1);
            rt_ipc_list_suspend(ipc.clone(), rt_thread_self().unwrap());
            hprintln!("test_ipc_thread_1 已挂起");
            break;
        }
    }
}

/// 测试线程2：测试IPC唤醒功能
pub extern "C" fn test_ipc_thread_2(arg: usize) -> () {
    hprintln!("test_ipc_thread_2 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_2 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_2 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 5000 {
            hprintln!("test_ipc_thread_2 准备唤醒线程...");
            // 唤醒IPC队列中的第一个线程
            let ipc = rt_ipc_init("test_sem", 1);
            if let Some(woken_thread) = rt_ipc_list_resume(ipc.clone()) {
                hprintln!("test_ipc_thread_2 已唤醒线程: {:?}", String::from_utf8_lossy(&woken_thread.name));
            } else {
                hprintln!("test_ipc_thread_2 队列为空，没有线程可唤醒");
            }
            break;
        }
    }
}

/// 测试线程3：测试IPC优先级队列
pub extern "C" fn test_ipc_thread_3(arg: usize) -> () {
    hprintln!("test_ipc_thread_3 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_3 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_3 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 2000 {
            hprintln!("test_ipc_thread_3 准备挂起...");
            // 获取IPC对象并挂起当前线程
            let ipc = rt_ipc_init("test_sem_prio", 1);
            rt_ipc_list_suspend(ipc.clone(), rt_thread_self().unwrap());
            hprintln!("test_ipc_thread_3 已挂起");
            break;
        }
    }
}

/// 测试线程4：测试IPC全部唤醒功能
pub extern "C" fn test_ipc_thread_4(arg: usize) -> () {
    hprintln!("test_ipc_thread_4 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_4 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_4 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 8000 {
            hprintln!("test_ipc_thread_4 准备唤醒所有线程...");
            // 唤醒IPC队列中的所有线程
            let ipc = rt_ipc_init("test_sem_prio", 1);
            rt_ipc_list_resume_all(ipc);
            hprintln!("test_ipc_thread_4 已唤醒所有线程");
            break;
        }
    }
}

/// 运行IPC测试
pub fn run_ipc_test() {
    hprintln!("开始IPC测试...");
    
    // 创建测试线程
    let thread_1 = rt_thread_create("test_ipc_thread_1", test_ipc_thread_1 as usize, 2*1024, 10, 1000);
    let thread_2 = rt_thread_create("test_ipc_thread_2", test_ipc_thread_2 as usize, 2*1024, 12, 1000);
    let thread_3 = rt_thread_create("test_ipc_thread_3", test_ipc_thread_3 as usize, 2*1024, 8, 1000);
    let thread_4 = rt_thread_create("test_ipc_thread_4", test_ipc_thread_4 as usize, 2*1024, 14, 1000);

    // 禁用中断并启动线程
    let level = rt_hw_interrupt_disable();
    
    // 启动所有线程
    rt_thread_startup(thread_1);
    rt_thread_startup(thread_2);
    rt_thread_startup(thread_3);
    rt_thread_startup(thread_4);
    
    // 重新启用中断
    rt_hw_interrupt_enable(level);
    
    hprintln!("IPC测试线程已启动");
}

/// 测试IPC初始化功能
pub fn test_ipc_init() {
    hprintln!("测试IPC初始化...");
    
    // 测试创建IPC对象
    let ipc1 = rt_ipc_init("test_semaphore", 1);
    let ipc2 = rt_ipc_init("test_mutex", 2);
    
    hprintln!("IPC对象1名称: {:?}", String::from_utf8_lossy(&ipc1.name));
    hprintln!("IPC对象1类型: {}", ipc1.object_type);
    hprintln!("IPC对象2名称: {:?}", String::from_utf8_lossy(&ipc2.name));
    hprintln!("IPC对象2类型: {}", ipc2.object_type);
    
    hprintln!("IPC初始化测试完成");
}

/// 测试IPC队列操作
pub fn test_ipc_queue_operations() {
    hprintln!("测试IPC队列操作...");
    
    let ipc = rt_ipc_init("test_queue", 1);
    
    // 创建测试线程
    let thread_1 = rt_thread_create("queue_test_1", test_ipc_thread_1 as usize, 1*1024, 10, 1000);
    let thread_2 = rt_thread_create("queue_test_2", test_ipc_thread_2 as usize, 1*1024, 12, 1000);
    
    // 测试挂起线程到队列
    rt_ipc_list_suspend(ipc.clone(), thread_1.clone());
    rt_ipc_list_suspend(ipc.clone(), thread_2.clone());
    
    // 检查队列长度
    let queue_len = ipc.thread_queue.exclusive_session(|queue| queue.len());
    hprintln!("IPC队列长度: {}", queue_len);
    
    // 测试唤醒第一个线程
    if let Some(woken_thread) = rt_ipc_list_resume(ipc.clone()) {
        hprintln!("已唤醒线程: {:?}", String::from_utf8_lossy(&woken_thread.name));
    } else {
        hprintln!("队列为空，没有线程可唤醒");
    }
    
    // 再次检查队列长度
    let queue_len_after = ipc.thread_queue.exclusive_session(|queue| queue.len());
    hprintln!("唤醒后IPC队列长度: {}", queue_len_after);
    
    hprintln!("IPC队列操作测试完成");
}

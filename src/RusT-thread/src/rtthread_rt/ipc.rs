//! ipc 模块
//! 
//! 本模块实现了RT-Thread的IPC机制
//! 包括消息队列、信号量、互斥锁、事件等
//! 出于实时性考虑，这里目前只实现 PRIO 模式（优先级模式），不去实现 FIFO 模式（先进先出模式）
//! 结构体：
//!     IPCBase: 基础 IPC 结构体
//!     Semaphore: 信号量结构体
//! 函数：
//!     rt_ipc_init: 初始化 IPC 结构体
//!     rt_ipc_list_suspend: 将线程挂起，并按优先级插入线程队列
//!     rt_ipc_list_resume: 将线程唤醒
//!     rt_ipc_list_resume_all: 将所有线程唤醒
//!     rt_sem_create: 创建并初始化 semaphore 结构体
//!     rt_sem_delete: 删除 semaphore 结构体
//!     rt_sem_take: 获取 semaphore
//!     rt_sem_release: 释放 semaphore

use lazy_static::lazy_static;

extern crate alloc;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::boxed::Box;

use crate::rtthread_rt::rtdef::*;
use crate::rtthread_rt::kservice::RTIntrFreeCell;
use crate::rtthread_rt::rtconfig::*;
use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::hardware::*;
use crate::rtthread_rt::timer::*;

use core::fmt::Debug;
use alloc::sync::Arc;
use alloc::alloc::{
    alloc,
    dealloc,
    Layout,
};
use cortex_m_semihosting::hprintln;

/// 基础 IPC 结构体
pub struct IPCBase {
    /// rt_object 结构体
    pub name: [u8; RT_NAME_MAX],
    pub object_type: u8,

    /// 线程队列
    pub thread_queue: RTIntrFreeCell<Vec<Arc<RtThread>>>,
}

/// semaphore 结构体
pub struct Semaphore {
    /// 基础 IPC 结构体
    pub parent: RTIntrFreeCell<Arc<IPCBase>>,

    /// 信号量计数
    pub count: Mutex<u32>,
}

/// 初始化 IPC 结构体
/// @param name 名称
/// @param object_type 对象类型
/// @return IPC 结构体
pub fn rt_ipc_init(name: &str, object_type: u8) -> Arc<IPCBase> {
    let name_bytes = name.as_bytes();
    let len = name_bytes.len().min(RT_NAME_MAX);
    let mut name_array = [0u8; RT_NAME_MAX];
    name_array[..len].copy_from_slice(&name_bytes[..len]);
    Arc::new(IPCBase {
        name: name_array,
        object_type,
        thread_queue: unsafe { RTIntrFreeCell::new(Vec::new()) },
    })
}

/// 将线程挂起，并按优先级插入线程队列
/// @param ipc IPC 结构体
/// @param thread 线程
pub fn rt_ipc_list_suspend(ipc: Arc<IPCBase>, thread: Arc<RtThread>) {
    rt_thread_suspend(thread.clone());

    // todo 这里不知道能不能正确运行
    
    // 若队列为空，则直接插入队列
    if ipc.thread_queue.exclusive_session(|queue| queue.is_empty()) {
        // hprintln!("ipc.thread_queue is empty");
        ipc.thread_queue.exclusive_session(|queue| {
            queue.push(thread);
        });
        return;
    }
    else {
        ipc.thread_queue.exclusive_session(|queue| {
            for i in 0..queue.len() {
                // 按优先级插入队列
                if queue[i].inner.field_ptr(|thread| &thread.current_priority) > thread.inner.field_ptr(|thread| &thread.current_priority) {
                    queue.insert(i, thread.clone());
                    return;
                }
            }

            // 若优先级最低，则插入队列末尾
            queue.push(thread);
        });
    }
}

/// 将线程唤醒
/// @param ipc IPC 结构体
/// @param thread 线程
pub fn rt_ipc_list_resume(ipc: Arc<IPCBase>) -> Option<Arc<RtThread>> {
    // 取出队列第一个线程
    ipc.thread_queue.exclusive_session(|queue| {
        if queue.is_empty() {
            None
        } else {
            let thread = queue.remove(0);
            rt_thread_resume(thread.clone());
            Some(thread)
        }
    })
}

/// 将所有线程唤醒
/// @param ipc IPC 结构体
/// @param thread 线程
pub fn rt_ipc_list_resume_all(ipc: Arc<IPCBase>) {
    let level = rt_hw_interrupt_disable();
    ipc.thread_queue.exclusive_session(|queue| {
        // 唤醒所有线程
        for thread in queue.iter() {
            rt_thread_resume(thread.clone());
        }
        // 清空队列
        queue.clear();
    });
    rt_hw_interrupt_enable(level);
}

/// 创建并初始化 semaphore 结构体
/// @param name 名称
/// @param count 计数
/// @return RT_EOK: 初始化成功
///         RT_ERROR: 初始化失败
pub fn rt_sem_create(name: &str, count: u32) -> RtErrT {
    if count > 0x10000u32 {
        return RT_ERROR;
    }
    let name_bytes = name.as_bytes();
    let len = name_bytes.len().min(RT_NAME_MAX);
    let mut name_array = [0u8; RT_NAME_MAX];
    name_array[..len].copy_from_slice(&name_bytes[..len]);
    let ipc_parent = rt_ipc_init(name, 1);
    let sem = Arc::new(Semaphore {
        parent: unsafe { RTIntrFreeCell::new(ipc_parent) },
        count: Mutex::new(count),
    });
    RT_EOK
}

/// 删除 semaphore 结构体
/// @param semaphore 结构体
/// @return RT_EOK: 删除成功
pub fn rt_sem_delete(sem: Arc<Semaphore>) -> RtErrT {
    rt_ipc_list_resume_all(sem.parent.exclusive_session(|ipc| ipc.clone()));
    RT_EOK
}

/// 获取 semaphore 结构体
/// @param semaphore 结构体
/// @return RT_EOK: 获取成功
pub fn rt_sem_take(sem: Arc<Semaphore>, timeout: usize) -> RtErrT {
    let level = rt_hw_interrupt_disable();
    if *sem.count.lock() > 0 {
        *sem.count.lock() -= 1;
        rt_hw_interrupt_enable(level);
        RT_EOK
    }
    else {
        if timeout == 0 {
            rt_hw_interrupt_enable(level);
            return RT_ETIMEOUT;
        }
        else {
            let thread = rt_thread_self().unwrap();
            thread.inner.exclusive_access().error = RT_ERROR;
            rt_ipc_list_suspend(sem.parent.exclusive_session(|ipc| ipc.clone()), thread.clone());
            hprintln!("rt_sem_take: {:?}", thread);
            hprintln!("进入计时，挂起");
            if timeout > 0 {
                let mut time = timeout as u32;
                // 创建单次定时器（不是周期定时器）
                let timer = Arc::new(Mutex::new(RtTimer::new(
                    "rt_sem_take",
                    0,
                    0x0,  // 单次定时器，不是周期定时器
                    Some(Box::new(move || {
                        hprintln!("计时结束");
                    })),
                    timeout as u32,
                    timeout as u32,
                )));

                hprintln!("计时即将开始");
                // 启动定时器
                timer::rt_timer_start(timer.clone());
                hprintln!("计时开始");
            }
            rt_hw_interrupt_enable(level);
            hprintln!("rt_sem_take: {:?}", thread);
            hprintln!("正在计时");
            rt_schedule();
            if thread.inner.exclusive_access().error != RT_EOK {
                return thread.inner.exclusive_access().error;
            }
            return RT_EOK;
        }
    }
}


/// 释放 semaphore 结构体
/// @param semaphore 结构体
/// @return RT_EOK: 释放成功
pub fn rt_sem_release(sem: Arc<Semaphore>) -> RtErrT {
    let mut need_schedule = false;
    let level = rt_hw_interrupt_disable();
    // 若挂起队列非空，则需要唤醒一个线程
    if !sem.parent.exclusive_session(|ipc| ipc.thread_queue.exclusive_session(|queue| queue.is_empty())) {
        let thread = rt_ipc_list_resume(sem.parent.exclusive_session(|ipc| ipc.clone()));
        if thread.is_some() {
            need_schedule = true;
        }
    }
    else {
        if *sem.count.lock() < 0x10000u32 {
            *sem.count.lock() += 1;
        }
        else {
            rt_hw_interrupt_enable(level);
            return RT_EFULL;
        }
    }
    rt_hw_interrupt_enable(level);
    if need_schedule {
        rt_schedule();
    }
    RT_EOK
}

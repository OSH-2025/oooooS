//! ipc 模块
//! 
//! 本模块实现了RT-Thread的IPC机制
//! 包括消息队列、信号量、互斥锁、事件等
//! 出于实时性考虑，这里目前只实现 PRIO 模式（优先级模式），不去实现 FIFO 模式（先进先出模式）
//! 结构体：
//! 
//! 函数：
//! 

use lazy_static::lazy_static;

extern crate alloc;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::boxed::Box;

use crate::rtthread_rt::rtdef::*;
use crate::rtthread_rt::kservice::RTIntrFreeCell;
use crate::rtthread_rt::rtconfig::*;
use crate::rtthread_rt::thread::*;

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
    pub parent: IPCBase,

    /// 信号量计数
    pub count: u32,

    /// 保留字段，用于扩展
    pub reserved: u32,
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
    ipc.thread_queue.exclusive_session(|queue| {
        // 唤醒所有线程
        for thread in queue.iter() {
            rt_thread_resume(thread.clone());
        }
        // 清空队列
        queue.clear();
    });
}


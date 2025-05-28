use crate::rtconfig;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::vec::Vec;
use crate::rtdef::*;
use crate::kservice::RTIntrFreeCell;
use alloc::boxed::Box;
use core::fmt::Debug;
use alloc::sync::Arc;
use alloc::alloc::{
    alloc,
    dealloc,
    Layout,
};
use cortex_m_semihosting::hprintln;
pub const KERNEL_STACK_SIZE: usize = 0x400;//1kB

lazy_static! {
    /// 总的线程列表，用户可从中获取所有线程
    static ref RT_THREAD_LIST: RTIntrFreeCell<Vec<Arc<RtThread>>> = unsafe { RTIntrFreeCell::new(Vec::new()) };
}

pub struct RtThreadInner {
    /// error
    pub error: isize,
    
    /// stat
    pub stat: ThreadState,
    
    /// current priority
    pub current_priority: u8,
    
    /// number mask
    pub number_mask: u32,

    /// 线程相关信息
    pub entry: Box<dyn FnOnce() + Send + Sync + 'static>, // 函数入口

    /// tick
    pub init_tick: usize,
    pub remaining_tick: usize,

    /// context
    pub kernel_stack: KernelStack,
    pub context: Vec<RtContext>,

    /// user data
    pub user_data: usize,
}



pub struct RtThread {
    /// object
    pub name: [u8; rtconfig::RT_NAME_MAX],
    pub object_type: u8,
    
    /// inner mutable state
    pub inner: RTIntrFreeCell<RtThreadInner>,
    
    pub cleanup: Option<fn(*mut RtThread)>,
}

// 实现partial_eq
impl PartialEq for RtThread {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
        && self.object_type == other.object_type
    }
}

impl Debug for RtThread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RtThread")
            .field("name", &self.name)
            .field("object_type", &self.object_type)
            .field("inner", &"<RTIntrFreeCell<RtThreadInner>>")
            .field("cleanup", &"<function>")
            .finish()
    }
}


impl RtThread {
    //todo: 线程创建、线程添加、线程删除、线程启动、线程挂起、线程恢复、线程等待、线程唤醒、线程退出、线程调度、线程优先级、线程栈

}
/// 上下文，用于线程切换
#[derive(Debug)]
pub struct RtContext{
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl RtContext {
    pub fn new() -> Self {
        RtContext {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}


/// 创建线程
/// @param name 线程名称
/// @param entry 线程入口函数
/// @param stack_size 线程栈大小
/// @param priority 线程优先级
/// @param tick 线程时间片
/// @return 线程对象
pub fn rt_thread_create(name: [u8; rtconfig::RT_NAME_MAX], entry: Box<dyn FnOnce() + Send + Sync + 'static>, stack_size: usize, priority: u8, tick: usize) -> Arc<RtThread> {
    let thread = RtThread {
        name,
        object_type: 0,
        inner: unsafe {
            RTIntrFreeCell::new(RtThreadInner {
            error: 0,
            stat: ThreadState::Ready,
            current_priority: priority,
            number_mask: 0,
            entry,
            init_tick: tick,
            remaining_tick: tick,
            kernel_stack: KernelStack::new(stack_size),
            context: Vec::new(),
            user_data: 0,
            })
        },
        cleanup: None,
    };
    let thread_arc = Arc::new(thread);
    RT_THREAD_LIST.exclusive_access().push(thread_arc.clone());
    thread_arc
}

pub struct KernelStack {
    bottom: usize,
    size: usize,
}

impl KernelStack {
    pub fn new(size: usize) -> Self {
        // hprintln!("KernelStack::new: enter");
        let bottom = unsafe {
            alloc(Layout::from_size_align(size, size).unwrap()) as usize
        };
        // hprintln!("KernelStack::new: bottom: {}", bottom);
        KernelStack { bottom, size }
    }

    pub fn new_empty() -> Self {
        KernelStack { bottom: 0, size: 0 }
    }


    pub fn size(&self) -> usize {
        self.size
    }

    pub fn bottom(&self) -> usize {
        self.bottom
    }

    pub fn init(&self,entry: usize,parameter: usize,texit: usize) {
        unsafe {
            
        }
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        if self.bottom != 0 {
            unsafe {
                dealloc(
                    self.bottom as _,
                    Layout::from_size_align(self.size, self.size).unwrap(),
                );
            }
        }
    }
}

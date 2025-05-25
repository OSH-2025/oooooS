use crate::rtconfig;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::vec::Vec;
use crate::rtdef::*;
use crate::kservice::RTIntrFreeCell;
use alloc::boxed::Box;
use core::fmt::Debug;
use alloc::sync::Arc;

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
    pub stack_size: usize,
    pub stack_addr: usize,
    pub sp: usize,
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
            stack_size,
            stack_addr: 0,
            sp: 0,
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
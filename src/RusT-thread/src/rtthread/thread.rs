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
    pub context: RtContext,

    /// user data
    pub user_data: usize,
}



pub struct RtThread {
    /// object
    pub name: [u8; rtconfig::RT_NAME_MAX],
    pub object_type: u8,
    pub flags: u8,
    
    /// inner mutable state
    pub inner: RTIntrFreeCell<RtThreadInner>,
    
    pub cleanup: fn(*mut RtThread),
}

// 实现partial_eq
impl PartialEq for RtThread {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
        && self.flags == other.flags
    }
}

impl Debug for RtThread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RtThread")
            .field("name", &self.name)
            .field("object_type", &self.object_type)
            .field("flags", &self.flags)
            .field("inner", &"<RTIntrFreeCell<RtThreadInner>>")
            .field("cleanup", &"<function>")
            .finish()
    }
}

/// 上下文，用于线程切换
#[derive(Debug)]
pub struct RtContext{
    ra: usize,
    sp: usize,
    s: [usize; 12],
}
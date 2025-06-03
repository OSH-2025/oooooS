//! RT-Thread core definitions
//! This module contains the basic type definitions and core structures for RT-Thread

use core::ffi::c_void;
use core::ptr;
use crate::rtconfig;
extern crate alloc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use heapless::pool::object::Object;
use lazy_static::lazy_static;
use spin::Mutex;



/// Basic type definitions
#[deprecated(since = "0.1.0", note = "Use native types i8 instead")]
pub type rt_int8_t = i8;
#[deprecated(since = "0.1.0", note = "Use native types u8 instead")]
pub type rt_uint8_t = u8;
#[deprecated(since = "0.1.0", note = "Use native types i16 instead")]
pub type rt_int16_t = i16;
#[deprecated(since = "0.1.0", note = "Use native types u16 instead")]
pub type rt_uint16_t = u16;
#[deprecated(since = "0.1.0", note = "Use native types i32 instead")]
pub type rt_int32_t = i32;
#[deprecated(since = "0.1.0", note = "Use native types u32 instead")]
pub type rt_uint32_t = u32;
#[deprecated(since = "0.1.0", note = "Use native types i64 instead")]
pub type rt_int64_t = i64;
#[deprecated(since = "0.1.0", note = "Use native types u64 instead")]
pub type rt_uint64_t = u64;
#[deprecated(since = "0.1.0", note = "Use native types usize instead")]
pub type rt_size_t = usize;
#[deprecated(since = "0.1.0", note = "Use native types bool instead")]
pub type rt_bool_t = bool;
#[deprecated(since = "0.1.0", note = "Use native types isize instead")]
pub type rt_base_t = isize;
#[deprecated(since = "0.1.0", note = "Use native types usize instead")]
pub type rt_ubase_t = usize;

#[deprecated(since = "0.1.0", note = "Use UpperCamelCase RtErrT instead")]
pub type rt_err_t = isize;
#[deprecated(since = "0.1.0", note = "Use UpperCamelCase RtTimeT instead")]
pub type rt_time_t = u32;
#[deprecated(since = "0.1.0", note = "Use UpperCamelCase RtTickT instead")]
pub type rt_tick_t = u32;
#[deprecated(since = "0.1.0", note = "Use UpperCamelCase RtFlagT instead")]
pub type rt_flag_t = isize;
#[deprecated(since = "0.1.0", note = "Use UpperCamelCase RtDevT instead")]
pub type rt_dev_t = usize;
#[deprecated(since = "0.1.0", note = "Use UpperCamelCase RtOffT instead")]
pub type rt_off_t = isize;


pub type RtErrT = isize;
pub type RtTimeT = u32;
pub type RtTickT = u32;
pub type RtFlagT = isize;
pub type RtDevT = usize;
pub type RtOffT = isize;

/// Boolean type definitions
pub const RT_TRUE: bool = true;
pub const RT_FALSE: bool = false;

/// Null pointer definition
pub const RT_NULL: *mut c_void = ptr::null_mut();

/// Maximum value definitions
pub const RT_UINT8_MAX: u8 = u8::MAX;
pub const RT_UINT16_MAX: u16 = u16::MAX;
pub const RT_UINT32_MAX: u32 = u32::MAX;
pub const RT_TICK_MAX: u32 = RT_UINT32_MAX;

/// Alignment size
pub const RT_ALIGN_SIZE: u32 = rtconfig::RT_ALIGN_SIZE;

/// IPC type maximum values
pub const RT_SEM_VALUE_MAX: u16 = RT_UINT16_MAX;
pub const RT_MUTEX_VALUE_MAX: u16 = RT_UINT16_MAX;
pub const RT_MUTEX_HOLD_MAX: u8 = RT_UINT8_MAX;
pub const RT_MB_ENTRY_MAX: u16 = RT_UINT16_MAX;
pub const RT_MQ_ENTRY_MAX: u16 = RT_UINT16_MAX;

/// Thread state definitions
pub const RT_THREAD_INIT: u8 = 0x00;
pub const RT_THREAD_READY: u8 = 0x01;
pub const RT_THREAD_SUSPEND: u8 = 0x02;
pub const RT_THREAD_RUNNING: u8 = 0x03;
pub const RT_THREAD_CLOSE: u8 = 0x04;
pub const RT_THREAD_STAT_MASK: u8 = 0x07;
pub const RT_THREAD_STAT_YIELD: u8 = 0x08;
pub const RT_THREAD_STAT_YIELD_MASK: u8 = RT_THREAD_STAT_YIELD;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThreadState {
    // 基本状态 (0x00-0x07)
    Init = 0x00,    // 初始状态
    Ready = 0x01,   // 就绪状态
    Suspend = 0x02, // 挂起状态
    Running = 0x03, // 运行状态
    Close = 0x04,   // 关闭状态

    // 状态掩码
    StatMask = 0x07,

    // 让出标志 (0x08)
    Yield = 0x08,   // 表示自上次调度以来是否重新加载了remaining_tick

    // 信号相关标志 (0x10-0xf0)
    Signal = 0x10,           // 任务持有信号
    SignalReady = 0x11,      // 信号就绪状态 (Signal | Ready)
    SignalWait = 0x20,       // 任务等待信号
    SignalPending = 0x40,    // 信号已持有但未处理
    SignalMask = 0xf0,
}

impl ThreadState {
    // 获取基本状态
    pub fn get_stat(&self) -> u8 {
        let value = *self as u8;
        value & (Self::StatMask as u8)
    }

    // 检查是否包含让出标志
    pub fn has_yield(&self) -> bool {
        let value = *self as u8;
        (value & (Self::Yield as u8)) != 0
    }

    // 设置让出标志
    pub fn set_yield(&mut self) {
        let value = *self as u8;
        *self = unsafe { core::mem::transmute(value | (Self::Yield as u8)) };
    }

    // 清除让出标志
    pub fn clear_yield(&mut self) {
        let value = *self as u8;
        *self = unsafe { core::mem::transmute(value & !(Self::Yield as u8)) };
    }

    // 检查是否包含信号标志
    pub fn has_signal(&self) -> bool {
        let value = *self as u8;
        (value & (Self::SignalMask as u8)) != 0
    }

    // 获取信号相关状态
    pub fn get_signal_stat(&self) -> u8 {
        let value = *self as u8;
        value & (Self::SignalMask as u8)
    }

    // 与
    pub fn and_signal(&mut self, signal: u8) {
        let value = *self as u8;
        *self = unsafe { core::mem::transmute(value & signal) };
    }

    // 或
    pub fn or_signal(&mut self, signal: u8) {
        let value = *self as u8;
        *self = unsafe { core::mem::transmute(value | signal) };
    }
    
    

}
/// 线程控制命令
pub const RT_THREAD_CTRL_STARTUP: u8 = 0x00;
pub const RT_THREAD_CTRL_CLOSE: u8 = 0x01;
pub const RT_THREAD_CTRL_CHANGE_PRIORITY: u8 = 0x02;
pub const RT_THREAD_CTRL_INFO: u8 = 0x03;
pub const RT_THREAD_CTRL_BIND_CPU: u8 = 0x04;

/// Error code definitions
pub const RT_EOK: RtErrT = 0;
pub const RT_ERROR: RtErrT = 1;
pub const RT_ETIMEOUT: RtErrT = 2;
pub const RT_EFULL: RtErrT = 3;
pub const RT_EEMPTY: RtErrT = 4;
pub const RT_ENOMEM: RtErrT = 5;
pub const RT_ENOSYS: RtErrT = 6;
pub const RT_EBUSY: RtErrT = 7;
pub const RT_EIO: RtErrT = 8;
pub const RT_EINTR: RtErrT = 9;
pub const RT_EINVAL: RtErrT = 10;
/// Error type for RT-Thread operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtError {
    Ok,
    Error,
    Timeout,
    Full,
    Empty,
    NoMemory,
    NoSystem,
    Busy,
    IoError,
    Interrupted,
    InvalidArgument,
}


/// IPC flags
pub const RT_IPC_FLAG_FIFO: u8 = 0x00;
pub const RT_IPC_FLAG_PRIO: u8 = 0x01;

/// IPC control commands
pub const RT_IPC_CMD_UNKNOWN: u8 = 0x00;
pub const RT_IPC_CMD_RESET: u8 = 0x01;

/// Wait definitions
pub const RT_WAITING_FOREVER: i32 = -1;
pub const RT_WAITING_NO: i32 = 0;



/// Thread priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadPriority(pub u8);


lazy_static! {
    static ref RT_OBJECT_LIST: Mutex<Vec<&'static RtObject>> = Mutex::new(Vec::new());
}


#[derive(Clone)]
pub struct RtObject {
    /// 用来储存对象的名称的数组
    pub name: [u8; rtconfig::RT_NAME_MAX],
    /// 对象的类型
    pub obj_type: u8,
    /// 对象的标志状态
    pub flag: u8,
}

impl RtObject {
    /// 创建一个新的RtObject实例
    pub fn new(name: &str, obj_type: u8, flag: u8) -> &'static Self {
        let mut name_buf = [0u8; rtconfig::RT_NAME_MAX];
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(rtconfig::RT_NAME_MAX);
        name_buf[..len].copy_from_slice(&name_bytes[..len]);
        let obj = Box::leak(Box::new(Self {
            name: name_buf,
            obj_type,
            flag,
        }));
        RT_OBJECT_LIST.lock().push(obj);
        obj
    }
    
}

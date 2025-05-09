//! RT-Thread core definitions
//! This module contains the basic type definitions and core structures for RT-Thread

use core::ffi::c_void;
use core::ptr;
use crate::rtconfig;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Init,
    Ready,
    Suspend,
    Running,
    Closed,
}

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


//! RT-Thread core definitions
//! This module contains the basic type definitions and core structures for RT-Thread

use core::ffi::c_void;
use core::ptr;
use crate::rtconfig;

/// Basic type definitions
pub type rt_int8_t = i8;
pub type rt_uint8_t = u8;
pub type rt_int16_t = i16;
pub type rt_uint16_t = u16;
pub type rt_int32_t = i32;
pub type rt_uint32_t = u32;
pub type rt_int64_t = i64;
pub type rt_uint64_t = u64;
pub type rt_size_t = usize;
pub type rt_bool_t = bool;
pub type rt_base_t = isize;
pub type rt_ubase_t = usize;
pub type rt_err_t = rt_base_t;
pub type rt_time_t = rt_uint32_t;
pub type rt_tick_t = rt_uint32_t;
pub type rt_flag_t = rt_base_t;
pub type rt_dev_t = rt_ubase_t;
pub type rt_off_t = rt_base_t;

/// Boolean type definitions
pub const RT_TRUE: rt_bool_t = true;
pub const RT_FALSE: rt_bool_t = false;

/// Null pointer definition
pub const RT_NULL: *mut c_void = ptr::null_mut();

/// Maximum value definitions
pub const RT_UINT8_MAX: rt_uint8_t = u8::MAX;
pub const RT_UINT16_MAX: rt_uint16_t = u16::MAX;
pub const RT_UINT32_MAX: rt_uint32_t = u32::MAX;
pub const RT_TICK_MAX: rt_tick_t = RT_UINT32_MAX;

/// IPC type maximum values
pub const RT_SEM_VALUE_MAX: rt_uint16_t = RT_UINT16_MAX;
pub const RT_MUTEX_VALUE_MAX: rt_uint16_t = RT_UINT16_MAX;
pub const RT_MUTEX_HOLD_MAX: rt_uint8_t = RT_UINT8_MAX;
pub const RT_MB_ENTRY_MAX: rt_uint16_t = RT_UINT16_MAX;
pub const RT_MQ_ENTRY_MAX: rt_uint16_t = RT_UINT16_MAX;

/// Thread state definitions
pub const RT_THREAD_INIT: rt_uint8_t = 0x00;
pub const RT_THREAD_READY: rt_uint8_t = 0x01;
pub const RT_THREAD_SUSPEND: rt_uint8_t = 0x02;
pub const RT_THREAD_RUNNING: rt_uint8_t = 0x03;
pub const RT_THREAD_CLOSE: rt_uint8_t = 0x04;
pub const RT_THREAD_STAT_MASK: rt_uint8_t = 0x07;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Init,
    Ready,
    Suspend,
    Running,
    Closed,
}

/// Error code definitions
pub const RT_EOK: rt_err_t = 0;
pub const RT_ERROR: rt_err_t = 1;
pub const RT_ETIMEOUT: rt_err_t = 2;
pub const RT_EFULL: rt_err_t = 3;
pub const RT_EEMPTY: rt_err_t = 4;
pub const RT_ENOMEM: rt_err_t = 5;
pub const RT_ENOSYS: rt_err_t = 6;
pub const RT_EBUSY: rt_err_t = 7;
pub const RT_EIO: rt_err_t = 8;
pub const RT_EINTR: rt_err_t = 9;
pub const RT_EINVAL: rt_err_t = 10;
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
pub const RT_IPC_FLAG_FIFO: rt_uint8_t = 0x00;
pub const RT_IPC_FLAG_PRIO: rt_uint8_t = 0x01;

/// IPC control commands
pub const RT_IPC_CMD_UNKNOWN: rt_uint8_t = 0x00;
pub const RT_IPC_CMD_RESET: rt_uint8_t = 0x01;

/// Wait definitions
pub const RT_WAITING_FOREVER: rt_int32_t = -1;
pub const RT_WAITING_NO: rt_int32_t = 0;

/// Thread priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadPriority(pub u8);


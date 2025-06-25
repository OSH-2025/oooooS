// ! 本模块是RT-Thread的配置模块
// ! 包含了RT-Thread的配置信息

#![warn(unused_imports)]

/// 最大优先级
pub const RT_THREAD_PRIORITY_MAX: usize = 32;

/// 时钟频率
pub const RT_TICK_PER_SECOND: u32 = 1000;

/// 对齐大小
pub const RT_ALIGN_SIZE: u32 = 4;

/// 最大名称长度
pub const RT_NAME_MAX: usize = 8;

/// 使用组件初始化
pub const RT_USING_COMPONENTS_INIT: bool = true;

/// 使用用户主函数
pub const RT_USING_USER_MAIN: bool = true;

/// 内核栈大小
pub const KERNEL_STACK_SIZE: usize = 0x400;//1kB

/// 用户主线程优先级
pub const RT_MAIN_THREAD_PRIORITY: u32 = 0;

/// 主函数堆栈大小
pub const RT_MAIN_THREAD_STACK_SIZE: u32 = 256;

/// 调试
pub const RT_DEBUG: bool = false;
pub const RT_DEBUG_INIT: u32 = 0;
pub const RT_USING_OVERFLOW_CHECK: bool = false;
pub const RT_USING_HOOK: bool = false;
pub const RT_USING_IDLE_HOOK: bool = false;
pub const RT_USING_TIMER_SOFT: bool = false;
pub const RT_TIMER_THREAD_PRIO: u32 = 4;
pub const RT_TIMER_THREAD_STACK_SIZE: u32 = 512;
pub const RT_USING_SEMAPHORE: bool = true;
pub const RT_USING_MUTEX: bool = false;
pub const RT_USING_EVENT: bool = false;
pub const RT_USING_SIGNALS: bool = false;
pub const RT_USING_MAILBOX: bool = true;
pub const RT_USING_MESSAGEQUEUE: bool = false;
pub const RT_USING_HEAP: bool = true;
pub const RT_USING_SMALL_MEM: bool = true;
pub const RT_USING_TINY_SIZE: bool = false;
pub const RT_CONSOLEBUF_SIZE: u32 = 256;
pub const RT_USING_CONSOLE: bool = true;

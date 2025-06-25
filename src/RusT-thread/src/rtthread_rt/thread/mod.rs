// ! 线程模块
// ! 
// ! 本模块实现了RT-Thread的线程管理功能

pub mod thread;
pub mod scheduler;
pub mod idle;
pub mod thread_priority_table;
pub mod kstack;

// 重新导出所有公共项
pub use self::scheduler::
{   Scheduler, RT_SCHEDULER, 
    rt_schedule_start, 
    rt_schedule_lock, 
    rt_schedule_unlock, 
    get_current_thread, 
    rt_schedule
};

pub use self::idle::{init_idle};
pub use self::thread::{
    RtThread,
    RtThreadInner,
    RtContext,
    rt_thread_create, 
    rt_thread_startup, 
    rt_thread_delete, 
    rt_thread_self, 
    rt_thread_yield,
    rt_thread_resume,
    rt_thread_suspend,
    rt_thread_control,
    rt_thread_sleep
};
pub use self::thread_priority_table::{
    ThreadPriorityTable,
    RT_THREAD_PRIORITY_TABLE,
    insert_thread, 
    remove_thread, 
    get_highest_priority, 
    get_highest_priority_thread, 
    pop_thread};
pub use self::kstack::{KernelStack};
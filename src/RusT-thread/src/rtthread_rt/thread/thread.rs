//! 线程相关函数
//! 
//! 结构体：RtThread、RtThreadInner
//! 函数：rt_thread_create、rt_thread_self、rt_thread_delete、rt_thread_startup、rt_thread_suspend、rt_thread_sleep、rt_thread_control、rt_thread_resume、rt_thread_yield

use lazy_static::lazy_static;

extern crate alloc;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::boxed::Box;

use crate::rtthread_rt::rtdef::*;
use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::kservice::RTIntrFreeCell;
use crate::rtthread_rt::hardware::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::rtconfig::*;

use core::fmt::Debug;
use alloc::sync::Arc;
use alloc::alloc::{
    alloc,
    dealloc,
    Layout,
};
use cortex_m_semihosting::hprintln;




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

    /// high mask
    pub high_mask: u32,

    /// 线程相关信息
    pub entry: usize, // 函数入口

    /// tick
    pub init_tick: usize,
    pub remaining_tick: usize,

    /// timer
    pub timer: timer::TimerHandle,

    /// context
    pub kernel_stack: KernelStack,
    pub stack_pointer: u32,
    

    /// user data
    pub user_data: usize,
}



pub struct RtThread {
    /// object
    pub name: [u8; RT_NAME_MAX],
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
        let name_str = core::str::from_utf8(&self.name)
            .unwrap_or("invalid utf8")
            .trim_end_matches('\0');
        f.debug_struct("RtThread")
            .field("name", &name_str)
            .field("object_type", &self.object_type)
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
pub fn rt_thread_create(name: &str, entry: usize, stack_size: usize, priority: u8, tick: usize) -> Arc<RtThread> {
    // todo 健壮性检查：同名线程是否存在、栈大小是否合理、优先级是否合理、时间片是否合理

    // hprintln!("in rt_thread_create:");
    let mut kernel_stack = KernelStack::new(stack_size);
    let stack_pointer = unsafe {
        rt_hw_stack_init(
            entry,
            0 as *mut u8,
            kernel_stack.top() as usize,
            0 as usize
        )
    };
    // hprintln!("stack_pointer in rt_thread_create: {:x}", stack_pointer.clone());
    let name_bytes = name.as_bytes();
    let len = name_bytes.len().min(RT_NAME_MAX);
    let timer_callback = move || {
        hprintln!("timer_callback");
    };
    let timer = Arc::new(Mutex::new(timer::RtTimer::new(name,0,0,None,0,0)));
    // hprintln!("timer in rt_thread_create");
    let inner =unsafe {
        RTIntrFreeCell::new(RtThreadInner {
        error: 0,
        stat: ThreadState::Init,
        current_priority: priority,
        number_mask: 0,
        high_mask: 0,
        entry,
        init_tick: tick,
        remaining_tick: tick,
        kernel_stack,
        stack_pointer: stack_pointer as u32,
        user_data: 0,
        timer,
        })
    };
    let mut name = [0u8; RT_NAME_MAX];
    name[..len].copy_from_slice(&name_bytes[..len]);
    let thread = RtThread {
        name,
        object_type: 0,
        inner,
        cleanup: None,
    };
    let thread_arc = Arc::new(thread);
    timer::rt_timer_start(thread_arc.clone().inner.exclusive_access().timer.clone());
    RT_THREAD_LIST.exclusive_access().push(thread_arc.clone()); 
    // hprintln!("rt_thread_create finished.");
    thread_arc
}



//todo 是否需要完成 初始化静态线程

/// 获取当前线程
/// @return 当前线程对象
pub fn rt_thread_self() -> Option<Arc<RtThread>> {
    get_current_thread()
}


/// 删除线程
/// @param thread 线程对象
/// @return RT_EOK: 删除成功
///         : 删除失败
pub fn rt_thread_delete(thread: Arc<RtThread>) -> RtErrT {
    if thread.inner.exclusive_access().stat.get_stat() == (ThreadState::Close as u8) {
        return RT_EOK;
    }
    if thread.inner.exclusive_access().stat.get_stat() != (ThreadState::Init as u8) {
        remove_thread(thread.clone());
    }
    
    let level = rt_hw_interrupt_disable();

    thread.inner.exclusive_access().stat = ThreadState::Close; 
    rt_schedule();

    rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 线程启动
/// @param thread 线程对象
/// @return RT_EOK: 启动成功
///         RT_ERROR: 启动失败
pub fn rt_thread_startup(thread: Arc<RtThread>) -> RtErrT {
    // hprintln!("rt_thread_startup...");
    if thread.inner.exclusive_access().stat.get_stat() != (ThreadState::Init as u8) {
        return RT_ERROR;
    }

    let level = rt_hw_interrupt_disable();
    thread.inner.exclusive_access().stat = ThreadState::Suspend;
    rt_thread_resume(thread.clone()); 
    rt_hw_interrupt_enable(level);
    rt_schedule();
    RT_EOK
}

/// 线程挂起
/// 将线程从就绪或运行状态挂起，并将其从就绪队列中移除
/// 调度器检测当前状态为挂起时，不会将其插入到就绪队列中
/// @param thread 线程对象
/// @return RT_EOK: 挂起成功
///         RT_ERROR: 挂起失败
pub fn rt_thread_suspend(thread: Arc<RtThread>) -> RtErrT {
    hprintln!("rt_thread_suspend: {:?}", thread);
    let stat = thread.inner.exclusive_access().stat.get_stat();
    if (stat != (ThreadState::Ready as u8)) && (stat != (ThreadState::Running as u8)) {
        return RT_ERROR;
    }

    let level = rt_hw_interrupt_disable();
    
    thread.inner.exclusive_access().stat = ThreadState::Suspend;
    // 如果线程在就绪队列中，则将其从就绪队列中移除
    remove_thread(thread.clone());
    // 调用调度器让出CPU
    rt_schedule();

    rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 使线程进入睡眠状态
/// 让权给其他线程
/// * `thread` 线程对象
/// * `tick` 睡眠时间
/// @return RT_EOK: 睡眠成功
///         RT_ERROR: 睡眠失败
pub fn rt_thread_sleep(thread: Arc<RtThread>, tick: usize) -> RtErrT {
    // 检查线程状态：允许Ready和Running状态的线程睡眠
    let stat = thread.inner.exclusive_access().stat.get_stat();
    if (stat != (ThreadState::Ready as u8)) && (stat != (ThreadState::Running as u8)) {
        return RT_ERROR;
    }

    let level = rt_hw_interrupt_disable();
    // 设置错误状态为超时，表示线程正在等待
    thread.inner.exclusive_access().error = RT_ETIMEOUT;

    // 挂起线程
    rt_thread_suspend(thread.clone());

    // 创建睡眠定时器回调
    let thread_clone = thread.clone();
    let timer_callback = move || {
        hprintln!("timer_callback: resume thread");
        // 在定时器回调中恢复线程
        rt_thread_resume(thread_clone.clone());
    };

    // 创建单次定时器（不是周期定时器）
    let timer = Arc::new(Mutex::new(RtTimer::new(
        "rt_thread_sleep",
        0,
        0x0,  // 单次定时器，不是周期定时器
        Some(Box::new(timer_callback)),
        tick as u32,
        tick as u32,
    )));
    
    // 启动定时器
    timer::rt_timer_start(timer.clone());
    
    // 将定时器句柄保存到线程中，以便需要时可以停止
    // 注意：这里需要修改线程结构体以支持睡眠定时器
    thread.inner.exclusive_access().timer = timer;

    // 调用调度器让出CPU
    rt_schedule();

    // 恢复中断
    rt_hw_interrupt_enable(level);
    
    RT_EOK
}


/// 控制线程
/// * `thread` 线程对象
/// * `cmd` 控制命令
/// * `arg` 控制参数
/// @return RT_EOK: 控制成功
///         RT_ERROR: 控制失败
pub fn rt_thread_control(thread: Arc<RtThread>, cmd: u8, arg: u8) -> RtErrT {
    match cmd {
        RT_THREAD_CTRL_STARTUP => {
            rt_thread_startup(thread)
        },
        RT_THREAD_CTRL_CLOSE => {
            let rt_err = rt_thread_delete(thread);
            rt_schedule();
            rt_err
        }
        RT_THREAD_CTRL_CHANGE_PRIORITY => {
            let priority = arg; //todo
            let level = rt_hw_interrupt_disable();
            if thread.inner.exclusive_access().stat.get_stat() == (ThreadState::Ready as u8) {
                remove_thread(thread.clone());
                thread.inner.exclusive_access().current_priority = priority;
                if cfg!(feature = "full_ffs") {
                    let number = priority >> 3;
                    thread.inner.exclusive_access().number_mask = 1 << number;
                    thread.inner.exclusive_access().high_mask = 1 << (priority & 0x07);
                }
                else {
                    thread.inner.exclusive_access().number_mask = 1 << priority;
                }
                insert_thread(thread.clone());
            }
            else {
                thread.inner.exclusive_access().current_priority = priority;

            }
            rt_hw_interrupt_enable(level);
            RT_EOK
        }
        _ => {
            RT_ERROR
        }
    }
    
}

/// 线程恢复
/// 将线程从挂起状态恢复到就绪状态（即插入到就绪队列中）
/// @param thread 线程对象
/// @return RT_EOK: 恢复成功
///         RT_ERROR: 恢复失败
pub fn rt_thread_resume(thread: Arc<RtThread>) -> RtErrT {
    // hprintln!("rt_thread_resume...");
    if thread.inner.exclusive_access().stat.get_stat() != (ThreadState::Suspend as u8) {
        return RT_ERROR;
    }

    let level = rt_hw_interrupt_disable();

    thread.inner.exclusive_access().stat = ThreadState::Ready;
    thread_priority_table::insert_thread(thread.clone());
    rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 线程让出CPU
/// 给正在运行的线程打上让出标志，并调用调度器进行线程切换
/// @return RT_EOK: 让出成功
///         RT_ERROR: 让出失败
pub fn rt_thread_yield() -> RtErrT {
    let level = rt_hw_interrupt_disable();

    if let Some(current_thread) = scheduler::get_current_thread() {
        let mut inner = current_thread.inner.exclusive_access();
        inner.remaining_tick = inner.init_tick;
        inner.stat.set_yield();
    }
    rt_hw_interrupt_enable(level);
    scheduler::rt_schedule();
    RT_EOK
}

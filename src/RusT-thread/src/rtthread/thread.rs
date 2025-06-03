use crate::rtconfig;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::vec::Vec;
use crate::rtdef::*;
use crate::kservice::RTIntrFreeCell;
use crate::rtthread::scheduler;
use core::ffi::c_void;
// use crate::rtthread::idle;
use crate::timer;
use crate::irq;
use alloc::boxed::Box;
use core::fmt::Debug;
use alloc::sync::Arc;
use alloc::alloc::{
    alloc,
    dealloc,
    Layout,
};
use cortex_m_semihosting::hprintln;
use crate::cpuport::rt_hw_stack_init;

pub const KERNEL_STACK_SIZE: usize = 0x400;//1kB

lazy_static! {
    /// 总的线程列表，用户可从中获取所有线程
    static ref RT_THREAD_LIST: RTIntrFreeCell<Vec<Arc<RtThread>>> = unsafe { RTIntrFreeCell::new(Vec::new()) };

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

    pub fn top(&self) -> usize {
        self.bottom + self.size
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
    // pub timer: timer::TimerHandle,

    /// context
    pub kernel_stack: KernelStack,
    pub stack_pointer: usize,

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
pub fn rt_thread_create(name: [u8; rtconfig::RT_NAME_MAX], entry: usize, stack_size: usize, priority: u8, tick: usize) -> Arc<RtThread> {
    
    let mut kernel_stack = KernelStack::new(stack_size);
    let stack_pointer = unsafe {
        rt_hw_stack_init(
            entry,
            0 as *mut u8,
            kernel_stack.top() as usize,
            0 as usize
        )
    };
    let thread = RtThread {
        name,
        object_type: 0,
        inner: unsafe {
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
            stack_pointer,
            user_data: 0,
            // timer: timer::TimerHandle::new(timer::RtTimer::new(name,0,0,None,0,0,0)),
            })
        },
        cleanup: None,
    };
    let thread_arc = Arc::new(thread);
    // timer::rt_timer_start(thread_arc.clone().inner.exclusive_access().timer.clone());
    RT_THREAD_LIST.exclusive_access().push(thread_arc.clone()); 
    thread_arc
}



//todo 是否需要完成 初始化静态线程

/// 获取当前线程
/// @return 当前线程对象

pub fn rt_thread_self() -> Arc<RtThread> {
    scheduler::get_current_thread()
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
        scheduler::remove_thread(thread.clone());
    }
    
    let level = irq::rt_hw_interrupt_disable();

    thread.inner.exclusive_access().stat = ThreadState::Close; 

    irq::rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 线程启动
/// @param thread 线程对象
/// @return RT_EOK: 启动成功
///         RT_ERROR: 启动失败
pub fn rt_thread_startup(thread: Arc<RtThread>) -> RtErrT {
    if thread.inner.exclusive_access().stat.get_stat() != (ThreadState::Init as u8) {
        return RT_ERROR;
    }
    
    let level = irq::rt_hw_interrupt_disable();
    
    thread.inner.exclusive_access().stat = ThreadState::Suspend;

    // todo 恢复线程
    // rt_thread_resume(thread.clone()); 

    /*
    if rt_thread_self() != RT_NULL {
        schedule::Scheduler::schedule(); 
    }
    */

    scheduler::rt_schedule();
    irq::rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 线程挂起
/// @param thread 线程对象
/// @return RT_EOK: 挂起成功
///         RT_ERROR: 挂起失败
pub fn rt_thread_suspend(thread: Arc<RtThread>) -> RtErrT {
    let stat = thread.inner.exclusive_access().stat.get_stat();
    if (stat != (ThreadState::Ready as u8)) && (stat != (ThreadState::Running as u8)) {
        return RT_ERROR;
    }

    let level = irq::rt_hw_interrupt_disable();
    scheduler::remove_thread(thread.clone());
    thread.inner.exclusive_access().stat = ThreadState::Suspend;
    
    // timer::rt_timer_stop(&thread.inner.exclusive_access().timer);

    irq::rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 使线程进入睡眠状态
/// @param thread 线程对象
/// @param tick 睡眠时间
/// @return RT_EOK: 睡眠成功
///         RT_ERROR: 睡眠失败
pub fn rt_thread_sleep(thread: Arc<RtThread>, tick: usize) -> RtErrT {
    if thread.inner.exclusive_access().stat.get_stat() != (ThreadState::Ready as u8) {
        return RT_ERROR;
    }

    let level = irq::rt_hw_interrupt_disable();

    thread.inner.exclusive_access().error = RT_EOK;

    rt_thread_suspend(thread.clone());

    // timer::rt_timer_control(&thread.inner.exclusive_access().timer, 1, &tick as *const usize as *mut c_void);

    // timer::rt_timer_start(&thread.inner.exclusive_access().timer);

    scheduler::rt_schedule();

    if thread.inner.exclusive_access().error == RT_ETIMEOUT {
        thread.inner.exclusive_access().error = RT_EOK;
    }

    irq::rt_hw_interrupt_enable(level);
    RT_EOK
}


/// 控制线程
/// @param thread 线程对象
/// @param cmd 控制命令
/// @param arg 控制参数
/// @return RT_EOK: 控制成功
///         RT_ERROR: 控制失败
pub fn rt_thread_control(thread: Arc<RtThread>, cmd: u8, arg: u8) -> RtErrT {
    match cmd {
        RT_THREAD_CTRL_STARTUP => {
            rt_thread_startup(thread)
        },
        RT_THREAD_CTRL_CLOSE => {
            let rt_err = rt_thread_delete(thread);
            scheduler::rt_schedule();
            rt_err
        }
        RT_THREAD_CTRL_CHANGE_PRIORITY => {
            let priority = arg; //todo
            let level = irq::rt_hw_interrupt_disable();
            if thread.inner.exclusive_access().stat.get_stat() == (ThreadState::Ready as u8) {
                scheduler::remove_thread(thread.clone());
                thread.inner.exclusive_access().current_priority = priority;
                if cfg!(feature = "full_ffs") {
                    let number = priority >> 3;
                    thread.inner.exclusive_access().number_mask = 1 << number;
                    thread.inner.exclusive_access().high_mask = 1 << (priority & 0x07);
                }
                else {
                    thread.inner.exclusive_access().number_mask = 1 << priority;
                }
                scheduler::insert_thread(thread.clone());
            }
            else {
                thread.inner.exclusive_access().current_priority = priority;

            }
            irq::rt_hw_interrupt_enable(level);
            RT_EOK
        }
        _ => {
            RT_ERROR
        }
    }
    
}

pub fn rt_thread_resume(thread: Arc<RtThread>) -> RtErrT {
    if thread.inner.exclusive_access().stat.get_stat() != (ThreadState::Suspend as u8) {
        return RT_ERROR;
    }

    let level = irq::rt_hw_interrupt_disable();
    // todo RT_THREAD_LIST.remove(thread.clone());未实现
    scheduler::insert_thread(thread.clone());
    irq::rt_hw_interrupt_enable(level);
    RT_EOK
}

pub fn rt_thread_yield() -> RtErrT {
    let level = irq::rt_hw_interrupt_disable();
    let current_thread = scheduler::get_current_thread();
    current_thread.inner.exclusive_access().remaining_tick = current_thread.inner.exclusive_access().init_tick;
    current_thread.inner.exclusive_access().stat.or_signal(RT_THREAD_STAT_YIELD);
    scheduler::rt_schedule();
    irq::rt_hw_interrupt_enable(level);
    RT_EOK
}

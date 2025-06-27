//! 线程相关函数
//! 
//! 结构体：RtThread、RtThreadInner
//! 函数：rt_thread_create、rt_thread_self、rt_thread_delete、rt_thread_startup、rt_thread_suspend、rt_thread_sleep、rt_thread_control、rt_thread_resume、rt_thread_yield

use lazy_static::lazy_static;

extern crate alloc;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::boxed::Box;
use stm32f4xx_hal::pac::cryp::init;

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
    /// 当前线程的优先级
    pub current_priority: u8,
    
    /// init priority
    /// 线程的初始优先级
    /// 静态，初始化时设置，之后不再自动改变（除非用户手动调用rt_thread_set_init_priority）
    /// (如果采用MFQ调度策略，current_priority会随着线程状态变化而变化)
    pub init_priority: u8,
    
    /// number mask
    /// 这个字段用于记录线程的优先级掩码，用于快速计算线程的优先级
    /// 仅在cfg(feature = "full_ffs")时使用
    pub number_mask: u32,

    /// high mask
    /// 这个字段用于记录线程的优先级掩码，用于快速计算线程的优先级
    /// 仅在cfg(feature = "full_ffs")时使用
    pub high_mask: u32,

    /// 线程相关信息
    pub entry: usize, // 函数入口

    /// tick
    /// 线程的初始时间片
    /// 静态，初始化时设置，之后不再改变
    pub init_tick: usize,
    
    /// 线程的剩余时间片
    /// 动态，每次进入就绪队列时，剩余时间片被初始化为init_tick
    /// 每个tick会减1，当剩余时间片为0时，让出CPU
    pub remaining_tick: usize,

    /// timer
    /// 线程的睡眠定时器
    /// 当线程进入睡眠状态时，会创建一个单次定时器，当定时器到期时，会唤醒线程
    pub timer: Option<TimerHandle>,

    /// context
    /// 线程的栈
    pub kernel_stack: KernelStack,

    /// 线程的栈指针
    pub stack_pointer: u32,


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

impl RtThread {
    /// 线程名
    /// 使用方法：hprint!("{}", XX.thread_name);
    pub fn thread_name(&self) -> &str {
        let name_str = core::str::from_utf8(&self.name)
            .unwrap_or("invalid utf8")
            .trim_end_matches('\0');
        name_str
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

    // 检查线程是否存在
    // if RT_THREAD_LIST.exclusive_access().iter().any(|thread| thread.name == name) {
    //     hprintln!("Warning: thread {} already exists", name);
    // }
    // if stack_size > RT_THREAD_STACK_SIZE_MAX {
    //     hprintln!("Warning: stack_size {} is too large", stack_size);
    // }
    if priority > RT_THREAD_PRIORITY_MAX {
        hprintln!("Warning: priority {} is too large", priority);
    }
    // if tick > RT_THREAD_TICK_MAX {
    //     hprintln!("Warning: tick {} is too large", tick);
    // }

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
// hprintln!("timer in rt_thread_create");
    let inner =unsafe {
        RTIntrFreeCell::new(RtThreadInner {
        error: 0,
        stat: ThreadState::Init,
        current_priority: priority,
        init_priority: priority,
        number_mask: 0,
        high_mask: 0,
        entry,
        init_tick: tick,
        remaining_tick: tick,
        kernel_stack,
        stack_pointer: stack_pointer as u32,
        timer: None,
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
    // hprintln!("rt_thread_suspend: {:?} level: {}", thread, rt_hw_get_interrupt_level());
    let stat = thread.inner.exclusive_access().stat.get_stat();
    if (stat != (ThreadState::Ready as u8)) && (stat != (ThreadState::Running as u8)) {
        return RT_ERROR;
    }

    let level = rt_hw_interrupt_disable();
    
    thread.inner.exclusive_access().stat = ThreadState::Suspend;
    // 如果线程在就绪队列中，则将其从就绪队列中移除
    remove_thread(thread.clone());
    // hprintln!("rt_thread_suspend: remove_thread done level: {}", rt_hw_get_interrupt_level());

    rt_hw_interrupt_enable(level);
    // hprintln!("rt_thread_suspend after enable: level: {}", rt_hw_get_interrupt_level());
    // 调用调度器让出CPU
    rt_schedule();
    RT_EOK
}

/// 使线程进入睡眠状态
/// 让权给其他线程
/// * `thread` 线程对象
/// * `tick` 睡眠时间
/// @return RT_EOK: 睡眠成功
///         RT_ERROR: 睡眠失败
pub fn rt_thread_sleep(thread: Arc<RtThread>, tick: usize) -> RtErrT {
    // hprintln!("rt_thread_sleep: level: {}", rt_hw_get_interrupt_level());
    // 检查线程状态：允许Ready和Running状态的线程睡眠
    let stat = thread.inner.exclusive_access().stat.get_stat();
    if (stat != (ThreadState::Ready as u8)) && (stat != (ThreadState::Running as u8)) {
        hprintln!("rt_thread_suspend: thread not in ready or running state");
        return RT_ERROR;
    }
    // hprintln!("rt_thread_sleep after check: level: {}", rt_hw_get_interrupt_level());

    // 设置错误状态为超时，表示线程正在等待
    thread.inner.exclusive_access().error = RT_ETIMEOUT;

    // hprintln!("rt_thread_sleep after set error: level: {}", rt_hw_get_interrupt_level());

    // 创建睡眠定时器回调
    let thread_clone = thread.clone();
    let timer_callback = move || {
        hprintln!("timer_callback: resume thread");
        let timer = thread_clone.inner.exclusive_access().timer.take().unwrap();
        rt_timer_stop(&timer);
        // 清空定时器
        thread_clone.inner.exclusive_access().timer = None;
        // 清空错误状态
        thread_clone.inner.exclusive_access().error = 0;
        // 在定时器回调中恢复线程
        rt_thread_resume(thread_clone.clone());
    };

    // hprintln!("rt_thread_sleep after timer callback: level: {}", rt_hw_get_interrupt_level());
    if thread.inner.exclusive_access().timer.is_some() {
        hprintln!("Warning: rt_thread_sleep: timer already exists");
        return RT_ERROR;
    }
    // 创建单次定时器
    let timer = Arc::new(Mutex::new(RtTimer::new(
        core::str::from_utf8(&thread.name).unwrap(),
        0,
        0x0,  // 单次定时器0，不是周期定时器2
        Some(Box::new(timer_callback)),
        tick as u32,
        tick as u32,
    )));
    // 将定时器句柄保存到线程中，以便需要时可以停止
    
    thread.inner.exclusive_access().timer = Some(timer.clone());   
    // hprintln!("rt_thread_sleep after set timer: level: {}", rt_hw_get_interrupt_level());
    // 启动定时器
    timer::rt_timer_start(timer.clone());
    // hprintln!("rt_thread_sleep after timer start: level: {}", rt_hw_get_interrupt_level());
    // 挂起线程
    rt_thread_suspend(thread.clone());
    // hprintln!("rt_thread_sleep after suspend: level: {}", rt_hw_get_interrupt_level());
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
            let priority = arg; // 优先级
            rt_thread_set_priority(thread, priority);
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
    
    // // reset_priority
    let init_priority = thread.inner.exclusive_access().init_priority.clone();
    rt_thread_set_priority(thread.clone(), init_priority);

    thread.inner.exclusive_access().stat = ThreadState::Ready;
    thread_priority_table::insert_thread(thread.clone());
    rt_hw_interrupt_enable(level);
    // rt_schedule();
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

/// 线程设置优先级
/// 设置线程的优先级，并将其从就绪队列中移除再插入
/// @param thread 线程对象
/// @param priority 优先级
/// @return RT_EOK: 设置优先级成功
///         RT_ERROR: 设置优先级失败
pub fn rt_thread_set_priority(thread: Arc<RtThread>,mut priority: u8) -> RtErrT {
    // hprintln!("rt_thread_set_priority: {:?} to {}", thread, priority);
    if priority > RT_THREAD_PRIORITY_MAX - 1 {// 饱和处理
        priority = RT_THREAD_PRIORITY_MAX - 1;
    }
    let mut inner = thread.inner.exclusive_access();
    // hprintln!("rt_thread_set_priority: get inner");
    inner.current_priority = priority;
    let level = rt_hw_interrupt_disable();
    if inner.stat.get_stat() == (ThreadState::Ready as u8) {// 如果线程在就绪队列中，则将其从就绪队列中移除再插入
        remove_thread(thread.clone());
        inner.current_priority = priority;
        if cfg!(feature = "full_ffs") {
            let number = priority >> 3;
            inner.number_mask = 1 << number;
            inner.high_mask = 1 << (priority & 0x07);
        }
        else {
            inner.number_mask = 1 << priority;
        }
        insert_thread(thread.clone());
    }
    // else if inner.stat.get_stat() == (ThreadState::Running as u8) {// 如果线程在运行状态，则直接设置优先级
    //     hprintln!("rt_thread_set_priority: running");
    //     inner.current_priority = priority;
        // RT_SCHEDULER.exclusive_access().set_current_priority(priority);// ! 死锁典型范例 （这里不能调用rt_thread_set_priority，因为会导致死锁）
    //     hprintln!("rt_thread_set_priority: running done");
    // }
    else {// 如果线程不在就绪队列中，则直接设置优先级
        inner.current_priority = priority;
        if cfg!(feature = "full_ffs") {
            let number = priority >> 3;
            inner.number_mask = 1 << number;
            inner.high_mask = 1 << (priority & 0x07);
        }
        else {
            inner.number_mask = 1 << priority;
        }
    }
    // hprintln!("rt_thread_set_priority done");
    rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 线程老化
/// 线程优先级每轮+1，直到达到最大优先级（之后恢复为init_priority）
/// @param thread 线程对象
/// @return RT_EOK: 老化成功
///         RT_ERROR: 老化失败
pub fn rt_thread_aging(thread: Arc<RtThread>) -> RtErrT {
    let mut priority = thread.inner.exclusive_access().current_priority.clone();
    if priority < RT_THREAD_PRIORITY_MAX - 2 {
        priority += 1;
    }
    else {
        priority = thread.inner.exclusive_access().init_priority.clone();
    }
    let level = rt_hw_interrupt_disable();
    rt_thread_set_priority(thread.clone(), priority);
    rt_hw_interrupt_enable(level);
    RT_EOK
}

/// 线程设置初始优先级
/// 设置线程的初始优先级
/// @param thread 线程对象
/// @param priority 优先级
/// @return RT_EOK: 设置初始优先级成功
///         RT_ERROR: 设置初始优先级失败
pub fn rt_thread_set_init_priority(thread: Arc<RtThread>, priority: u8) -> RtErrT {
    let mut inner = thread.inner.exclusive_access();
    inner.init_priority = priority;
    RT_EOK
}
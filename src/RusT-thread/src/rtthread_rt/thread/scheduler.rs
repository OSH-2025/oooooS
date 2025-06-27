//! 调度器相关函数
//! 
//! 结构体：Scheduler
//! 函数：rt_schedule、rt_schedule_start、rt_schedule_lock、rt_schedule_unlock、remove_thread、insert_thread、get_current_thread、get_highest_priority、get_highest_priority_thread、pop_thread、output_priority_table

use lazy_static::lazy_static;
extern crate alloc;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use cortex_m_semihosting::{hprintln, hprint};

use crate::rtthread_rt::kservice::RTIntrFreeCell;
use crate::rtthread_rt::rtconfig::*;
use crate::rtthread_rt::rtdef::ThreadState;
use crate::rtthread_rt::hardware::*;
use crate::rtthread_rt::thread::*;


lazy_static! {
    /// 调度器
    pub static ref RT_SCHEDULER: RTIntrFreeCell<Scheduler> = unsafe { RTIntrFreeCell::new(Scheduler::new()) };
    
}

/// 调度器
pub struct Scheduler {
    /// 当前线程
    current_thread: Option<Arc<RtThread>>,
    /// 当前优先级
    current_priority: u8,
    /// 锁嵌套计数
    lock_nest: u8,
    /// 调度策略
    scheduling_policy: Box<dyn SchedulingPolicy>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            current_thread: None,
            current_priority: 0,
            lock_nest: 0,
            scheduling_policy: Box::new(PrioritySchedulingPolicy),
        }
    }
    
    /// 获取当前线程
    pub fn get_current_thread(&self) -> Option<Arc<RtThread>> {
        self.current_thread.clone()
    }

    /// 设置当前优先级
    pub fn set_current_priority(&mut self, priority: u8) {
        self.current_priority = priority;
    }

    /// 设置调度策略
    pub fn set_scheduling_policy(&mut self, policy: Box<dyn SchedulingPolicy>) {
        self.scheduling_policy = policy;
    }

    /// 获取调度策略的引用
    pub fn get_scheduling_policy(&self) -> &dyn SchedulingPolicy {
        self.scheduling_policy.as_ref()
    }
}

/// 切换到指定线程
fn switch_to_thread(thread: Arc<RtThread>) {
    let stack_pointer = thread.inner.field_mut_ptr(|thread| &mut thread.stack_pointer);
    rt_hw_context_switch_to(stack_pointer);
}

/// 从指定线程切换到指定线程
fn switch_to_thread_from_to(from_thread: Arc<RtThread>, to_thread: Arc<RtThread>) {
    let from_stack_pointer = from_thread.inner.field_mut_ptr(|thread| &mut thread.stack_pointer);
    let to_stack_pointer = to_thread.inner.field_mut_ptr(|thread| &mut thread.stack_pointer);
    rt_hw_context_switch(from_stack_pointer, to_stack_pointer);
}

/// 线程切换相关的数据结构
struct ThreadSwitchContext {
    from_thread: Option<Arc<RtThread>>,
    to_thread: Arc<RtThread>,
    /// 是否需要将原线程重新插入就绪队列：true表示需要，false表示不需要
    /// true：当原线程状态为运行或让出时，需要将原线程重新插入就绪队列
    need_insert_from_thread: bool,
}

/// 准备线程切换
fn prepare_thread_switch() -> Option<ThreadSwitchContext> {
    let mut scheduler = RT_SCHEDULER.exclusive_access();

    // 使用调度策略选择下一个线程
    let (to_thread, need_insert_from_thread) = scheduler.get_scheduling_policy()
        .select_next_thread(&scheduler.current_thread)?;
    // hprintln!("prepare_thread_switch: to_thread: {:?}", to_thread);
    // ----------------------------检查是否需要切换线程以及准备工作（更新当前线程）----------------------------
    if to_thread != scheduler.current_thread.clone().unwrap() {// 需要切换线程
        let priority_of_to_thread = to_thread.inner.exclusive_access().current_priority;
        RT_THREAD_PRIORITY_TABLE.exclusive_access().pop_thread(priority_of_to_thread);
        // 更新当前线程
        scheduler.current_priority = priority_of_to_thread;
        let from_thread = scheduler.current_thread.take();
        scheduler.current_thread = Some(to_thread.clone());

        Some(ThreadSwitchContext {
            from_thread,
            to_thread,
            need_insert_from_thread,
        })
    } else {// 不需要切换线程
        // 不需要切换线程，但需要更新状态
        to_thread.inner.exclusive_access().stat = ThreadState::Running;
        None
    }
}



/// 执行线程切换
fn execute_thread_switch(context: ThreadSwitchContext) {
    
    let ThreadSwitchContext {
        from_thread,
        to_thread,
        need_insert_from_thread,
    } = context;

    if need_insert_from_thread {
        if let Some(from) = &from_thread {
            from.inner.exclusive_access().stat = ThreadState::Ready;
            // 将原线程重新插入就绪队列
            let priority = from.inner.exclusive_access().current_priority;
            RT_THREAD_PRIORITY_TABLE.exclusive_access().push_back_to_priority(priority, from.clone());
        }
    }
    // if let Some(from) = &from_thread {
    //     hprintln!("execute_thread_switch: from_thread: {:?} to_thread: {:?}", from, to_thread);
    // }


    // 设置新线程状态为运行
    to_thread.inner.exclusive_access().stat = ThreadState::Running;

    // 执行线程切换
    if let Some(from) = from_thread {
        switch_to_thread_from_to(from, to_thread);
    } else {
        switch_to_thread(to_thread);
    }
}

/// 调度器（外部接口）
/// 
/// 主要分两步：
/// 1. 准备线程切换（prepare_thread_switch()）
/// 2. 执行线程切换（execute_thread_switch()）
pub fn rt_schedule() {
    // 关中断
    let level = rt_hw_interrupt_disable();
    // 检查锁嵌套计数
    {
        let mut scheduler = RT_SCHEDULER.exclusive_access();
        if scheduler.lock_nest > 0 {
            rt_hw_interrupt_enable(level);
            return;
        }
        // 此处scheduler的借用已经释放
    }
    // 准备线程切换
    if let Some(context) = prepare_thread_switch() {
        if rt_interrupt_get_nest() == 0 {// 非中断环境：execute_thread_switch() → 开中断 → 返回
            // 在非中断环境下切换
            execute_thread_switch(context);
            // 开中断
            rt_hw_interrupt_enable(level);
            return;
        } else {// 中断环境：execute_thread_switch() → 继续执行 → 开中断 → 返回
            // 在中断环境下切换
            execute_thread_switch(context);
        }
    }
    // 开中断
    rt_hw_interrupt_enable(level);
}

/// 启动调度器(内部函数)
pub fn start_scheduler() -> Option<Arc<RtThread>>{
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    // 获取最高优先级
    scheduler.current_priority = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_highest_priority();
    // 获取最高优先级的线程
    scheduler.current_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().pop_thread(scheduler.current_priority);
    // hprintln!("current_thread: {:?}", &scheduler.current_thread.clone().unwrap());
    if scheduler.current_thread.is_some() {
        // 设置线程状态为运行
        scheduler.current_thread.as_ref().unwrap().inner.exclusive_access().stat = ThreadState::Running;
    }
    scheduler.current_thread.clone()
}

/// 启动调度器(用户接口)
pub fn rt_schedule_start(){
    let current_thread = start_scheduler();
    if current_thread.is_some() {
        switch_to_thread(current_thread.unwrap());
    }
}

/// 锁定调度器（不允许调度）
pub fn rt_schedule_lock(){
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    scheduler.lock_nest += 1;
}

/// 解锁调度器（允许调度）
pub fn rt_schedule_unlock(){
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    scheduler.lock_nest -= 1;
}

/// 获取当前线程
pub fn get_current_thread() -> Option<Arc<RtThread>> {
    RT_SCHEDULER.exclusive_access().get_current_thread()
}



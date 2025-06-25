//! 调度器相关函数
//! 
//! 结构体：Scheduler
//! 函数：rt_schedule、rt_schedule_start、rt_schedule_lock、rt_schedule_unlock、remove_thread、insert_thread、get_current_thread、get_highest_priority、get_highest_priority_thread、pop_thread、output_priority_table

use lazy_static::lazy_static;
extern crate alloc;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
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
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            current_thread: None,
            current_priority: 0,
            lock_nest: 0,
        }
    }
    
    /// 获取当前线程
    pub fn get_current_thread(&self) -> Option<Arc<RtThread>> {
        self.current_thread.clone()
    }
}

fn switch_to_thread(thread: Arc<RtThread>) {
    let stack_pointer = thread.inner.exclusive_access().stack_pointer;
    // hprintln!("switch_to_thread: {:x}", &stack_pointer);
    rt_hw_context_switch_to(&raw const stack_pointer as *mut u32);
}

fn switch_to_thread_from_to(from_thread: Arc<RtThread>, to_thread: Arc<RtThread>) {
    let from_stack_pointer = from_thread.inner.exclusive_access().stack_pointer;
    let to_stack_pointer = to_thread.inner.exclusive_access().stack_pointer;
    // hprintln!("switch: from: {:x}, to: {:x}", &from_stack_pointer, &to_stack_pointer);
    rt_hw_context_switch(&from_stack_pointer as *const usize as *mut u32, &to_stack_pointer as *const usize as *mut u32);
}

// 线程切换相关的数据结构
struct ThreadSwitchContext {
    from_thread: Option<Arc<RtThread>>,
    to_thread: Arc<RtThread>,
    need_insert_from_thread: bool,
}

fn prepare_thread_switch() -> Option<ThreadSwitchContext> {
    // hprintln!("prepare_thread_switch");
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    // hprintln!("get scheduler");
    // 获取最高优先级
    let priority = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_highest_priority();
    // hprintln!("get highest priority");
    // 获取最高优先级的线程
    let to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().pop_thread(priority)?;
    // hprintln!("get to_thread");

    // 是否需要将原线程重新插入就绪队列
    let mut need_insert_from_thread = false;
    // hprintln!("get need_insert_from_thread");
    // 检查当前线程状态
    if let Some(current_thread) = &scheduler.current_thread {
        let current_stat = current_thread.inner.exclusive_access().stat;
        // hprintln!("get current_stat");
        // 当前线程状态为运行
        if current_stat == ThreadState::Running {
            // 获取当前线程优先级
            let current_priority = current_thread.inner.exclusive_access().current_priority;
            // 当前线程优先级小于新线程优先级
            if current_priority < priority {
                // 当前线程优先级更高，继续运行当前线程
                need_insert_from_thread = false;
            } else if current_priority == priority && !current_thread.inner.exclusive_access().stat.has_yield() {
                // 优先级相同且未让出CPU，继续运行当前线程
                need_insert_from_thread = false;
            } else {
                // 需要切换到新线程
                need_insert_from_thread = true;
            }
            // 清除让出标志
            current_thread.inner.exclusive_access().stat.clear_yield();
        }
    }
    else {
        hprintln!("Warning: current_thread is None ! ! !");
    }
    
    if to_thread != scheduler.current_thread.clone().unwrap() {
        // 需要切换线程
        scheduler.current_priority = priority;
        let from_thread = scheduler.current_thread.take();
        scheduler.current_thread = Some(to_thread.clone());

        Some(ThreadSwitchContext {
            from_thread,
            to_thread,
            need_insert_from_thread,
        })
    } else {
        // 不需要切换线程，但需要更新状态
        to_thread.inner.exclusive_access().stat = ThreadState::Running;
        None
    }
}

fn execute_thread_switch(context: ThreadSwitchContext) {
    let ThreadSwitchContext {
        from_thread,
        to_thread,
        need_insert_from_thread,
    } = context;

    if need_insert_from_thread {
        if let Some(from) = &from_thread {
            // 将原线程重新插入就绪队列
            let priority = from.inner.exclusive_access().current_priority;
            RT_THREAD_PRIORITY_TABLE.exclusive_access().push_back_to_priority(priority, from.clone());
        }
    }

    // 设置新线程状态为运行
    to_thread.inner.exclusive_access().stat = ThreadState::Running;

    // 执行线程切换
    if let Some(from) = from_thread {
        switch_to_thread_from_to(from, to_thread);
    } else {
        switch_to_thread(to_thread);
    }
}

pub fn rt_schedule() {
    // hprintln!("schedule");
    // 关中断
    let level = rt_hw_interrupt_disable();

    // 检查锁嵌套计数
    {
        // hprintln!("check lock_nest");
        let mut scheduler = RT_SCHEDULER.exclusive_access();
        if scheduler.lock_nest > 0 {
            // hprintln!("schedule: lock_nest > 0");
            rt_hw_interrupt_enable(level);
            return;
        }
        // 此处scheduler的借用已经释放
    }
    // hprintln!("lock_nest <= 0");
    // 准备线程切换
    if let Some(context) = prepare_thread_switch() {
        // hprintln!("prepare_thread_switch");

        if rt_interrupt_get_nest() == 0 {
            // 在非中断环境下切换
            // hprintln!("execute_thread_switch: level: {:x}", &level);

            execute_thread_switch(context);
            // 开中断
            rt_hw_interrupt_enable(level);
            return;
        } else {
            // 在中断环境下切换
            // hprintln!("execute_thread_switch: level: {:x}", &level);
            execute_thread_switch(context);
        }
    }

    // 开中断
    rt_hw_interrupt_enable(level);
}



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

pub fn rt_schedule_start(){
    let current_thread = start_scheduler();
    if current_thread.is_some() {
        switch_to_thread(current_thread.unwrap());
    }
}

pub fn rt_schedule_lock(){
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    scheduler.lock_nest += 1;
}

pub fn rt_schedule_unlock(){
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    scheduler.lock_nest -= 1;
}

pub fn get_current_thread() -> Arc<RtThread> {
    RT_SCHEDULER.exclusive_access().get_current_thread().unwrap()
}
//! 调度策略模块
//! 
//! 本模块实现了RT-Thread的调度策略功能
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
use crate::rtthread_rt::thread::thread_priority_table::output_priority_table;
use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::rt_tick_get;


/// 调度策略trait
pub trait SchedulingPolicy: Send + Sync {
    /// 选择下一个要运行的线程
    /// 
    /// 注意：调用该函数时，current_thread在运行中，to_thread应该在就绪队列中
    /// 调用完成后，to_thread被移除就绪队列，
    /// 
    /// # 参数
    /// * `current_thread` - 当前运行的线程
    /// 
    /// # 返回值
    /// * `Option<(Arc<RtThread>, bool)>` - (选中的线程, 是否需要将原线程重新插入就绪队列)
    fn select_next_thread(
        &self,
        current_thread: &Option<Arc<RtThread>>,
    ) -> Option<(Arc<RtThread>, bool)>;

    /// 获取策略名称
    fn get_policy_name(&self) -> &'static str;
}

/// 优先级调度策略（默认策略）
pub struct PrioritySchedulingPolicy;

impl SchedulingPolicy for PrioritySchedulingPolicy {
    fn select_next_thread(
        &self,
        current_thread: &Option<Arc<RtThread>>,
    ) -> Option<(Arc<RtThread>, bool)> {
        // hprintln!("PrioritySchedulingPolicy: at tick: {}", rt_tick_get());
        // ----------------------------获取最高优先级的线程----------------------------

        if RT_THREAD_PRIORITY_TABLE.exclusive_access().empty() {
            hprintln!("Warning: PrioritySchedulingPolicy: empty");
            return None;
        }
        // 获取最高优先级
        let priority_of_to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_highest_priority();
        let mut to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_thread(priority_of_to_thread);
        if to_thread.is_none() {
            hprintln!("PrioritySchedulingPolicy: to_thread is none");
            return None;
        }
        let mut to_thread = to_thread.unwrap();
        // hprintln!("PrioritySchedulingPolicy 0.5");
        let thread_stat = to_thread.inner.exclusive_access().stat.get_stat();
        // hprintln!("PrioritySchedulingPolicy 1");
        if thread_stat != (ThreadState::Ready as u8) {
            hprintln!("Warning: Found non-Ready thread in priority table. Thread state: {}, removing from table.", thread_stat);
            // 移除状态不正确的线程
            let _ = RT_THREAD_PRIORITY_TABLE.exclusive_access().remove_thread(to_thread.clone());
            // 重新获取下一个线程
            to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_thread(priority_of_to_thread)?;
        }
        // hprintln!("PrioritySchedulingPolicy 2");

        // 是否需要将原线程重新插入就绪队列：true表示需要，false表示不需要
        let mut need_insert_from_thread = false;

        // ----------------------------核心调度逻辑（优先级+时间片(通过yield传入)）----------------------------
        if let Some(current_thread) = current_thread {
            let current_stat = current_thread.inner.exclusive_access().stat;

            // 如果当前线程状态为运行，则需要将原线程重新插入就绪队列
            if current_stat.get_stat() == (ThreadState::Running as u8){
                need_insert_from_thread = true;
                // 获取当前线程优先级
                let current_priority = current_thread.inner.exclusive_access().current_priority;
                // 当前线程优先级小于新线程优先级
                if current_priority < priority_of_to_thread {
                    // 当前线程优先级更高，继续运行当前线程
                    to_thread = current_thread.clone();
                } else if current_priority == priority_of_to_thread && !current_thread.inner.exclusive_access().stat.has_yield() {
                    // 优先级相同且未让出CPU，继续运行当前线程
                    to_thread = current_thread.clone();
                }         
                // 清除让出标志
                current_thread.inner.exclusive_access().stat.clear_yield();
            } else {// 被挂起、删除
                need_insert_from_thread = false;
                // 不需要比较优先级，直接切换到新线程
            }
        }
        else {
            hprintln!("Warning: current_thread is None ! ! !");
        }
        // hprintln!("PrioritySchedulingPolicy 3");
        Some((to_thread, need_insert_from_thread))
    }

    fn get_policy_name(&self) -> &'static str {
        "Priority Scheduling"
    }
}


/// 多级反馈队列调度策略
pub struct MultiLevelFeedbackQueuePolicy;

impl SchedulingPolicy for MultiLevelFeedbackQueuePolicy {
    fn select_next_thread(
        &self,
        current_thread: &Option<Arc<RtThread>>,
    ) -> Option<(Arc<RtThread>, bool)> {
        // 实现多级反馈队列调度逻辑
        // 当前线程：优先级还原为初始优先级
        if let Some(current_thread) = current_thread {
            let init_priority = current_thread.inner.exclusive_access().init_priority.clone();
            rt_thread_set_priority(current_thread.clone(), init_priority);
            // hprintln!("MFQ: current_thread: {} init_priority: {}", current_thread.thread_name(), init_priority);
        }

        // 其它线程：优先级-1
        RT_THREAD_PRIORITY_TABLE.exclusive_access().batch_aging();
        // output_priority_table();
        
        // 使用优先级调度作为基础
        let priority_policy = PrioritySchedulingPolicy;
        priority_policy.select_next_thread(current_thread)
    }

    fn get_policy_name(&self) -> &'static str {
        "Multi-Level Feedback Queue"
    }
}



/// 设置调度策略为优先级调度
pub fn set_priority_scheduling() {
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    scheduler.set_scheduling_policy(Box::new(PrioritySchedulingPolicy));
}

/// 设置调度策略为多级反馈队列调度
pub fn set_mfq_scheduling() {
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    scheduler.set_scheduling_policy(Box::new(MultiLevelFeedbackQueuePolicy));
}

/// 获取当前调度策略名称
pub fn get_current_scheduling_policy_name() -> &'static str {
    let scheduler = RT_SCHEDULER.exclusive_access();
    // 这里可以通过类型检查来确定当前策略
    scheduler.get_scheduling_policy().get_policy_name()
}

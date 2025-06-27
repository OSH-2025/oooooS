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
use crate::rtthread_rt::thread::*;


/// 调度策略trait
pub trait SchedulingPolicy: Send + Sync {
    /// 选择下一个要运行的线程
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

        // ----------------------------获取最高优先级的线程----------------------------
        // 获取最高优先级
        let priority_of_to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_highest_priority();
        let mut to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_thread(priority_of_to_thread)?;
        let thread_stat = to_thread.inner.exclusive_access().stat.get_stat();
        if thread_stat != (ThreadState::Ready as u8) {
            hprintln!("Warning: Found non-Ready thread in priority table. Thread state: {}, removing from table.", thread_stat);
            // 移除状态不正确的线程
            RT_THREAD_PRIORITY_TABLE.exclusive_access().remove_thread(to_thread.clone());
            // 重新获取下一个线程
            to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_thread(priority_of_to_thread)?;
        }
        
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
        // 这里可以调用Aging()函数来实现优先级调整
        
        // 暂时使用优先级调度作为基础
        let priority_policy = PrioritySchedulingPolicy;
        priority_policy.select_next_thread(current_thread)
    }

    fn get_policy_name(&self) -> &'static str {
        "Multi-Level Feedback Queue"
    }
}

/// 老化算法
/// 
/// 如果启用MFQ（多级反馈队列），则需要使用老化算法
/// 
/// 常见的实现是：
/// 
/// 新创建的线程从最高优先级开始
/// 如果线程用完时间片，降低其优先级
/// 如果线程主动让出CPU，保持当前优先级
/// 低优先级线程会逐渐提升优先级，防止饥饿
/// 
/// 故在每次线程切换时，调用以下函数，实现优先级的动态调整
fn Aging(){
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

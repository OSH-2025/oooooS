use crate::rtthread::thread::RtThread;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::kservice::RTIntrFreeCell;
use crate::rtconfig;
use alloc::vec::Vec;
use crate::rtdef::ThreadState;

// 静态变量：一个单例
lazy_static! {
    /// 调度器
    static ref RT_SCHEDULER: RTIntrFreeCell<Scheduler> = unsafe { RTIntrFreeCell::new(Scheduler::new()) };
    /// 就绪优先级表
    static ref RT_THREAD_PRIORITY_TABLE: RTIntrFreeCell<ThreadPriorityTable> = unsafe { 
        RTIntrFreeCell::new(ThreadPriorityTable::new()) 
    };
}

/// 线程优先级表
struct ThreadPriorityTable {
    table: [VecDeque<Arc<RtThread>>; rtconfig::RT_THREAD_PRIORITY_MAX],
    #[cfg(feature = "full_ffs")]
    ready_table: [u8; 32],
    ready_priority_group: u32,
}

impl ThreadPriorityTable {
    /// 创建一个线程优先级表
    fn new() -> Self {
        let mut table = Vec::with_capacity(rtconfig::RT_THREAD_PRIORITY_MAX);
        for _ in 0..rtconfig::RT_THREAD_PRIORITY_MAX {
            table.push(VecDeque::new());
        }
        Self {
            table: table.try_into().unwrap(),
            #[cfg(feature = "full_ffs")]
            ready_table: [0; 32],
            ready_priority_group: 0,
        }
    }
    /// 获取优先级表中优先级为priority的线程
    pub fn get_thread(&self, priority: u8) -> Option<Arc<RtThread>> {
        self.table[priority as usize].front().cloned()
    }
    /// 获取优先级表中thread的索引
    pub fn get_thread_index(&self, thread: Arc<RtThread>) -> Option<usize> {
        self.table.iter().enumerate().find_map(|(i, queue)| {
            if queue.contains(&thread) {
                Some(i)
            } else {
                None
            }
        })
    }
    /// 从优先级表中移除优先级为priority的线程
    pub fn pop_thread(&mut self, priority: u8) -> Option<Arc<RtThread>> {
        // 若优先级表为空，则返回None
        if self.table[priority as usize].is_empty() {
            return None;
        }
        // 若优先级表只剩一个线程，需更新就绪优先级组
        if self.table[priority as usize].len() == 1 {
            self.tag_off_priority(priority);
        }
        self.table[priority as usize].pop_front()
    }

    pub fn remove_thread(&mut self, priority: u8, index: usize) {
        self.table[priority as usize].remove(index);
        // 若优先级表为空，需更新就绪优先级组
        if self.table[priority as usize].is_empty() {
            self.tag_off_priority(priority);
        }
    }

    #[cfg(feature = "full_ffs")]
    pub fn get_highest_priority(&self) -> u8 {
        let number = __rt_ffs(self.ready_priority_group) - 1;
        (number << 3) + __rt_ffs(self.ready_table[number as usize])
    }

    #[cfg(feature = "tiny_ffs")]
    pub fn get_highest_priority(&self) -> u8 {
        __rt_ffs(self.ready_priority_group) - 1
    }
    /// 去除priority在ready_priority_group中的标记
    #[cfg(feature = "full_ffs")]
    fn tag_off_priority(&self, priority: u8) {
        todo!();
    }
    /// 设置priority在ready_priority_group中的标记
    #[cfg(feature = "full_ffs")]
    fn tag_on_priority(&self, priority: u8) {
        todo!();
    }

    /// 去除priority在ready_priority_group中的标记
    #[cfg(feature = "tiny_ffs")]
    fn tag_off_priority(&self, priority: u8) {
        todo!();
    }
    /// 设置priority在ready_priority_group中的标记
    #[cfg(feature = "tiny_ffs")]
    fn tag_on_priority(&self, priority: u8) {
        todo!();
    }
}

/// 调度器
struct Scheduler {
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
    /// 切换到线程
    fn switch_to_thread(&mut self, thread: Option<Arc<RtThread>>) {
        todo!()
    }
   

    pub fn start(&mut self) {

        // 获取最高优先级
        self.current_priority = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_highest_priority();
        // 获取最高优先级的线程
        self.current_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().pop_thread(self.current_priority);
        
        if self.current_thread.is_some() {
            // 设置线程状态为运行
            self.current_thread.as_ref().unwrap().inner.exclusive_access().stat = ThreadState::Running;
        }

        // 切换到最高优先级的线程
        self.switch_to_thread(self.current_thread.clone());
    }
    /// 调度
    pub fn schedule(&mut self) {
        // 关中断
        todo!("关中断");

        // 检查锁嵌套计数
        if self.lock_nest > 0 {
            return;
        }
        



        // 开中断
        todo!("开中断");
    }

    pub fn lock(&mut self) {
        self.lock_nest += 1;
    }

    pub fn unlock(&mut self) {
        self.lock_nest -= 1;
    }
}

/// ffs
///
/// 查找最低位设置的实现

#[cfg(feature = "tiny_ffs")]
const __LOWEST_BIT_BITMAP: [u8; 37] = [
    /*  0 - 7  */  0,  1,  2, 27,  3, 24, 28, 32,
    /*  8 - 15 */  4, 17, 25, 31, 29, 12, 32, 14,
    /* 16 - 23 */  5,  8, 18, 32, 26, 23, 32, 16,
    /* 24 - 31 */ 30, 11, 13,  7, 32, 22, 15, 10,
    /* 32 - 36 */  6, 21,  9, 20, 19
];

/**
 * This function finds the first bit set (beginning with the least significant bit)
 * in value and return the index of that bit.
 *
 * Bits are numbered starting at 1 (the least significant bit).  A return value of
 * zero from any of these functions means that the argument was zero.
 *
 * @return return the index of the first bit set. If value is 0, then this function
 * shall return 0.
 */
#[cfg(feature = "tiny_ffs")]
/// 查找最低位设置的实现
pub fn __rt_ffs(value: u32) -> u8 {
    return __LOWEST_BIT_BITMAP[((value & (value - 1) ^ value) % 37) as usize];
}

#[cfg(feature = "full_ffs")]
const __LOWEST_BIT_BITMAP: [u8; 256] = [
    /* 00 */ 0, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 10 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 20 */ 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 30 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 40 */ 6, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 50 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 60 */ 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 70 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 80 */ 7, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* 90 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* A0 */ 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* B0 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* C0 */ 6, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* D0 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* E0 */ 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0,
    /* F0 */ 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
];

/**
 * This function finds the first bit set (beginning with the least significant bit)
 * in value and return the index of that bit.
 *
 * Bits are numbered starting at 1 (the least significant bit).  A return value of
 * zero from any of these functions means that the argument was zero.
 *
 * @return Return the index of the first bit set. If value is 0, then this function
 *         shall return 0.
 */
#[cfg(feature = "full_ffs")]
pub fn __rt_ffs(value: u32) -> u8 {
    if value == 0 {
        return 0;
    }

    if (value & 0xff) != 0 {
        return __LOWEST_BIT_BITMAP[value & 0xff] + 1;
    }

    if (value & 0xff00) != 0 {
        return __LOWEST_BIT_BITMAP[(value & 0xff00) >> 8] + 9;
    }

    if (value & 0xff0000) != 0 {
        return __LOWEST_BIT_BITMAP[(value & 0xff0000) >> 16] + 17;
    }

    return __LOWEST_BIT_BITMAP[(value & 0xff000000) >> 24] + 25;
}


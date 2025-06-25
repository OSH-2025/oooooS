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
    /// 就绪优先级表
    pub static ref RT_THREAD_PRIORITY_TABLE: RTIntrFreeCell<ThreadPriorityTable> = unsafe { 
        RTIntrFreeCell::new(ThreadPriorityTable::new()) 
    };
}

/// 线程优先级表
pub struct ThreadPriorityTable {
    table: [VecDeque<Arc<RtThread>>; RT_THREAD_PRIORITY_MAX],
    #[cfg(feature = "full_ffs")]
    ready_table: [u8; 32],
    ready_priority_group: u32,
}

impl ThreadPriorityTable {
    /// 创建一个线程优先级表
    fn new() -> Self {
        let mut table = Vec::with_capacity(RT_THREAD_PRIORITY_MAX);
        for _ in 0..RT_THREAD_PRIORITY_MAX {
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
    /// 即在当前优先级的队列的第几个
    pub fn get_thread_index(&self, thread: Arc<RtThread>) -> Option<usize> {
        // 获取线程的优先级，这样可以只检查对应优先级的队列
        let priority = thread.inner.exclusive_access().current_priority;
        self.table[priority as usize]
            .iter()
            .position(|t| t == &thread)
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

    pub fn remove_thread_by_id(&mut self, priority: u8, index: usize) {
        // hprintln!("remove_thread_by_id: priority: {}, index: {}", &priority, &index);
        self.table[priority as usize].remove(index);
        // 若优先级表为空，需更新就绪优先级组
        if self.table[priority as usize].is_empty() {
            self.tag_off_priority(priority);
        }
    }

    pub fn insert_thread(&mut self, thread: Arc<RtThread>) {
        // hprintln!("insert_thread: ");
        let priority = thread.inner.exclusive_access().current_priority;
        // hprintln!("insert_thread 1");
        self.table[priority as usize].push_back(thread.clone());
        // hprintln!("insert_thread 2");
        self.tag_on_priority(priority);
        // hprintln!("insert_thread 3");
        // hprintln!("insert_thread: priority: {}, thread: {:?}", &priority, &thread);
    }

    pub fn remove_thread(&mut self,thread: Arc<RtThread>) {
        let priority = thread.inner.exclusive_access().current_priority;
        if let Some(index) = self.get_thread_index(thread.clone()) {
            self.remove_thread_by_id(priority, index);
        }
    }
    
    /// 获取就绪优先级组
    pub fn get_ready_priority_group(&self) -> u32 {
        self.ready_priority_group
    }
    
    /// 获取指定优先级的线程队列
    pub fn get_priority_queue(&self, priority: u8) -> &VecDeque<Arc<RtThread>> {
        &self.table[priority as usize]
    }
    
    /// 将线程添加到指定优先级的队列末尾
    pub fn push_back_to_priority(&mut self, priority: u8, thread: Arc<RtThread>) {
        self.table[priority as usize].push_back(thread);
        self.tag_on_priority(priority);
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
    fn tag_off_priority(&mut self, priority: u8) {
        let group = priority >> 3;
        let bit = priority & 0x07;
        self.ready_table[group as usize] &= !(1 << bit);
        if self.ready_table[group as usize] == 0 {
            self.ready_priority_group &= !(1 << group);
        }
    }
    /// 设置priority在ready_priority_group中的标记
    #[cfg(feature = "full_ffs")]
    fn tag_on_priority(&mut self, priority: u8) {
        let group = priority >> 3;
        let bit = priority & 0x07;
        self.ready_table[group as usize] |= 1 << bit;
        self.ready_priority_group |= 1 << group;
    }

    /// 去除priority在ready_priority_group中的标记
    #[cfg(feature = "tiny_ffs")]
    fn tag_off_priority(&mut self, priority: u8) {
        self.ready_priority_group &= !(1 << priority);
    }
    /// 设置priority在ready_priority_group中的标记
    #[cfg(feature = "tiny_ffs")]
    fn tag_on_priority(&mut self, priority: u8) {
        self.ready_priority_group |= 1 << priority;
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


pub fn remove_thread(thread: Arc<RtThread>) {
    RT_THREAD_PRIORITY_TABLE.exclusive_access().remove_thread(thread);
}

pub fn insert_thread(thread: Arc<RtThread>) {
    // hprintln!("insert_thread: {:?}", &thread);
    RT_THREAD_PRIORITY_TABLE.exclusive_access().insert_thread(thread);
}


pub fn get_highest_priority() -> u8 {
    RT_THREAD_PRIORITY_TABLE.exclusive_access().get_highest_priority()
}

pub fn get_highest_priority_thread() -> Arc<RtThread> {
    let priority = get_highest_priority();
    RT_THREAD_PRIORITY_TABLE.exclusive_access().get_thread(priority).unwrap()
}

pub fn pop_thread(priority: u8) -> Option<Arc<RtThread>> {
    RT_THREAD_PRIORITY_TABLE.exclusive_access().pop_thread(priority)
}

#[cfg(feature = "test")]
pub fn output_priority_table(){
    let priority_table = RT_THREAD_PRIORITY_TABLE.exclusive_access();
    hprintln!("bitmap:");
    hprintln!("{:08b}\n",&priority_table.ready_priority_group);


    hprintln!("priority table:");

    for i in 0..RT_THREAD_PRIORITY_MAX {
        hprintln!("priority: {}", i);
        for thread in priority_table.table[i].iter() {
            hprintln!("thread: {:?}", thread);
        }
    }
}



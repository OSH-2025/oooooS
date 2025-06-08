use crate::rtthread::thread::RtThread;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::kservice::RTIntrFreeCell;
use crate::rtconfig;
use alloc::vec::Vec;
use crate::rtdef::ThreadState;
use crate::irq;
use crate::context::{rt_hw_context_switch_to, rt_hw_context_switch};
use cortex_m_semihosting::{hprintln, hprint};

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
        hprintln!("remove_thread_by_id: priority: {}, index: {}", &priority, &index);
        self.table[priority as usize].remove(index);
        // 若优先级表为空，需更新就绪优先级组
        if self.table[priority as usize].is_empty() {
            self.tag_off_priority(priority);
        }
    }

    pub fn insert_thread(&mut self, thread: Arc<RtThread>) {
        let priority = thread.inner.exclusive_access().current_priority;
        self.table[priority as usize].push_back(thread.clone());
        self.tag_on_priority(priority);
    }

    pub fn remove_thread(&mut self,thread: Arc<RtThread>) {
        let priority = thread.inner.exclusive_access().current_priority;
        if let Some(index) = self.get_thread_index(thread.clone()) {
            self.remove_thread_by_id(priority, index);
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
}

fn switch_to_thread(thread: Arc<RtThread>) {
    let stack_pointer = thread.inner.exclusive_access().stack_pointer;
    hprintln!("switch_to_thread: {:x}", &stack_pointer);
    rt_hw_context_switch_to(&raw const stack_pointer as *mut u32);
}

fn switch_to_thread_from_to(from_thread: Arc<RtThread>, to_thread: Arc<RtThread>) {
    let from_stack_pointer = from_thread.inner.exclusive_access().stack_pointer;
    let to_stack_pointer = to_thread.inner.exclusive_access().stack_pointer;
    hprintln!("switch: from: {:x}, to: {:x}", &from_stack_pointer, &to_stack_pointer);
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
    // 获取最高优先级的线程
    let to_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().pop_thread(priority)?;

    // 是否需要将原线程重新插入就绪队列
    let mut need_insert_from_thread = false;

    // 检查当前线程状态
    if let Some(current_thread) = &scheduler.current_thread {
        let current_stat = current_thread.inner.exclusive_access().stat;
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
            RT_THREAD_PRIORITY_TABLE.exclusive_access().table[from.inner.exclusive_access().current_priority as usize]
                .push_back(from.clone());
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
    hprintln!("schedule");
    // 关中断
    let level = irq::rt_hw_interrupt_disable();

    // 检查锁嵌套计数
    {
        // hprintln!("check lock_nest");
        let mut scheduler = RT_SCHEDULER.exclusive_access();
        if scheduler.lock_nest > 0 {
            hprintln!("schedule: lock_nest > 0");
            irq::rt_hw_interrupt_enable(level);
            return;
        }
        // 此处scheduler的借用已经释放
    }
    // hprintln!("lock_nest <= 0");
    // 准备线程切换
    if let Some(context) = prepare_thread_switch() {
        // 此时scheduler的借用已经释放
        if irq::rt_interrupt_get_nest() == 0 {
            // 在非中断环境下切换
            execute_thread_switch(context);
            // 开中断
            irq::rt_hw_interrupt_enable(level);
            return;
        } else {
            // 在中断环境下切换
            execute_thread_switch(context);
        }
    }

    // 开中断
    irq::rt_hw_interrupt_enable(level);
}



pub fn start_scheduler() -> Option<Arc<RtThread>>{
    let mut scheduler = RT_SCHEDULER.exclusive_access();
    // 获取最高优先级
    scheduler.current_priority = RT_THREAD_PRIORITY_TABLE.exclusive_access().get_highest_priority();
    // 获取最高优先级的线程
    scheduler.current_thread = RT_THREAD_PRIORITY_TABLE.exclusive_access().pop_thread(scheduler.current_priority);
    hprintln!("current_thread: {:?}", &scheduler.current_thread.clone().unwrap());
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
    RT_THREAD_PRIORITY_TABLE.exclusive_access().insert_thread(thread);
}

pub fn get_current_thread() -> Arc<RtThread> {
    RT_SCHEDULER.exclusive_access().current_thread.clone().unwrap()
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

    for i in 0..rtconfig::RT_THREAD_PRIORITY_MAX {
        hprintln!("priority: {}", i);
        for thread in priority_table.table[i].iter() {
            hprintln!("thread: {:?}", thread);
        }
    }
}



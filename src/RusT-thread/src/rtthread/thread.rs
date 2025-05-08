use crate::rtconfig;

pub struct RtThread {
    /// object
    pub name: [u8; rtconfig::RT_NAME_MAX as usize],
    pub object_type: u8,
    pub flags: u8,

    /// list 
    // todo: 需要一个链表，来装载线程
    /**
     * pub thread_list: rt_list,
     */

    /// stack
    /// todo: 需要一个栈，来装载线程
    /**
     * pub thread_stack: rt_stack,
     */
    
    /// error
    pub error: isize,
    
    /// stat
    pub stat: u8,
    
    /// current priority
    pub current_priority: u8,
    
    /// number mask
    pub number_mask: u32,
    
    /// tick
    pub init_tick: usize,
    pub remaining_tick: usize,

    /// timer
    /// todo: 需要一个定时器
    /**
     * pub thread_timer: rt_timer,
     */
    
    pub cleanup: fn(*mut RtThread),
    
    /// user data
    pub user_data: usize,
}
    


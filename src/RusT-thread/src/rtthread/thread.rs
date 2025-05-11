use crate::rtconfig;


pub struct RtThread {
    /// object
    pub name: [u8; rtconfig::RT_NAME_MAX],
    pub object_type: u8,
    pub flags: u8,
    
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
    

impl RtThread {
    /// 创建一个新线程
    pub fn new() -> Self {
        todo!();
    }

    /// 初始化线程
    pub fn init(&mut self) {
        todo!()
    }

    /// 启动线程
    pub fn startup(&mut self) {
        todo!()
    }

    /// 分离线程
    pub fn detach(&mut self) {
        todo!()
    }

    /// 让出线程
    pub fn yield_thread(&mut self) {
        todo!()
    }

    /// 等待线程
    pub fn wait(&mut self) {
        todo!()
    }

    /// 睡眠线程
    pub fn sleep(&mut self) {
        todo!()
    }

    /// 延迟线程
    pub fn delay(&mut self, tick: usize) {
        todo!("{}", tick)
    }

    /// 延迟线程直到
    pub fn delay_until(&mut self, tick: usize) {
        todo!("{}", tick)
    }

    /// 延迟线程毫秒
    pub fn mdelay(&mut self, ms: usize) {
        todo!("{}", ms)
    }

    /// 延迟线程毫秒直到
    pub fn mdelay_until(&mut self, ms: usize, tick: usize) {
        todo!("{}, {}", ms, tick)
    }

    /// 挂起线程
    pub fn suspend(&mut self) {
        todo!()
    }

    /// 恢复线程
    pub fn resume(&mut self) {
        todo!()
    }

    /// 查找线程, 该函数在Object中实现
    pub fn find_thread_by_name(&mut self, name: &str) -> *mut RtThread {
        todo!("{}", name)
    }

}


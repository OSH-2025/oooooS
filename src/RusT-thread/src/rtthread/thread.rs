use crate::rtconfig;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::vec::Vec;
use crate::rtdef::*;
use crate::kservice::RTIntrFreeCell;
use alloc::boxed::Box;

lazy_static! {
    static ref RT_THREAD_LIST: RTIntrFreeCell<Vec<&'static RtThread>> = unsafe { RTIntrFreeCell::new(Vec::new()) };
}
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

    /// 线程相关信息
    // pub sp: Weak<RefCell<u32>>, // 栈指针
    pub entry: Box<dyn FnOnce() + Send + Sync + 'static>, // 函数入口
    // pub parameter: Weak<RefCell<u32>>, // 参数
    // pub stack_addr: Weak<RefCell<u32>>, // 栈地址
    // pub stack_size: Weak<RefCell<u32>>, // 栈大小
    
    
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
        Self {
            name: [0; rtconfig::RT_NAME_MAX],
            object_type: 0,
            flags: 0,
            error: 0,
            stat: RT_THREAD_INIT,
            current_priority: 0,
            number_mask: 0,
            entry: Box::new(|| {}),
            init_tick: 0,
            remaining_tick: 0,
            cleanup: |_| {},
            user_data: 0,
        }

    }

    /// 初始化线程
    pub fn init(&mut self, name: &str, priority: u8, tick: usize) {
        // 设置线程名称
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(rtconfig::RT_NAME_MAX);
        self.name[..len].copy_from_slice(&name_bytes[..len]);
        
        // 设置优先级
        self.current_priority = priority;
        self.number_mask = 1 << priority;
        
        // 设置时间片
        self.init_tick = tick;
        self.remaining_tick = tick;
        
        // 设置状态为初始化
        self.stat = RT_THREAD_INIT;
    }

    /// 启动线程
    pub fn startup(&mut self) {
        // 检查线程状态
        if (self.stat & RT_THREAD_STAT_MASK) != RT_THREAD_INIT {
            return;
        }
        
        // 设置状态为就绪
        self.stat = RT_THREAD_READY;
        
        // TODO: 将线程插入调度器就绪队列
    }

    /// 分离线程
    pub fn detach(&mut self) {
        // 检查线程状态
        if (self.stat & RT_THREAD_STAT_MASK) == RT_THREAD_CLOSE {
            return;
        }
        
        // 从调度器中移除
        // TODO: 调用调度器remove_thread
        
        // 设置状态为关闭
        self.stat = RT_THREAD_CLOSE;
    }

    /// 让出线程
    pub fn yield_thread(&mut self) {
        // 设置剩余时间片
        self.remaining_tick = self.init_tick;
        
        // 设置让出标志
        self.stat |= 0x02; // RT_THREAD_STAT_YIELD
        
        // TODO: 调用调度器schedule
    }

    /// 等待线程
    pub fn wait(&mut self) {
        // TODO: 实现等待逻辑
    }

    /// 睡眠线程
    pub fn sleep(&mut self) {
        // 设置状态为挂起
        self.stat = RT_THREAD_SUSPEND;
        
        // TODO: 调用调度器schedule
    }

    /// 延迟线程
    pub fn delay(&mut self, tick: usize) {
        // 设置剩余时间片
        self.remaining_tick = tick;
        
        // 调用睡眠
        self.sleep();
    }

    /// 延迟线程直到
    pub fn delay_until(&mut self, tick: usize) {
        // TODO: 实现延迟直到指定时间
    }

    /// 延迟线程毫秒
    pub fn mdelay(&mut self, ms: usize) {
        // TODO: 将毫秒转换为tick
        let tick = ms;
        self.delay(tick);
    }

    /// 延迟线程毫秒直到
    pub fn mdelay_until(&mut self, ms: usize, tick: usize) {
        // TODO: 实现毫秒延迟直到指定时间
    }

    /// 挂起线程
    pub fn suspend(&mut self) {
        // 检查线程状态
        if (self.stat & RT_THREAD_STAT_MASK) != RT_THREAD_READY && 
           (self.stat & RT_THREAD_STAT_MASK) != RT_THREAD_RUNNING {
            return;
        }
        
        // 设置状态为挂起
        self.stat = RT_THREAD_SUSPEND;
        
        // TODO: 从调度器中移除
    }

    /// 恢复线程
    pub fn resume(&mut self) {
        // 检查线程状态
        if (self.stat & RT_THREAD_STAT_MASK) != RT_THREAD_SUSPEND {
            return;
        }
        
        // 设置状态为就绪
        self.stat = RT_THREAD_READY;
        
        // TODO: 将线程插入调度器就绪队列
    }

    /// 查找线程, 该函数在Object中实现
    pub fn find_thread_by_name(&mut self, name: &str) -> Option<*mut RtThread> {
        // TODO: 实现线程查找
        None
    }
}


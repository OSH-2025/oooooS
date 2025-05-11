use crate::rtthread::thread::RtThread;
use lazy_static::lazy_static;
use spin::Mutex;

// 静态变量：一个单例
lazy_static! {
    static ref RT_SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);
}

/// 调度器
struct Scheduler {
    
}

impl Scheduler {
    /// 创建一个新调度器
    pub fn new() -> Self {
        todo!()
    }

    /// 初始化调度器
    pub fn init(&mut self) {
        todo!()
    }

    /// 启动调度器
    pub fn start(&mut self) {
        todo!()
    }

    /// 调度器调度
    pub fn schedule(&mut self) {
        todo!()
    }
    
    /// 获取当前线程
    pub fn get_current_thread(&self) -> *mut RtThread {
        todo!()
    }

    /// 插入线程
    pub fn insert_thread(&mut self, thread: *mut RtThread) {
        todo!()
    }

    /// 移除线程
    pub fn remove_thread(&mut self, thread: *mut RtThread) {
        todo!()
    }

    /// 进入临界区
    pub fn enter_critical(&mut self) {
        todo!()
    }

    /// 退出临界区
    pub fn exit_critical(&mut self) {
        todo!()
    }

    /// 获取最高优先级线程
    fn _scheduler_get_highest_priority_thread(priority: usize) -> *mut RtThread {
        todo!()
    }
}






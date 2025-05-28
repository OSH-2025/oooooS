extern crate alloc;
use crate redef::RtObject;
use core::ffi::c_void;
use alloc::vec::Vec;
use alloc::boxed::Box;

//由于rust中可以使用高级容器：动态数组，所以不需要使用链表（跳表）算法来加速
//定时器的查找，而可以使用二分查找，所以我们决定把定时器存放在动态数组中

//定时器结构体
//注意回调函数应是一个函数或闭包，且其类型满足FnMut(&mut T) + Send + Sync + 'static,
// 定义一个通用的定时器行为 trait
pub struct RtTimer {
    pub parent: RtObject,
    pub timeout_callback: Option<Box<dyn FnMut(*mut ()) + Send + Sync + 'static>>,
    pub user_data: *mut c_void,
    pub init_tick: u32,
    pub timeout_tick: u32,
}
impl RtTimer {
    /// 创建一个新的 RtTimer 实例
    pub fn new(
        name: &str,
        obj_type: u8,
        flag: u8,
        timeout_func: Option<Box<dyn FnMut(*mut ()) + Send + Sync + 'static>>,
        parameter: *mut c_void,
        init_tick: u32,
        timeout_tick: u32,
    ) -> Self {
        let parent_object = RtObject::new(name, obj_type, flag);
        Self {
            parent: parent_object,
            timeout_callback: timeout_func,
            user_data: parameter,
            init_tick,
            timeout_tick,
        }
    }

    /// 设置定时器回调函数和用户数据
    pub fn set_timeout_callback<F, T: 'static>(&mut self, callback: F, data: &mut T)
    where
        F: FnMut(*mut T) + Send + Sync + 'static,
    {
        self.user_data = data as *mut T as *mut c_void;
        self.timeout_callback = Some(Box::new(move |raw_ptr| {
            let typed_ptr = raw_ptr as *mut T;
            if !typed_ptr.is_null() {
                callback(typed_ptr);
            }
        }));
    }

    /// 触发定时器回调函数
    pub fn trigger_timeout(&mut self) {
        if let Some(callback) = &self.timeout_callback {
            let user_data_ptr = self.user_data;
            callback(user_data_ptr);
        }
    }

    /// 销毁当前的 RtTimer 实例
    pub fn destroy(self) {
        // println!("Timer for object '{}' destroyed (c_void version).", String::from_utf8_lossy(&self.parent.name));
        // Rust 会自动 drop self，包括 parent, timeout_callback 和 user_data。
        // user_data 是裸指针，Rust 不会对其指向的内存做任何操作。
        // 如果 user_data 指向的内存需要手动释放，你需要在 drop 或 destroy 中处理。
    }
}
//运行中定时器动态数组
let mut timers: Vec<RtTimer> = Vec::new();

//初始化timer请使用new方法
//删除timer请使用destroy方法

//开始定时器

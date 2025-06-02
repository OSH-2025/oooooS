extern crate alloc;
use crate::rtdef::RtObject;
use crate::clock::rt_tick_get;
use crate::irq::rt_hw_interrupt_disable;
use crate::irq::rt_hw_interrupt_enable;
use core::ffi::c_void;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;
use cortex_m::peripheral::syst::SystClkSource;

//由于rust中可以使用高级容器：动态数组，所以不需要使用链表（跳表）算法来加速
//定时器的查找，而可以使用二分查找，所以我们决定把定时器存放在动态数组中

pub const RT_TIMER_FLAG_ACTIVATED: u8 = 0x1;

//定时器结构体
//注意回调函数应是一个函数或闭包，且其类型满足FnMut(*mut ()) + Send + Sync + 'static
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
            parent: parent_object.clone(),
            timeout_callback: timeout_func,
            user_data: parameter,
            init_tick,
            timeout_tick,
        }
    }

    /// 设置定时器回调函数和用户数据
    pub fn set_timeout_callback<F, T: 'static>(&mut self, mut callback: F, data: &mut T)
    where
        F: FnMut(*mut T) + Send + Sync + 'static,
    {
        self.user_data = data as *mut T as *mut c_void;
        self.timeout_callback = Some(Box::new(move |raw_ptr| {
            let typed_ptr = raw_ptr as *mut T;
            if !typed_ptr.is_null() {
                unsafe { callback(typed_ptr); }
            }
        }));
    }

    /// 触发定时器回调函数
    pub fn trigger_timeout(&mut self) {
        if let Some(callback) = self.timeout_callback.as_mut() {
            let user_data_ptr = self.user_data;
            unsafe {
                 callback(user_data_ptr as *mut());
            }
        }
    }
}

impl Drop for RtTimer {
    fn drop(&mut self) {
        // This code runs automatically when RtTimer is dropped (last Arc is dropped)
        // println!("RtTimer dropped: {}", String::from_utf8_lossy(&self.parent.name));
        // TODO: If user_data points to manually allocated memory, free it here
        // For example: unsafe { rt_free(self.user_data); } // Assuming you have an rt_free equivalent
    }
}

// 定时器句柄类型
pub type TimerHandle = Arc<Mutex<RtTimer>>;

// 单线程环境下的全局定时器数组，只能通过本文件的接口操作
static mut TIMERS: Option<Mutex<Vec<TimerHandle>>> = Some(Mutex::new(Vec::new()));

/// 初始化定时器系统（可选，根据需要调用）
pub fn timer_system_init() {
    unsafe {
        // TIMERS is already initialized in the static declaration now
        // TIMERS = Some(Mutex::new(Vec::new()));
    }
}

/// 启动定时器，将其插入到timers数组中，保持timeout_tick升序
pub fn rt_timer_start(timer: TimerHandle) {
    let level = rt_hw_interrupt_disable();
    unsafe {
        if let Some(ref timers_mutex) = TIMERS {
            let mut timers = timers_mutex.lock();
            let mut timer_locked = timer.lock();
            timer_locked.parent.flag |= RT_TIMER_FLAG_ACTIVATED;
            timer_locked.timeout_tick = timer_locked.init_tick + rt_tick_get();
            let timeout_tick = timer_locked.timeout_tick;
            drop(timer_locked);

            let pos = timers.binary_search_by(|probe| probe.lock().timeout_tick.cmp(&timeout_tick))
                .unwrap_or_else(|e| e);
            timers.insert(pos, timer);
        }
    }
    rt_hw_interrupt_enable(level);
}

/// 停止定时器，将其从timers数组中移除并释放空间
pub fn rt_timer_stop(timer: &TimerHandle) {
    let level = rt_hw_interrupt_disable();
    unsafe {
        if let Some(ref timers_mutex) = TIMERS {
            let mut timers = timers_mutex.lock();
            if let Some(pos) = timers.iter().position(|t| Arc::ptr_eq(t, timer)) {
                timers.remove(pos);
            }
        }
    }
    rt_hw_interrupt_enable(level);
}

/// 控制定时器参数和状态，仿照RT-Thread的rt_timer_control实现
pub fn rt_timer_control(timer: &TimerHandle, cmd: i32, arg: *mut c_void) {
    let level = rt_hw_interrupt_disable();
    let mut timer_ref = timer.lock();
    match cmd {
        0 /* RT_TIMER_CTRL_GET_TIME */ => {
            unsafe { *(arg as *mut u32) = timer_ref.init_tick; }
        }
        1 /* RT_TIMER_CTRL_SET_TIME */ => {
            unsafe {
                let new_tick = *(arg as *mut u32);
                timer_ref.init_tick = new_tick;
            }
        }
        2 /* RT_TIMER_CTRL_SET_ONESHOT */ => {
            timer_ref.parent.flag &= !0x2; // 假设0x2为RT_TIMER_FLAG_PERIODIC
        }
        3 /* RT_TIMER_CTRL_SET_PERIODIC */ => {
            timer_ref.parent.flag |= 0x2; // 假设0x2为RT_TIMER_FLAG_PERIODIC
        }
        4 /* RT_TIMER_CTRL_GET_STATE */ => {
            unsafe {
                if timer_ref.parent.flag & RT_TIMER_FLAG_ACTIVATED != 0 {
                    *(arg as *mut u32) = RT_TIMER_FLAG_ACTIVATED as u32;
                } else {
                    *(arg as *mut u32) = 0; // RT_TIMER_FLAG_DEACTIVATED
                }
            }
        }
        5 /* RT_TIMER_CTRL_GET_REMAIN_TIME */ => {
            unsafe { *(arg as *mut u32) = timer_ref.timeout_tick; }
        }
        _ => {}
    }
    rt_hw_interrupt_enable(level);
}

/// 检查所有定时器，处理超时事件
pub fn rt_timer_check() {
    let mut expired_timers: Vec<TimerHandle> = Vec::new();
    let level = rt_hw_interrupt_disable();
    let current_tick = rt_tick_get();
    unsafe {
        if let Some(ref timers_mutex) = TIMERS {
            let mut timers = timers_mutex.lock();
            // 二分查找第一个未超时定时器的位置
            let pos = timers.binary_search_by(|timer| {
                let t = timer.lock();
                if (current_tick.wrapping_sub(t.timeout_tick)) < 0x7fffffff { // This logic might need adjustment for wrapping arithmetic
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            }).unwrap_or_else(|e| e);
            // 批量移除所有超时定时器
            expired_timers = timers.drain(0..pos).collect();
        }
    }
    rt_hw_interrupt_enable(level);
    for timer in expired_timers {
        let mut t = timer.lock();
        (*t).trigger_timeout();
        let is_periodic = t.parent.flag & 0x2 != 0;
        let is_activated = t.parent.flag & RT_TIMER_FLAG_ACTIVATED != 0;
        
        // Unlock before potential recursive call to rt_timer_start
        drop(t);

        if is_periodic && is_activated {
            // Re-lock to modify flag if needed, or handle state transition before drop
            let mut t_recheck = timer.lock();
            t_recheck.parent.flag &= !RT_TIMER_FLAG_ACTIVATED;
            drop(t_recheck);
            rt_timer_start(timer.clone());
        }
    }
}

/// 初始化系统定时器（SysTick）
/// 配置 SysTick 以产生中断，用于 tick 计数
pub fn rt_system_timer_init(mut syst: cortex_m::peripheral::SYST, clocks: &stm32f4xx_hal::rcc::Clocks) {
    // 获取系统时钟频率
    let sys_clk_freq = clocks.sysclk().to_Hz(); // 获取系统时钟频率 (Hz)

    // 计算 SysTick 的重载值
    let reload_value = (sys_clk_freq / crate::rtconfig::RT_TICK_PER_SECOND) - 1; // 使用 crate::rtdef

    // 配置 SysTick
    syst.set_reload(reload_value);
    syst.enable_counter();
    syst.enable_interrupt();
    // 使用 AHB 总线时钟作为 SysTick 的时钟源
    syst.set_clock_source(SystClkSource::Core); // 使用导入的 SystClkSource
}
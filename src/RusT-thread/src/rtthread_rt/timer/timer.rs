//! 定时器模块
//! 
//! 本模块实现了RT-Thread的定时器功能
//! 包括定时器的创建、启动、停止、控制等


extern crate alloc;
use crate::rtthread_rt::rtdef::RtObject;
use crate::rtthread_rt::timer::clock::rt_tick_get;
use crate::rtthread_rt::hardware::irq::{rt_hw_interrupt_disable, rt_hw_interrupt_enable};
use crate::rtthread_rt::rtconfig::RT_TICK_PER_SECOND;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;
use cortex_m::peripheral::syst::SystClkSource;
use core::fmt::{self, Display};
use cortex_m_semihosting::hprintln;

//由于rust中可以使用高级容器：动态数组，所以不需要使用链表（跳表）算法来加速
//定时器的查找，而可以使用二分查找，所以我们决定把定时器存放在动态数组中
pub const RT_TIMER_FLAG_ACTIVATED: u8 = 0x1;
// Assuming 0x2 is RT_TIMER_FLAG_PERIODIC based on usage
const RT_TIMER_FLAG_PERIODIC: u8 = 0x2;


/// 定时器结构体
/// 注意回调函数应是一个函数或闭包，且其类型满足FnMut() + Send + Sync + 'static
/// 
/// 使用示例：
/// ```
/// let timer = Arc::new(Mutex::new(RtTimer::new("timer", 0, 0, None, 0, 0))); // 创建一个定时器
/// timer.set_timeout_callback(|| {
///     hprintln!("timer timeout");
/// });
/// rt_timer_start(timer.clone());
/// ```
/// 

pub struct RtTimer {
    pub parent: RtObject,
    pub timeout_callback: Option<Callable>,
    pub init_tick: u32,
    pub timeout_tick: u32,
}

impl Display for RtTimer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 使用core::str::from_utf8来处理字节数组
        let name_str = core::str::from_utf8(&self.parent.name)
            .unwrap_or("invalid_utf8")
            .trim_matches('\0'); // 移除null字符
        write!(f, "RtTimer {{ name: {:?}, init_tick: {}, timeout_tick: {} }}", name_str, self.init_tick, self.timeout_tick)
    }
}


impl RtTimer {
    /// 创建一个新的 RtTimer 实例
    /// 回调函数通过闭包捕获其环境来访问所需数据，无需额外的 user_data 指针
    /// * `name` 定时器名称
    /// * `obj_type` 定时器对象类型
    /// * `flag` 定时器标志：0：单次定时器，2：周期定时器
    /// * `timeout_func` 定时器回调函数
    /// * `init_tick` 定时器初始超时时间
    /// * `timeout_tick` 定时器超时时间
    pub fn new(
        name: &str,
        obj_type: u8,
        flag: u8,
        timeout_func: Option<Callable>,
        init_tick: u32,
        timeout_tick: u32,
    ) -> Self {
        let parent_object = RtObject::new(name, obj_type, flag);
        Self {
            parent: parent_object.clone(),
            timeout_callback: timeout_func,
            init_tick,
            timeout_tick,
        }
    }

    /// 设置定时器回调函数
    /// 回调函数通过闭包捕获其环境来访问所需数据
    pub fn set_timeout_callback<F>(&mut self, callback: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        self.timeout_callback = Some(Box::new(callback));
    }

    /// 触发定时器回调函数
    pub fn trigger_timeout(&mut self) {
        if let Some(callback) = self.timeout_callback.as_mut() {
            // The callback directly accesses captured data, no parameter needed
            callback();
        }
    }
}

impl Drop for RtTimer {
    fn drop(&mut self) {
        // This code runs automatically when RtTimer is dropped (last Arc is dropped)
        // println!("RtTimer dropped: {}", String::from_utf8_lossy(&self.parent.name));
        // user_data is removed, so no raw pointer to free here
    }
}

/// 定时器句柄类型
pub type TimerHandle = Arc<Mutex<RtTimer>>;
pub type Callable = Box<dyn FnMut() + Send + Sync + 'static>;

/// 单线程环境下的全局定时器数组，只能通过本文件的接口操作
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
            let mut timers = timers_mutex.lock();// 获取定时器数组锁
            let mut timer_locked = timer.lock();// 获取定时器锁
            timer_locked.parent.flag |= RT_TIMER_FLAG_ACTIVATED;// 设置定时器激活状态
            timer_locked.timeout_tick = timer_locked.init_tick.wrapping_add(rt_tick_get());// 设置定时器超时时间
            let timeout_tick = timer_locked.timeout_tick;// 获取定时器超时时间
            drop(timer_locked); // 释放定时器锁

            // 使用定时器超时时间进行二分查找
            let pos = timers.binary_search_by(|probe| {
                 let probe_locked = probe.lock();
                 probe_locked.timeout_tick.cmp(&timeout_tick)
            })
                .unwrap_or_else(|e| e);
            timers.insert(pos, timer);// 将定时器插入到timers数组中
        }
    }
    
    // hprintln!("rt_timer_start at tick: {}", rt_tick_get());
    // // 打印定时器内容
    // unsafe {
    //     if let Some(ref timers_mutex) = TIMERS {
    //         let timers = timers_mutex.lock();
    //         for timer in timers.iter() {
    //             let timer_locked = timer.lock();
    //             hprintln!("timer: {}", *timer_locked);
    //             drop(timer_locked);
    //         }
    //     }
    // }
    
    rt_hw_interrupt_enable(level);
}

/// 停止定时器，将其从timers array 中移除并释放空间
pub fn rt_timer_stop(timer: &TimerHandle) {
    let level = rt_hw_interrupt_disable();
    unsafe {
        if let Some(ref timers_mutex) = TIMERS {
            let mut timers = timers_mutex.lock();
            // Need to compare the Arc pointers themselves, not the locked RtTimer
            if let Some(pos) = timers.iter().position(|t| Arc::ptr_eq(t, timer)) {
                timers.remove(pos);
            }
        }
    }
    rt_hw_interrupt_enable(level);
}

/// 定义定时器控制命令的枚举
/// 这种方式比使用原始指针更类型安全和符合 Rust 习惯
/// 使用示例：
/// ```
/// let timer = Arc::new(Mutex::new(RtTimer::new("timer", 0, 0, None, 0, 0)));
/// let mut time = 0;
/// rt_timer_control(&timer, TimerControlCmd::GetTime(&mut time));
/// ```
pub enum TimerControlCmd<'a> {
    /// 获取初始超时时间（tick 值），结果存入 `&mut u32`
    GetTime(&'a mut u32),
    /// 设置初始超时时间（tick 值），从 `u32` 读取
    SetTime(u32),
    /// 设置定时器为单次模式
    SetOneshot,
    /// 设置定时器为周期模式
    SetPeriodic,
    /// 获取定时器的激活状态 (0 或 RT_TIMER_FLAG_ACTIVATED)，结果存入 `&mut u32`
    GetState(&'a mut u32),
    /// 获取距离超时的剩余时间（tick 值），结果存入 `&mut u32`
    /// 注意：这里返回的是 `timeout_tick`，如果需要真正的"剩余时间"，
    /// 可能需要根据当前 tick 重新计算。此处保留与原 C 函数行为相似。
    GetRemainTime(&'a mut u32),
    // 根据需要添加其他控制命令
}

/// 控制定时器参数和状态，仿照RT-Thread的rt_timer_control实现
/// 使用 TimerControlCmd 枚举代替 C 的 (cmd, arg) 对
pub fn rt_timer_control(timer: &TimerHandle, cmd: TimerControlCmd) {
    let level = rt_hw_interrupt_disable();
    let mut timer_ref = timer.lock();

    match cmd {
        TimerControlCmd::GetTime(result) => {
            *result = timer_ref.init_tick;
        }
        TimerControlCmd::SetTime(new_tick) => {
            timer_ref.init_tick = new_tick;
            // 如果定时器已激活，修改 init_tick 后通常需要重新计算 timeout_tick
            // 这里仅修改 init_tick，实际 RTOS 中可能需要重新启动或调整定时器
        }
        TimerControlCmd::SetOneshot => {
            timer_ref.parent.flag &= !RT_TIMER_FLAG_PERIODIC;
        }
        TimerControlCmd::SetPeriodic => {
            timer_ref.parent.flag |= RT_TIMER_FLAG_PERIODIC;
        }
        TimerControlCmd::GetState(result) => {
            if timer_ref.parent.flag & RT_TIMER_FLAG_ACTIVATED != 0 {
                *result = RT_TIMER_FLAG_ACTIVATED as u32;
            } else {
                *result = 0; // 假设 0 表示 RT_TIMER_FLAG_DEACTIVATED
            }
        }
        TimerControlCmd::GetRemainTime(result) => {
             // 根据 RT-Thread 的行为，此处可能需要返回 timeout_tick - current_tick 的差值
             // 但 u32 减法需要处理回绕。简单返回 timeout_tick 可能是为了适配某种特定用法。
             // 如果需要真正的剩余时间，应计算 current_tick.wrapping_sub(timer_ref.timeout_tick) 的补码或类似逻辑。
             *result = timer_ref.timeout_tick;
        }
        // 如果所有命令都由枚举覆盖，则无需 _ => {} 分支
    }

    rt_hw_interrupt_enable(level);
}

/// 检查所有定时器，处理超时事件
pub fn rt_timer_check() { 
    // if rt_tick_get() % 1000 == 0 {
    //     hprintln!("rt_timer_check at tick: {}", rt_tick_get());
    //     unsafe {
    //         if let Some(ref timers_mutex) = TIMERS {
    //             let timers = timers_mutex.lock();
    //             for timer in timers.iter() {
    //                 let timer_locked = timer.lock();
    //                 hprintln!("timer: {}", *timer_locked);
    //                 drop(timer_locked);
    //             }
    //         }
    //     }
    // }
    let mut expired_timers: Vec<TimerHandle> = Vec::new();
    let level = rt_hw_interrupt_disable();
    let current_tick = rt_tick_get();
    unsafe {
        if let Some(ref timers_mutex) = TIMERS {
            let mut timers = timers_mutex.lock();
            
            // 找到第一个未过期的定时器位置
            let mut expired_count = 0;
            for timer_handle in timers.iter() {
                let t = timer_handle.lock();
                let diff = current_tick.wrapping_sub(t.timeout_tick);
                
                // 如果diff >= 0，说明定时器已过期
                if (diff as i32) >= 0 {
                    expired_count += 1;
                } else {
                    // 找到第一个未过期的定时器，停止搜索
                    break;
                }
            }
            
            // 只移除已过期的定时器
            if expired_count > 0 {
                // hprintln!("expired_count: {}", &expired_count);
                expired_timers = timers.drain(0..expired_count).collect();
            }
        }
    }
    rt_hw_interrupt_enable(level);

    for timer_handle in expired_timers {
        // Lock the timer to trigger the callback and check periodic flag
        let mut t = timer_handle.lock();
        (*t).trigger_timeout(); // Trigger the callback which uses captured data

        let is_periodic = t.parent.flag & RT_TIMER_FLAG_PERIODIC != 0;
        let is_activated = t.parent.flag & RT_TIMER_FLAG_ACTIVATED != 0;

        // Unlock before potential recursive call to rt_timer_start
        drop(t);

        // If periodic and still active (not stopped within the callback)
        if is_periodic && is_activated {
             // Re-lock to update flag and re-start
             let mut t_recheck = timer_handle.lock();
             // The timer is about to be re-started, so it's considered active for the next cycle
             // No need to clear RT_TIMER_FLAG_ACTIVATED here, rt_timer_start will set it.
             drop(t_recheck);
             rt_timer_start(timer_handle.clone()); // Re-start the periodic timer
        }
    }
}


/// 初始化系统定时器（SysTick）
/// 配置 SysTick 以产生中断，用于 tick 计数
pub fn rt_system_timer_init(mut syst: cortex_m::peripheral::SYST, clocks: &stm32f4xx_hal::rcc::Clocks) {
    // 获取系统时钟频率
    let sys_clk_freq = clocks.sysclk().to_Hz(); // 获取系统时钟频率 (Hz)

    // 计算 SysTick 的重载值
    // RT_TICK_PER_SECOND should be defined elsewhere, e.g., in rtconfig
    let reload_value = (sys_clk_freq / RT_TICK_PER_SECOND) - 1;

    // Configure SysTick
    syst.set_reload(reload_value);
    syst.enable_counter();
    syst.enable_interrupt();// 使能SysTick中断
    // Use the core clock as SysTick source
    syst.set_clock_source(SystClkSource::Core);
}
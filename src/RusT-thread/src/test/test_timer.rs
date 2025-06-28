extern crate alloc;
use alloc::sync::Arc;
use spin::Mutex;
use alloc::boxed::Box;
use cortex_m_semihosting::hprintln;
use cortex_m::asm;
use crate::rtthread_rt::timer::{RtTimer, rt_timer_start, rt_tick_get, rt_timer_stop};

struct SharedCounter {
    count: u32,
}

/// 定时器使用示例
pub fn example_timer() {
    // 1. Create the shared data, protected by Arc and Mutex
    let counter = Arc::new(Mutex::new(SharedCounter { count: 0 }));

    // 2. Clone the Arc for the timer's callback
    let counter_for_callback = counter.clone();

    // 3. Define the callback closure.
    // This closure captures `counter_for_callback`.
    // `move` keyword is used to move `counter_for_callback` into the closure.
    let timer_callback = move || {
        // Inside the callback, acquire the mutex lock to access the shared data
        let mut data = counter_for_callback.lock();
        data.count += 1;
        hprintln!("Timer callback triggered! Counter: {}", data.count);

        // You can access other things captured by the closure if needed
        // For example, if you captured a timer handle to stop the timer after N calls:
        // if data.count >= 5 {
        //    // Need to pass the timer handle into the closure or find another way to stop it
        //    // Stopping from within the same timer's callback needs careful handling to avoid deadlocks
        // }
    };

    // 4. Create the RtTimer instance.
    // Pass the Boxed closure as the timeout_func.
    let my_timer = Arc::new(Mutex::new(RtTimer::new(
        "my_periodic_timer", // name
        0, // obj_type (example value)
        0x2, // flag (assuming 0x2 is periodic)
        Some(Box::new(timer_callback)), // timeout_func
        10, // init_tick (initial delay in ticks)
        10, // timeout_tick (period for periodic timers)
    )));

    // 5. Start the timer
    rt_timer_start(my_timer.clone());

    // In a real RTOS scenario, you would likely start the scheduler here
    // and the timer interrupt would eventually call rt_timer_check.

    // You can access the shared data from the main thread as well
    // let current_count = counter.lock().count;
    // println!("Current count from main: {}", current_count);
}    

pub fn run_all_timer_tests() {
    // single_timer_test();
    periodic_timer_test();
}

/// 简单的定时器测试
pub fn single_timer_test() {
    let timer_callback = move || {
        hprintln!("单次定时器测试：成功执行回调函数 at tick: {}", rt_tick_get());
    };

    let timer = Arc::new(Mutex::new(RtTimer::new(
        "single_timer_test", // name
        0, // obj_type (example value)
        0x0, // flag (assuming 0x2 is periodic)
        Some(Box::new(timer_callback)), // timeout_func
        1000, // init_tick (initial delay in ticks)
        1000, // timeout_tick (period for periodic timers)
    )));
    
    // 打印定时器内容
    {
        let timer_ref = timer.lock();
        hprintln!("timer: {}", *timer_ref);
    }

    rt_timer_start(timer.clone());
}

/// 周期性定时器测试
pub fn periodic_timer_test() {
    let counter = Arc::new(Mutex::new(SharedCounter { count: 0 }));
    
    // 先创建timer变量，但暂时不设置回调函数
    let timer = Arc::new(Mutex::new(RtTimer::new(
        "periodic_timer_test", // name
        0, // obj_type (example value)
        0x2, // flag (assuming 0x2 is periodic)
        None, // 暂时不设置回调函数
        1000, // init_tick (initial delay in ticks)
        1000, // timeout_tick (period for periodic timers)
    )));
    
    // 创建Clone
    let timer_clone = timer.clone();
    
    let timer_callback = move || {
        let mut data = counter.lock();
        data.count += 1;
        hprintln!("周期性定时器测试：成功执行回调函数 at tick: {}, count: {}", rt_tick_get(), data.count);
        if data.count >= 5 {
            hprintln!("周期性定时器测试：达到5次回调，停止定时器");
            rt_timer_stop(&timer_clone);
        }
    };

    // 现在设置回调函数
    {
        let mut timer_ref = timer.lock();
        timer_ref.set_timeout_callback(timer_callback);
    }

    rt_timer_start(timer.clone());
}


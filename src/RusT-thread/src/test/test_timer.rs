extern crate alloc;
use crate::timer::{RtTimer, TimerHandle, rt_timer_start};
use crate::rtdef::RtObject; // Assuming RtObject is needed for timer creation
use alloc::sync::Arc;
use spin::Mutex;
use alloc::boxed::Box;
use cortex_m_semihosting::hprintln;

struct SharedCounter {
    count: u32,
}

pub fn run_all_timer_tests() {
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
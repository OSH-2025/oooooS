#![no_std]
#![no_main]

#![allow(warnings)]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use cortex_m::asm;
use cortex_m_rt::{entry, exception}; // 引入 exception 宏
use cortex_m_semihosting::hprintln;
// 引入 stm32f4xx-hal crate
use stm32f4xx_hal::{
    prelude::*, // 引入一些常用的 trait 和类型
    pac,        // 引入外设访问 crate
    rcc::RccExt, // 引入 RCC 扩展 trait
    // flash::FlashExt, // 根据需要引入 Flash 扩展
    // power::Dbgmcu, // 根据需要引入 Debug MCU
};
// use fugit::RateExtU32; // 引入频率单位扩展 trait

mod rtthread_rt;
mod test;
mod user_main;

use user_main::main_entry;
use rtthread_rt::rtconfig;
use rtthread_rt::thread::{
    thread,
    idle,
    scheduler,
    thread_priority_table,
};
use rtthread_rt::rtdef::ThreadState;

/// --- SysTick 中断处理函数 ---
/// 使用 #[exception] 宏将此函数标记为 SysTick 中断处理程序
#[exception]
unsafe fn SysTick() {
    // 在 SysTick ISR 中调用 rt_tick_increase
    // rt_tick_increase 函数现在在 clock 模块中
    rtthread_rt::timer::clock::rt_tick_increase();

    // 如果需要，可以在这里检查是否需要进行任务调度
    // 例如：crate::rtthread::rt_schedule(); // 假设存在调度函数
}

#[entry]
fn entry() -> ! {

    hprintln!("Hello, world!");
    
    //初始化
    init();
    
    if cfg!(feature = "test") {
        // hprintln!("Running tests...");
        test::run_all_tests();
        // hprintln!("Tests finished.");
    }

    //在此处初始化线程，并启动调度器（将跳转到主线程入口）
    init_thread();
    // 此后代码不会被执行

    panic!("程序不应该运行到这里，请检查初始化是否正确");
}

/// 初始化函数
/// 初始化硬件、内存、定时器等
fn init() {
    // hprintln!("Initializing...");


    // 获取外设的所有权
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // --- HAL 时钟配置示例 ---
    // 这部分代码根据您的具体硬件和需求进行修改
    // 以下是一个使用 HSE 并配置 PLL 的示例
    // 您需要根据您的晶振频率和期望的系统频率进行调整
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    // // --- SysTick 初始化 ---
    let syst = cp.SYST;
    // 调用 timer.rs 中的 rt_system_timer_init 函数来配置 SysTick
    rtthread_rt::timer::timer::rt_system_timer_init(syst, &clocks);
    // 内存分配器初始化
    rtthread_rt::mem::allocator::init_heap();

    // hprintln!("Initialization finished.");
}

fn init_thread() {
    // hprintln!("Initializing thread...");
    idle::init_idle();
    // 创建用户主线程
    let main = thread::rt_thread_create("main", main_entry as usize, rtconfig::RT_MAIN_THREAD_STACK_SIZE as usize, rtconfig::RT_MAIN_THREAD_PRIORITY as u8, 1000);
    main.inner.exclusive_access().stat = ThreadState::Ready;
    thread_priority_table::insert_thread(main.clone());
    // 启动调度器
    scheduler::rt_schedule_start();
    // hprintln!("Thread initialized.");
}




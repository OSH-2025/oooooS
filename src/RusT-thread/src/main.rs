#![no_std]
#![no_main]
#![allow(warnings)]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

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

use fugit::RateExtU32; // 引入频率单位扩展 trait

// 引入 timer 和 clock 模块
mod rtdef;
mod irq;
mod context;
mod rtthread;
mod kservice;
mod mem;
mod rtconfig;
mod clock;
mod timer;
mod test;
mod cpuport;


#[entry]
fn main() -> ! {

    hprintln!("Hello, world!");

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
    timer::rt_system_timer_init(syst, &clocks);
    
    //初始化内存
    init();
    
    if cfg!(feature = "test") {
            // 使用条件编译来包含测试代码
        #[cfg(feature = "test_small_mem")]
        {
            hprintln!("开始小内存管理测试...");
            test::test_small_mem::run_simple_mem_tests();
            hprintln!("小内存管理测试完成！");
        }
        #[cfg(feature = "test_timer")]
        {
            hprintln!("开始定时器测试...");
            test::test_timer::run_all_timer_tests();
            hprintln!("定时器测试完成！");
        }
        test::run_all_tests();
        hprintln!("Tests finished.");
    }

    // --- 应用主循环 ---
    loop {
        // 可以在这里添加应用程序的主要逻辑
        asm::nop(); // 空操作，防止编译器优化掉循环
    }
}

fn init() {
    // 内存分配器初始化
    hprintln!("start init...");
    mem::allocator::init_heap();
    // context::init(); // 如果需要，初始化上下文
    // hprintln!("init done");
}

// --- SysTick 中断处理函数 ---
// 使用 #[exception] 宏将此函数标记为 SysTick 中断处理程序
#[exception]
unsafe fn SysTick() {
    // 在 SysTick ISR 中调用 rt_tick_increase
    // rt_tick_increase 函数现在在 clock 模块中
    clock::rt_tick_increase();

    // 如果需要，可以在这里检查是否需要进行任务调度
    // 例如：crate::rtthread::rt_schedule(); // 假设存在调度函数
}


// ! 测试注意
// ! 推荐的测试方式是单独写一个测试文件，
// ! 例如 `test_xxx.rs`，然后在 `Cargo.toml` 中添加对应的测试条件编译
// ! 例如 test_xxx = []
// ! 在main.rs中使用 `#[cfg(test_xxx)]` 来包含测试代码。
// ! 这样可以避免在主程序中引入测试代码，保持代码的整洁性和可维护性。
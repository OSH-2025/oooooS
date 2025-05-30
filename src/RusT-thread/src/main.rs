#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;


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
    init();
    if cfg!(feature = "test") {
        test::run_all_tests();
    }
    loop {
        asm::nop();
    }    

}

fn init() {
    mem::allocator::init_heap();
    // context::init();
    // hprintln!("init done");
}


// ! 测试注意
// ! 推荐的测试方式是单独写一个测试文件，
// ! 例如 `test_xxx.rs`，然后在 `Cargo.toml` 中添加对应的测试条件编译
// ! 例如 test_xxx = []
// ! 在main.rs中使用 `#[cfg(test_xxx)]` 来包含测试代码。
// ! 这样可以避免在主程序中引入测试代码，保持代码的整洁性和可维护性。
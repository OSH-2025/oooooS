//! 测试（也可以是示例代码）
//! 
//! 推荐的测试方式是单独写一个测试文件，
//! 例如 `test_xxx.rs`，然后在 `Cargo.toml` 中添加对应的测试条件编译
//! 例如 test_xxx = []
//! 在test/mod.rs中使用 `#[cfg(test_xxx)]` 来包含测试代码。
//! 这样可以避免在主程序中引入测试代码，保持代码的整洁性和可维护性。

#![allow(warnings)]
pub mod test_thread;
pub mod test_mem;
// use cortex_m_semihosting::hprintln;
pub mod test_excp;
pub mod test_scheduler;
pub mod test_interupt;
pub mod example;
pub mod test_cell;
pub mod performance_test;
pub mod switch_time_test;
pub mod comprehensive_example;
pub mod test_interruput_latency;
pub mod thread_creation_test;

// #[cfg(feature = "test_timer")]
pub mod test_timer;
#[cfg(feature = "test_small_mem")]
pub mod test_small_mem;
#[cfg(feature = "test_allocator_compare")]
pub mod test_allocator_compare;
pub mod example_mfq;
pub mod test_all;



pub fn run_all_tests() {
    test_all::print_logo();

    // test_timer::run_all_timer_tests();

    // test::test_small_mem::run_simple_mem_tests();
    
    // test_mem::test_vec();
    // test_mem::test_alloc_dealloc();
    // test_mem::test_box();

    // test_thread::test_thread_context_switch();
    // test_thread::test_thread_context_switch_from_to();
    // test_scheduler::test_insert_thread();
    // test_scheduler::test_schedule_start();
    // test_scheduler::test_schedule();

    // test_interupt::test_interupt();
    // test_interupt::test_RtIntrFreeCell();
    // test_cell::test_cell();
}

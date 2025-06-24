#![allow(warnings)]
pub mod test_thread;
pub mod test_mem;
// use cortex_m_semihosting::hprintln;
pub mod test_excp;
pub mod test_scheduler;
pub mod test_interupt;



#[cfg(feature = "test_timer")]
pub mod test_timer;
#[cfg(feature = "test_small_mem")]
pub mod test_small_mem;
#[cfg(feature = "test_allocator_compare")]
pub mod test_allocator_compare;


pub fn run_all_tests() {
    #[cfg(feature = "test_small_mem")]
    {
        hprintln!("开始小内存管理测试...");
        test::test_small_mem::run_simple_mem_tests();
        hprintln!("小内存管理测试完成！");
    }
    #[cfg(feature = "test_timer")]
    {
        hprintln!("开始定时器测试...");
        test_timer::run_all_timer_tests();
        hprintln!("定时器测试完成！");
    }
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
}

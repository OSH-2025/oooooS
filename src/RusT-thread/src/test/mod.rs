pub mod test_thread;
pub mod test_mem;

// 按照 main.rs 中的建议，使用条件编译包含测试代码
#[cfg(feature = "test_small_mem")]
pub mod test_small_mem;

// 全局分配器对比测试
#[cfg(feature = "test_allocator_compare")]
pub mod test_allocator_compare;
pub mod test_excp;

#[cfg(feature = "test_timer")]
pub mod test_timer;
pub fn run_all_tests() {
    // test_mem::test_vec();
    // test_mem::test_alloc_dealloc();
    // test_mem::test_box();
    // test_thread::test_thread_context_switch();
}

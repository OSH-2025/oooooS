//! 内存分配器
//! 
//! 定义了内存分配器，并给出了对应的init_heap函数(堆初始化函数)
//! 默认使用buddy_system_allocator

#![warn(unused_imports)]

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m_semihosting::hprintln;

// Global allocator implementation

// Initialize heap status
static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);
// Re-export from the original file
const HEAP_START: usize = 0x2000_0400;  // RAM 起始地址
const HEAP_SIZE: usize = 32 * 1024;     // 使用 32KB 作为堆大小，留出足够空间给其他用途



// Declare global allocators based on the selected feature
// 确保两个分配器是互斥的

#[cfg(all(feature = "good_memory_allocator", not(feature = "buddy_system_allocator")))]
use good_memory_allocator::SpinLockedAllocator;

#[cfg(all(feature = "buddy_system_allocator", not(feature = "good_memory_allocator")))]
use buddy_system_allocator::LockedHeap;

#[cfg(all(feature = "good_memory_allocator", not(feature = "buddy_system_allocator")))]
#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

#[cfg(all(feature = "buddy_system_allocator", not(feature = "good_memory_allocator")))]
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

/// 初始化堆内存（一定要在main函数之前调用）
/// 
/// 使用示例:
/// ```rust
/// init_heap();
/// ```
pub fn init_heap() {
    // hprintln!("Initializing heap...");
    if !HEAP_INITIALIZED.load(Ordering::SeqCst) {
        // hprintln!("Heap not initialized.");
        unsafe {
            #[cfg(all(feature = "good_memory_allocator", not(feature = "buddy_system_allocator")))]
            {
                ALLOCATOR.init(HEAP_START, HEAP_SIZE);
            }
            
            #[cfg(all(feature = "buddy_system_allocator", not(feature = "good_memory_allocator")))]
            {
                // hprintln!("Initializing buddy_system_allocator...");
                HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
                // hprintln!("buddy_system_allocator initialized.");
            }
        }
        HEAP_INITIALIZED.store(true, Ordering::SeqCst);
    }
    // hprintln!("Heap initialized.");
}

// 编译时检查：确保只启用了一个分配器
#[cfg(all(feature = "good_memory_allocator", feature = "buddy_system_allocator"))]
compile_error!("不能同时启用两个内存分配器！请只选择一个：good_memory_allocator 或 buddy_system_allocator");

#[cfg(not(any(feature = "good_memory_allocator", feature = "buddy_system_allocator")))]
compile_error!("必须启用一个内存分配器！请选择：good_memory_allocator 或 buddy_system_allocator");

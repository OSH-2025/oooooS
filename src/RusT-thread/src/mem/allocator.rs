use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m_semihosting::hprintln;

// Global allocator implementation

// Initialize heap status
static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);
// Re-export from the original file
const HEAP_START: usize = 0x2000_0400;  // RAM 起始地址
const HEAP_SIZE: usize = 32 * 1024;     // 使用 32KB 作为堆大小，留出足够空间给其他用途



// Declare global allocators based on the selected feature

#[cfg(feature = "good_memory_allocator")]
use good_memory_allocator::SpinLockedAllocator;

#[cfg(feature = "buddy_system_allocator")]
use buddy_system_allocator::LockedHeap;

#[cfg(feature = "good_memory_allocator")]
#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

#[cfg(feature = "buddy_system_allocator")]
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

#[cfg(feature = "good_memory_allocator")]
/// Initialize heap memory for the global allocator
pub fn init_heap() {
    if !HEAP_INITIALIZED.load(Ordering::SeqCst) {
        unsafe {
            hprintln!("init_heap");
            ALLOCATOR.init(HEAP_START, HEAP_SIZE);
            hprintln!("init_heap done");
        }
        HEAP_INITIALIZED.store(true, Ordering::SeqCst);
    }
}

#[cfg(feature = "buddy_system_allocator")]
/// Initialize heap memory for the global allocator
pub fn init_heap() {
    if !HEAP_INITIALIZED.load(Ordering::SeqCst) {
        hprintln!("init_heap");
        unsafe {
            HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
            // or
            // HEAP_ALLOCATOR.lock().add_to_heap(HEAP_START, HEAP_START + HEAP_SIZE);
        }
        HEAP_INITIALIZED.store(true, Ordering::SeqCst);
        hprintln!("init_heap done");
    }
}
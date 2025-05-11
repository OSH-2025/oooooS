/// Memory management functions
/// todo: 
/// 1. ...
/// 
/// 
/// 我(@luohaomin)找到了一些可用的global_allocator
/// 1. good_memory_allocator
/// 2. buddy_system_allocator
/// 3. linked_list_allocator
/// 4. ...
/// 我先将它们全部引入，然后根据需要进行选择
/// 
/// 当然，之后负责Mem管理的人，可以自行实现更加合适的内存管理方式，将其替换


const HEAP_START: usize = 0x10000000;
const HEAP_SIZE: usize = 1024 * 1024; // 1MB

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
pub fn init_heap() {
    unsafe {
        ALLOCATOR.init(heap_start, heap_size);
    }
}

#[cfg(feature = "buddy_system_allocator")]
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        // or
        HEAP_ALLOCATOR.lock().add_to_heap(HEAP_START, HEAP_START + HEAP_SIZE);
    }
} 
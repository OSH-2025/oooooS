//! Memory management module for RT-Thread
//! 
//! This module provides memory management functionality for RT-Thread.
//! It supports multiple allocator implementations through features:
//! - good_memory_allocator: A simple and efficient memory allocator
//! - buddy_system_allocator: A buddy system based memory allocator
//! 
//! The module also provides RT-Thread compatible memory management APIs.

// 从rtdef module导入类型
use crate::rtdef::{
    rt_err_t, rt_size_t, 
    RT_EOK, RT_TRUE, RT_FALSE, RT_ALIGN_SIZE,
};

use core::{mem, ptr}; // mem module，作用是提供内存相关的函数和类型，ptr module，作用是提供指针相关的函数和类型

use core::sync::atomic::{AtomicBool, Ordering}; // 原子布尔类型，支持线程安全的布尔操作，常用于线程安全的标志状态；Ordering 指定原子操作的内存顺序，确保内存可见性和有序性

// 此处应该导入中断控制函数，但由于rthw模块尚未实现，我们创建临时空实现
// 实际使用时需要替换为真实的中断控制函数
#[inline]
fn rt_hw_interrupt_disable() -> usize {
    // 临时空实现，返回一个假的中断状态
    0
}

#[inline]
fn rt_hw_interrupt_enable(_level: usize) {
    // 临时空实现，不做任何操作
}

// Re-export from the original file
const HEAP_START: usize = 0x10000000;
const HEAP_SIZE: usize = 1024 * 1024; // 1MB

// 内存管理的算法常量
const HEAP_MAGIC: u32 = 0x1ea0; // magic number是用于识别堆是否初始化或已销毁的标志

// 以下两个常量用于不同平台上的不同指针宽度
#[cfg(target_pointer_width = "64")]
const MIN_SIZE: usize = 24; // 表示最小内存块大小
#[cfg(target_pointer_width = "32")]
const MIN_SIZE: usize = 12; // 表示最小内存块大小


// 计算对齐后的最小内存块大小，本质是向上对齐
const MIN_SIZE_ALIGNED: usize = (MIN_SIZE + RT_ALIGN_SIZE as usize - 1) & !(RT_ALIGN_SIZE as usize - 1);

/// Memory object structure
#[repr(C)]
pub struct RTMemory {
    /// 父对象
    pub parent: RTObject,
    /// 内存算法名称
    pub algorithm: &'static str, // 全局变量
    /// 内存地址
    pub address: usize,
    /// 内存大小
    pub total: usize,
    /// 内存使用大小
    pub used: usize,
    /// 最大内存使用大小
    pub max_used: usize,
}

/// Object structure
#[repr(C)]
pub struct RTObject {
    /// 用来储存对象的名称的数组
    pub name: [u8; crate::rtconfig::RT_NAME_MAX as usize],
    /// 对象的类型
    pub obj_type: u8,
    /// 对象的标志状态
    pub flag: u8,
    /// 对象的列表节点
    pub list: RTList,
}

/// 链表节点
#[repr(C)]
pub struct RTList {
    /// 下一个节点的指针
    pub next: *mut RTList,
    /// 前一个节点的指针
    pub prev: *mut RTList,
}

/// RTSmallMemItem 结构体表示一个小内存块的基本信息，主要用于管理内存池中的单个内存块
#[repr(C)]
pub struct RTSmallMemItem {
    /// 内存池指针
    pub pool_ptr: usize,
    #[cfg(target_pointer_width = "64")] // 条件编译，64位系统生效
    pub resv: u32, // 保留字段，用于对齐
    /// 下一个空闲块的指针
    pub next: usize,
    /// 前一个空闲块的指针
    pub prev: usize, // 前一个空闲块的指针
    #[cfg(feature = "mem_trace")] // 条件编译，内存跟踪生效
    #[cfg(target_pointer_width = "64")] // 条件编译，64位系统生效
    pub thread: [u8; 8], // 线程ID
    #[cfg(feature = "mem_trace")] // 条件编译，内存跟踪生效
    #[cfg(target_pointer_width = "32")] // 条件编译，32位系统生效
    pub thread: [u8; 4], // 线程ID
}

/// RTSmallMem 结构体表示一个小内存管理对象，负责管理整个小内存池的状态和信息。
#[repr(C)]
pub struct RTSmallMem {
    /// 父内存对象
    pub parent: RTMemory,
    /// 堆指针
    pub heap_ptr: *mut u8,
    /// 堆结束
    pub heap_end: *mut RTSmallMemItem,
    /// 最低空闲内存，用来找到可用的最小空闲内存块
    pub lfree: *mut RTSmallMemItem,
    /// 对齐后的内存大小
    pub mem_size_aligned: usize,
}

/// 内存管理类型
pub type RTSmemT = *mut RTMemory;

const MEM_MASK: usize = 0xfffffffe; // 用于内存管理中的位操作，帮助判断内存块的状态

/// 检查小内存块是否被使用
#[inline]
fn mem_is_used(mem: *const RTSmallMemItem) -> bool {
    unsafe { ((*mem).pool_ptr & !MEM_MASK) != 0 }
}

/// 从内存项获取内存池
#[inline]
fn mem_pool(mem: *const RTSmallMemItem) -> *mut RTSmallMem {
    unsafe { ((*mem).pool_ptr & MEM_MASK) as *mut RTSmallMem }
}

/// 获取内存大小，传入的参数表示 内存堆 的状态和 内存块 的状态
#[inline] 
fn mem_size(heap: *const RTSmallMem, mem: *const RTSmallMemItem) -> usize {
    unsafe {
        let heap_ptr = (*heap).heap_ptr as usize;
        let mem_ptr = mem as usize;
        (*mem).next - (mem_ptr - heap_ptr) - mem::size_of::<RTSmallMemItem>()
    }
}

/// 内存项大小对齐
const SIZEOF_STRUCT_MEM: usize = (mem::size_of::<RTSmallMemItem>() + RT_ALIGN_SIZE as usize - 1) 
                                & !(RT_ALIGN_SIZE as usize - 1);

/// 设置内存块为使用，
#[inline]
fn mem_used(small_mem: *const RTSmallMem) -> usize {
    (small_mem as usize & MEM_MASK) | 0x1
}

/// 设置内存块为释放
#[inline]
fn mem_freed(small_mem: *const RTSmallMem) -> usize {
    (small_mem as usize & MEM_MASK) | 0x0
}

/// 合并相邻的空闲内存块
fn plug_holes(m: *mut RTSmallMem, mem: *mut RTSmallMemItem) {
    unsafe {
        let heap_ptr = (*m).heap_ptr;
        let heap_end = (*m).heap_end;
        
        // 确保mem在有效范围内
        assert!((mem as usize) >= (heap_ptr as usize));
        assert!((mem as usize) < (heap_end as usize));
        
        // 向前插洞
        let nmem_ptr = heap_ptr.add((*mem).next) as *mut RTSmallMemItem;
        if (mem as usize) != (nmem_ptr as usize) && !mem_is_used(nmem_ptr) && ((nmem_ptr as usize) != (heap_end as usize)) {
            // 如果mem->next是未使用的，并且不是堆的结束，则合并mem和mem->next
            if ((*m).lfree as usize) == (nmem_ptr as usize) {
                (*m).lfree = mem;
            }
            (*nmem_ptr).pool_ptr = 0;
            (*mem).next = (*nmem_ptr).next;
            
            let next_mem = heap_ptr.add((*nmem_ptr).next) as *mut RTSmallMemItem;
            (*next_mem).prev = (mem as *mut u8).offset_from(heap_ptr) as usize;
        }
        
        // 向后插洞
        let pmem_ptr = heap_ptr.add((*mem).prev) as *mut RTSmallMemItem;
        if (pmem_ptr as usize) != (mem as usize) && !mem_is_used(pmem_ptr) {
            // 如果mem->prev是未使用的，则合并mem和mem->prev
            if ((*m).lfree as usize) == (mem as usize) {
                (*m).lfree = pmem_ptr;
            }
            (*mem).pool_ptr = 0;
            (*pmem_ptr).next = (*mem).next;
            
            let next_mem = heap_ptr.add((*mem).next) as *mut RTSmallMemItem;
            (*next_mem).prev = (pmem_ptr as *mut u8).offset_from(heap_ptr) as usize;
        }
    }
}

/// 初始化小内存管理
pub fn rt_smem_init(name: &str, begin_addr: *mut u8, size: usize) -> RTSmemT {
    unsafe {
        // 对齐起始地址
        let small_mem_ptr = (begin_addr as usize + RT_ALIGN_SIZE as usize - 1) 
                            & !(RT_ALIGN_SIZE as usize - 1);
        let small_mem = small_mem_ptr as *mut RTSmallMem;
        
        let start_addr = small_mem_ptr + mem::size_of::<RTSmallMem>();
        let begin_align = (start_addr + RT_ALIGN_SIZE as usize - 1) 
                           & !(RT_ALIGN_SIZE as usize - 1);
        let end_align = (begin_addr as usize + size) & !(RT_ALIGN_SIZE as usize - 1);
        
        // 检查对齐和大小
        if end_align <= (2 * SIZEOF_STRUCT_MEM) || 
           (end_align - 2 * SIZEOF_STRUCT_MEM) < begin_align {
            return ptr::null_mut();
        }
        
        // 计算对齐后的内存大小
        let mem_size = end_align - begin_align - 2 * SIZEOF_STRUCT_MEM;
        
        // 初始化小内存对象
        ptr::write_bytes(small_mem, 0, 1);
        
        // Initialize memory parent object
        rt_object_init(&mut (*small_mem).parent.parent, RT_OBJECT_CLASS_MEMORY, name);
        (*small_mem).parent.algorithm = "small";
        (*small_mem).parent.address = begin_align;
        (*small_mem).parent.total = mem_size;
        (*small_mem).mem_size_aligned = mem_size;
        
        // Point to begin address of heap
        (*small_mem).heap_ptr = begin_align as *mut u8;
        
        // Initialize the start of the heap
        let mem = (*small_mem).heap_ptr as *mut RTSmallMemItem;
        (*mem).pool_ptr = mem_freed(small_mem);
        (*mem).next = (*small_mem).mem_size_aligned + SIZEOF_STRUCT_MEM;
        (*mem).prev = 0;
        
        #[cfg(feature = "mem_trace")]
        rt_smem_setname(mem, "INIT");
        
        // Initialize the end of the heap
        (*small_mem).heap_end = ((*small_mem).heap_ptr.add((*mem).next)) as *mut RTSmallMemItem;
        (*(*small_mem).heap_end).pool_ptr = mem_used(small_mem);
        (*(*small_mem).heap_end).next = (*small_mem).mem_size_aligned + SIZEOF_STRUCT_MEM;
        (*(*small_mem).heap_end).prev = (*small_mem).mem_size_aligned + SIZEOF_STRUCT_MEM;
        
        #[cfg(feature = "mem_trace")]
        rt_smem_setname((*small_mem).heap_end, "INIT");
        
        // Initialize the lowest-free pointer to the start of the heap
        (*small_mem).lfree = mem;
        
        &mut (*small_mem).parent
    }
}

/// Detach a small memory block
pub fn rt_smem_detach(m: RTSmemT) -> rt_err_t {
    // 修改为正确的对象引用
    unsafe {
        rt_object_detach(&mut (*m).parent);
    }
    RT_EOK
}

/// Allocate memory from the small memory heap
pub fn rt_smem_alloc(m: RTSmemT, size: rt_size_t) -> *mut u8 {
    if m.is_null() || size == 0 {
        return ptr::null_mut();
    }
    
    // 禁用中断，保证内存分配的原子性
    let level = rt_hw_interrupt_disable();
    
    unsafe {
        let small_mem = m as *mut RTSmallMem;
        let size_aligned = (size + RT_ALIGN_SIZE as usize - 1) & !(RT_ALIGN_SIZE as usize - 1);
        
        // Size too small, adjust to MIN_SIZE_ALIGNED
        let size_needed = if size_aligned < MIN_SIZE_ALIGNED {
            MIN_SIZE_ALIGNED
        } else {
            size_aligned
        };
        
        // Size too large, no memory
        if size_needed > (*small_mem).mem_size_aligned {
            rt_hw_interrupt_enable(level);
            return ptr::null_mut();
        }
        
        // Start with lowest free memory block
        let mut mem = (*small_mem).lfree;
        let mut ptr = (mem as *mut u8).offset_from((*small_mem).heap_ptr) as usize;
        let mut found = RT_FALSE;
        
        // Scan through the free list looking for a block that's big enough
        while !found {
            // Get current memory block
            mem = ((*small_mem).heap_ptr.add(ptr)) as *mut RTSmallMemItem;
            
            // Check if block is free and large enough
            if !mem_is_used(mem) && mem_size(small_mem, mem) >= size_needed {
                found = RT_TRUE;
            }
            
            // Move to next block if not found
            if !found {
                if (*mem).next >= (*small_mem).mem_size_aligned + SIZEOF_STRUCT_MEM {
                    // End of list, no suitable memory found
                    rt_hw_interrupt_enable(level);
                    return ptr::null_mut();
                }
                ptr = (*mem).next;
            }
        }
        
        // Block found, check if we need to split it
        if mem_size(small_mem, mem) >= size_needed + SIZEOF_STRUCT_MEM + MIN_SIZE_ALIGNED {
            // Split block
            let ptr2 = ptr + SIZEOF_STRUCT_MEM + size_needed;
            let mem2 = ((*small_mem).heap_ptr.add(ptr2)) as *mut RTSmallMemItem;
            
            // Set next pointer of new block
            (*mem2).next = (*mem).next;
            
            // Set next pointer of current block to point to new block
            (*mem).next = ptr2;
            
            // Set prev pointer of new block's next to point to new block
            let next_of_mem2 = ((*small_mem).heap_ptr.add((*mem2).next)) as *mut RTSmallMemItem;
            (*next_of_mem2).prev = ptr2;
            
            // Set prev pointer of new block to point to current block
            (*mem2).prev = ptr;
            
            // Set memory pool pointer of new block
            (*mem2).pool_ptr = mem_freed(small_mem);
            
            #[cfg(feature = "mem_trace")]
            rt_smem_setname(mem2, "SPLIT");
        }
        
        // Mark block as used
        (*mem).pool_ptr = mem_used(small_mem);
        
        // Update lowest free block pointer if necessary
        if (*small_mem).lfree == mem {
            // Find next free block
            while ptr < (*small_mem).mem_size_aligned + SIZEOF_STRUCT_MEM && 
                  mem_is_used(((*small_mem).heap_ptr.add(ptr)) as *mut RTSmallMemItem) {
                ptr = (*(((*small_mem).heap_ptr.add(ptr)) as *mut RTSmallMemItem)).next;
            }
            
            if ptr >= (*small_mem).mem_size_aligned + SIZEOF_STRUCT_MEM {
                // No free blocks left!
                (*small_mem).lfree = (*small_mem).heap_end;
            } else {
                (*small_mem).lfree = ((*small_mem).heap_ptr.add(ptr)) as *mut RTSmallMemItem;
            }
        }
        
        // Update statistics
        (*m).used += mem_size(small_mem, mem);
        if (*m).max_used < (*m).used {
            (*m).max_used = (*m).used;
        }
        
        // 重新启用中断
        rt_hw_interrupt_enable(level);
        
        // Return pointer to allocated memory (after the struct)
        (mem as *mut u8).add(SIZEOF_STRUCT_MEM)
    }
}

/// Reallocate memory from small memory heap
pub fn rt_smem_realloc(m: RTSmemT, rmem: *mut u8, newsize: rt_size_t) -> *mut u8 {
    if m.is_null() {
        return ptr::null_mut();
    }
    
    if rmem.is_null() {
        return rt_smem_alloc(m, newsize);
    }
    
    if newsize == 0 {
        rt_smem_free(rmem);
        return ptr::null_mut();
    }
    
    // 禁用中断，保证内存重分配的原子性
    let level = rt_hw_interrupt_disable();
    
    unsafe {
        let small_mem = m as *mut RTSmallMem;
        
        // Get memory block header
        let mem = (rmem as *mut u8).sub(SIZEOF_STRUCT_MEM) as *mut RTSmallMemItem;
        
        // Check that the memory belongs to this heap
        if mem_pool(mem) != small_mem {
            rt_hw_interrupt_enable(level);
            return ptr::null_mut();
        }
        
        // Align new size
        let size_aligned = (newsize + RT_ALIGN_SIZE as usize - 1) & !(RT_ALIGN_SIZE as usize - 1);
        let size_needed = if size_aligned < MIN_SIZE_ALIGNED {
            MIN_SIZE_ALIGNED
        } else {
            size_aligned
        };
        
        // Current memory block size
        let mem_size_cur = mem_size(small_mem, mem);
        
        // Simple case: new size fits in current block
        if mem_size_cur >= size_needed {
            // Check if we can split this block
            if mem_size_cur >= size_needed + SIZEOF_STRUCT_MEM + MIN_SIZE_ALIGNED {
                // Split block
                let ptr = (mem as *mut u8).offset_from((*small_mem).heap_ptr) as usize;
                let ptr2 = ptr + SIZEOF_STRUCT_MEM + size_needed;
                let mem2 = ((*small_mem).heap_ptr.add(ptr2)) as *mut RTSmallMemItem;
                
                // Set up new block
                (*mem2).pool_ptr = mem_freed(small_mem);
                (*mem2).next = (*mem).next;
                (*mem2).prev = ptr;
                
                // Update next block's prev pointer
                let next_of_mem2 = ((*small_mem).heap_ptr.add((*mem2).next)) as *mut RTSmallMemItem;
                (*next_of_mem2).prev = ptr2;
                
                // Update current block's next pointer
                (*mem).next = ptr2;
                
                // Insert new block in free list and merge if possible
                plug_holes(small_mem, mem2);
                
                // Update statistics
                (*m).used -= mem_size_cur - mem_size(small_mem, mem);
            }
            
            rt_hw_interrupt_enable(level);
            return rmem;
        } else {
            // 要启用中断，因为下面的调用可能会长时间运行或递归调用
            rt_hw_interrupt_enable(level);
            
            // New size doesn't fit, allocate new block
            let new_mem = rt_smem_alloc(m, newsize);
            if !new_mem.is_null() {
                // Copy old data to new location
                core::ptr::copy_nonoverlapping(rmem, new_mem, core::cmp::min(newsize, mem_size_cur));
                // Free old memory
                rt_smem_free(rmem);
            }
            return new_mem;
        }
    }
}

/// Free memory block
pub fn rt_smem_free(rmem: *mut u8) {
    if rmem.is_null() {
        return;
    }
    
    // 禁用中断，保证内存释放的原子性
    let level = rt_hw_interrupt_disable();
    
    unsafe {
        // Get block header
        let mem = (rmem as *mut u8).sub(SIZEOF_STRUCT_MEM) as *mut RTSmallMemItem;
        
        // Get memory pool from header
        let small_mem = mem_pool(mem);
        if small_mem.is_null() {
            rt_hw_interrupt_enable(level);
            return;
        }
        
        // Validate memory block - 修改debug_assert
        debug_assert!((mem as usize) >= ((*small_mem).heap_ptr as usize));
        debug_assert!((mem as usize) < ((*small_mem).heap_end as usize));
        debug_assert!(mem_is_used(mem));
        
        // Update statistics
        (*small_mem).parent.used -= mem_size(small_mem, mem);
        
        // Mark as free
        (*mem).pool_ptr = mem_freed(small_mem);
        
        // If lowest free is higher than this block, update it
        if (*small_mem).lfree > mem {
            (*small_mem).lfree = mem;
        }
        
        // Merge with adjacent free blocks
        plug_holes(small_mem, mem);
        
        // 重新启用中断
        rt_hw_interrupt_enable(level);
    }
}

/// Object class types
pub const RT_OBJECT_CLASS_MEMORY: u8 = 8;

/// Initialize an object
fn rt_object_init(object: *mut RTObject, obj_type: u8, name: &str) {
    unsafe {
        // Copy name with limited length
        ptr::write_bytes(&mut (*object).name, 0, (*object).name.len());
        let name_bytes = name.as_bytes();
        let copy_len = core::cmp::min(name_bytes.len(), (*object).name.len());
        ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            (*object).name.as_mut_ptr(),
            copy_len
        );
        
        // Set type and flags
        (*object).obj_type = obj_type;
        (*object).flag = 0;
        
        // Initialize list
        (*object).list.next = &mut (*object).list;
        (*object).list.prev = &mut (*object).list;
    }
}

/// Detach an object
fn rt_object_detach(object: *mut RTObject) {
    unsafe {
        // Just clear the object's list node
        (*object).list.next = &mut (*object).list;
        (*object).list.prev = &mut (*object).list;
    }
}

// Global allocator implementation

// Initialize heap status
static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "good_memory_allocator")]
/// Initialize heap memory for the global allocator
pub fn init_heap() {
    if !HEAP_INITIALIZED.load(Ordering::SeqCst) {
        unsafe {
            ALLOCATOR.init(HEAP_START as *mut u8, HEAP_SIZE);
        }
        HEAP_INITIALIZED.store(true, Ordering::SeqCst);
    }
}

#[cfg(feature = "buddy_system_allocator")]
/// Initialize heap memory for the global allocator
pub fn init_heap() {
    if !HEAP_INITIALIZED.load(Ordering::SeqCst) {
        unsafe {
            HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        }
        HEAP_INITIALIZED.store(true, Ordering::SeqCst);
    }
}

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

// Export public functions
pub use self::rt_smem_init as rt_mem_init;
pub use self::rt_smem_detach as rt_mem_detach;
pub use self::rt_smem_alloc as rt_malloc;
pub use self::rt_smem_realloc as rt_realloc;
pub use self::rt_smem_free as rt_free;
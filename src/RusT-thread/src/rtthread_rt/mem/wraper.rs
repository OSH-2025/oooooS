use crate::rtthread_rt::mem::mem::{rt_smem_init, rt_smem_detach, rt_smem_alloc, rt_smem_realloc, rt_smem_free};
use crate::rtthread_rt::mem::mem::RTSmemT;
use crate::rtthread_rt::mem::mem::RTSmallMem;

/// 安全的内存分配器包装
pub struct MemAllocator {
    mem: RTSmemT,
}

impl MemAllocator {
    /// 创建一个新的内存分配器
    pub fn new(name: &str, begin_addr: *mut u8, size: usize) -> Option<Self> {
        let mem = rt_smem_init(name, begin_addr, size);
        if mem.is_null() {
            None
        } else {
            Some(Self { mem })
        }
    }
    
    /// 分配指定大小的内存块
    pub fn alloc(&self, size: usize) -> Option<*mut u8> {
        let ptr = rt_smem_alloc(self.mem, size);
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
    
    /// 重新分配内存块
    pub fn realloc(&self, ptr: *mut u8, new_size: usize) -> Option<*mut u8> {
        let new_ptr = rt_smem_realloc(self.mem, ptr, new_size);
        if new_ptr.is_null() {
            None
        } else {
            Some(new_ptr)
        }
    }
    
    /// 释放内存块
    pub fn free(&self, ptr: *mut u8) {
        rt_smem_free(ptr);
    }
    
    /// 获取已使用的内存大小
    pub fn used_size(&self) -> usize {
        unsafe { (*self.mem).used }
    }
    
    /// 获取总内存大小
    pub fn total_size(&self) -> usize {
        unsafe { (*self.mem).total }
    }
}

impl Drop for MemAllocator {
    fn drop(&mut self) {
        rt_smem_detach(self.mem);
    }
}

/// 打印内存管理状态信息，用于调试
pub fn rt_mem_info(mem: RTSmemT) {
    use cortex_m_semihosting::hprintln;
    
    if mem.is_null() {
        let _ = hprintln!("Memory object is null");
        return;
    }
    
    unsafe {
        let _ = hprintln!("--- Memory Info ---");
        let _ = hprintln!("Algorithm: {}", (*mem).algorithm);
        let _ = hprintln!("Address: 0x{:x}", (*mem).address);
        let _ = hprintln!("Total size: {} bytes", (*mem).total);
        let _ = hprintln!("Used size: {} bytes", (*mem).used);
        let _ = hprintln!("Max used: {} bytes", (*mem).max_used);
        let _ = hprintln!("------------------");
        
        // 如果是小内存对象，打印更多信息
        let small_mem = mem as *mut RTSmallMem;
        let _ = hprintln!("Heap ptr: {:p}", (*small_mem).heap_ptr);
        let _ = hprintln!("Heap end: {:p}", (*small_mem).heap_end);
        let _ = hprintln!("Lowest free: {:p}", (*small_mem).lfree);
        let _ = hprintln!("------------------");
    }
}

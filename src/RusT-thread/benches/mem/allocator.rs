//! 内存分配器适配器
//! 
//! 提供统一的接口来比较不同的内存分配算法

use crate::memory::RTSmallMem;
use std::alloc::{GlobalAlloc, Layout};

/// 通用内存分配器特征
pub trait MemoryAllocator {
    /// 分配内存
    fn allocate(&mut self, size: usize) -> *mut u8;
    
    /// 释放内存
    fn deallocate(&mut self, ptr: *mut u8);
    
    /// 重分配内存
    fn reallocate(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8;
    
    /// 获取分配器名称
    fn name(&self) -> &'static str;
    
    /// 获取内存使用统计
    fn stats(&self) -> AllocatorStats;
    
    /// 重置分配器状态
    fn reset(&mut self);
}

/// 分配器统计信息
#[derive(Debug, Clone, Copy, Default)]
pub struct AllocatorStats {
    pub allocated_bytes: usize,
    pub allocated_count: usize,
    pub deallocated_count: usize,
    pub peak_allocated: usize,
    pub fragmentation_ratio: f64,
}

/// RT-Thread 小内存分配器
pub struct RTThreadAllocator {
    inner: Box<RTSmallMem>,
    stats: AllocatorStats,
}

impl RTThreadAllocator {
    pub fn new(heap_size: usize) -> Self {
        Self {
            inner: RTSmallMem::new(heap_size),
            stats: AllocatorStats::default(),
        }
    }
}

impl MemoryAllocator for RTThreadAllocator {
    fn allocate(&mut self, size: usize) -> *mut u8 {
        let ptr = self.inner.alloc(size);
        if !ptr.is_null() {
            self.stats.allocated_bytes += size;
            self.stats.allocated_count += 1;
            if self.stats.allocated_bytes > self.stats.peak_allocated {
                self.stats.peak_allocated = self.stats.allocated_bytes;
            }
        }
        ptr
    }
    
    fn deallocate(&mut self, ptr: *mut u8) {
        if !ptr.is_null() {
            self.inner.free(ptr);
            self.stats.deallocated_count += 1;
        }
    }
    
    fn reallocate(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8 {
        self.inner.realloc(ptr, new_size)
    }
    
    fn name(&self) -> &'static str {
        "RT-Thread Small Memory"
    }
    
    fn stats(&self) -> AllocatorStats {
        let mem_stats = self.inner.get_stats();
        AllocatorStats {
            allocated_bytes: mem_stats.used,
            peak_allocated: mem_stats.max_used,
            fragmentation_ratio: 1.0 - (mem_stats.used as f64 / mem_stats.total as f64),
            ..self.stats
        }
    }
    
    fn reset(&mut self) {
        let heap_size = self.inner.parent.total;
        self.inner = RTSmallMem::new(heap_size);
        self.stats = AllocatorStats::default();
    }
}

/// 标准库分配器包装器
pub struct StdAllocator {
    stats: AllocatorStats,
    allocations: std::collections::HashMap<*mut u8, usize>,
}

impl StdAllocator {
    pub fn new() -> Self {
        Self {
            stats: AllocatorStats::default(),
            allocations: std::collections::HashMap::new(),
        }
    }
}

impl Default for StdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryAllocator for StdAllocator {
    fn allocate(&mut self, size: usize) -> *mut u8 {
        if size == 0 {
            return std::ptr::null_mut();
        }
        
        let layout = Layout::from_size_align(size, std::mem::align_of::<u8>()).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        
        if !ptr.is_null() {
            self.allocations.insert(ptr, size);
            self.stats.allocated_bytes += size;
            self.stats.allocated_count += 1;
            if self.stats.allocated_bytes > self.stats.peak_allocated {
                self.stats.peak_allocated = self.stats.allocated_bytes;
            }
        }
        
        ptr
    }
    
    fn deallocate(&mut self, ptr: *mut u8) {
        if let Some(size) = self.allocations.remove(&ptr) {
            let layout = Layout::from_size_align(size, std::mem::align_of::<u8>()).unwrap();
            unsafe { std::alloc::dealloc(ptr, layout) };
            
            self.stats.deallocated_count += 1;
            if self.stats.allocated_bytes >= size {
                self.stats.allocated_bytes -= size;
            }
        }
    }
    
    fn reallocate(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8 {
        if ptr.is_null() {
            return self.allocate(new_size);
        }
        
        if new_size == 0 {
            self.deallocate(ptr);
            return std::ptr::null_mut();
        }
        
        if let Some(&old_size) = self.allocations.get(&ptr) {
            let old_layout = Layout::from_size_align(old_size, std::mem::align_of::<u8>()).unwrap();
            let new_ptr = unsafe { std::alloc::realloc(ptr, old_layout, new_size) };
            
            if !new_ptr.is_null() {
                self.allocations.remove(&ptr);
                self.allocations.insert(new_ptr, new_size);
                
                if new_size > old_size {
                    self.stats.allocated_bytes += new_size - old_size;
                } else {
                    self.stats.allocated_bytes -= old_size - new_size;
                }
                
                if self.stats.allocated_bytes > self.stats.peak_allocated {
                    self.stats.peak_allocated = self.stats.allocated_bytes;
                }
            }
            
            new_ptr
        } else {
            self.allocate(new_size)
        }
    }
    
    fn name(&self) -> &'static str {
        "Standard Library Allocator"
    }
    
    fn stats(&self) -> AllocatorStats {
        self.stats
    }
    
    fn reset(&mut self) {
        // 清理所有分配的内存
        for (&ptr, &size) in &self.allocations {
            let layout = Layout::from_size_align(size, std::mem::align_of::<u8>()).unwrap();
            unsafe { std::alloc::dealloc(ptr, layout) };
        }
        
        self.allocations.clear();
        self.stats = AllocatorStats::default();
    }
}

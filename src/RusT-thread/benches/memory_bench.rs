//! RT-Thread 内存管理核心实现
//! 
//! 基于 RT-Thread 的小内存管理算法，适配为标准库环境

use std::{mem, ptr};

/// RT-Thread 内存对齐常量
pub const RT_ALIGN_SIZE: usize = 8;

/// 内存管理的算法常量
const HEAP_MAGIC: u32 = 0x1ea0;

/// 最小内存块大小
#[cfg(target_pointer_width = "64")]
const MIN_SIZE: usize = 24;
#[cfg(target_pointer_width = "32")]
const MIN_SIZE: usize = 12;

const MIN_SIZE_ALIGNED: usize = (MIN_SIZE + RT_ALIGN_SIZE - 1) & !(RT_ALIGN_SIZE - 1);

/// 内存管理掩码
const MEM_MASK: usize = 0xfffffffe;

/// RT-Thread 内存对象
#[repr(C)]
#[derive(Debug)]
pub struct RTMemory {
    pub algorithm: &'static str,
    pub address: usize,
    pub total: usize,
    pub used: usize,
    pub max_used: usize,
}

/// RT-Thread 小内存块项
#[repr(C)]
#[derive(Debug)]
pub struct RTSmallMemItem {
    pub pool_ptr: usize,
    #[cfg(target_pointer_width = "64")]
    pub resv: u32,
    pub next: usize,
    pub prev: usize,
}

/// RT-Thread 小内存管理对象
#[repr(C)]
pub struct RTSmallMem {
    pub parent: RTMemory,
    pub heap_ptr: *mut u8,
    pub heap_end: *mut RTSmallMemItem,
    pub lfree: *mut RTSmallMemItem,
    pub mem_size_aligned: usize,
}

/// 内存项大小对齐
const SIZEOF_STRUCT_MEM: usize = (mem::size_of::<RTSmallMemItem>() + RT_ALIGN_SIZE - 1) 
                                & !(RT_ALIGN_SIZE - 1);

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

/// 获取内存大小
#[inline] 
fn mem_size(heap: *const RTSmallMem, mem: *const RTSmallMemItem) -> usize {
    unsafe {
        let heap_ptr = (*heap).heap_ptr as usize;
        let mem_ptr = mem as usize;
        (*mem).next - (mem_ptr - heap_ptr) - mem::size_of::<RTSmallMemItem>()
    }
}

/// 设置内存块为使用
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
        if (mem as usize) < (heap_ptr as usize) || (mem as usize) >= (heap_end as usize) {
            return;
        }
        
        // 向前合并
        let nmem_ptr = heap_ptr.add((*mem).next) as *mut RTSmallMemItem;
        if (mem as usize) != (nmem_ptr as usize) && 
           (nmem_ptr as usize) < (heap_end as usize) && 
           !mem_is_used(nmem_ptr) && 
           ((nmem_ptr as usize) != (heap_end as usize)) {
            
            if ((*m).lfree as usize) == (nmem_ptr as usize) {
                (*m).lfree = mem;
            }
            (*nmem_ptr).pool_ptr = 0;
            (*mem).next = (*nmem_ptr).next;
            
            let next_mem = heap_ptr.add((*nmem_ptr).next) as *mut RTSmallMemItem;
            if (next_mem as usize) < (heap_end as usize) {
                (*next_mem).prev = (mem as *mut u8).offset_from(heap_ptr) as usize;
            }
        }
        
        // 向后合并
        if (*mem).prev > 0 {
            let pmem_ptr = heap_ptr.add((*mem).prev) as *mut RTSmallMemItem;
            if (pmem_ptr as usize) != (mem as usize) && 
               (pmem_ptr as usize) >= (heap_ptr as usize) &&
               !mem_is_used(pmem_ptr) {
                
                if ((*m).lfree as usize) == (mem as usize) {
                    (*m).lfree = pmem_ptr;
                }
                (*mem).pool_ptr = 0;
                (*pmem_ptr).next = (*mem).next;
                
                let next_mem = heap_ptr.add((*mem).next) as *mut RTSmallMemItem;
                if (next_mem as usize) < (heap_end as usize) {
                    (*next_mem).prev = (*mem).prev;
                }
            }
        }
    }
}

impl RTSmallMem {
    /// 创建新的 RT-Thread 内存管理器
    pub fn new(size: usize) -> Box<Self> {
        let heap_data = vec![0u8; size].into_boxed_slice();
        let heap_ptr = Box::into_raw(heap_data) as *mut u8;
        
        let mut small_mem = Box::new(RTSmallMem {
            parent: RTMemory {
                algorithm: "small_mem",
                address: heap_ptr as usize,
                total: size,
                used: 0,
                max_used: 0,
            },
            heap_ptr,
            heap_end: ptr::null_mut(),
            lfree: ptr::null_mut(),
            mem_size_aligned: (size + RT_ALIGN_SIZE - 1) & !(RT_ALIGN_SIZE - 1),
        });
        
        // 初始化内存管理器
        unsafe {
            let mem_size_aligned = small_mem.mem_size_aligned;
            let heap_end = heap_ptr.add(mem_size_aligned) as *mut RTSmallMemItem;
            small_mem.heap_end = heap_end.sub(1);
            
            let mem = heap_ptr as *mut RTSmallMemItem;
            (*mem).pool_ptr = mem_freed(small_mem.as_ref() as *const RTSmallMem);
            (*mem).next = mem_size_aligned - SIZEOF_STRUCT_MEM;
            (*mem).prev = 0;
            
            // 设置结束标记
            let end_mem = heap_ptr.add((*mem).next) as *mut RTSmallMemItem;
            (*end_mem).pool_ptr = mem_used(small_mem.as_ref() as *const RTSmallMem);
            (*end_mem).next = mem_size_aligned - SIZEOF_STRUCT_MEM;
            (*end_mem).prev = (*mem).next;
            
            small_mem.lfree = mem;
        }
        
        small_mem
    }
    
    /// 分配内存
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        if size == 0 {
            return ptr::null_mut();
        }
        
        let size = if size < MIN_SIZE_ALIGNED {
            MIN_SIZE_ALIGNED
        } else {
            (size + RT_ALIGN_SIZE - 1) & !(RT_ALIGN_SIZE - 1)
        };
        
        if size > self.mem_size_aligned {
            return ptr::null_mut();
        }
        
        unsafe {
            let mut mem = self.lfree;
            
            // 寻找合适的内存块
            while (mem as usize) < (self.heap_end as usize) {
                if !mem_is_used(mem) && mem_size(self as *const RTSmallMem, mem) >= size {
                    break;
                }
                mem = self.heap_ptr.add((*mem).next) as *mut RTSmallMemItem;
            }
            
            if (mem as usize) >= (self.heap_end as usize) {
                return ptr::null_mut();
            }
            
            // 分割内存块
            let mem_size_val = mem_size(self as *const RTSmallMem, mem);
            if mem_size_val >= size + SIZEOF_STRUCT_MEM + MIN_SIZE_ALIGNED {
                let ptr_offset = (mem as *mut u8).offset_from(self.heap_ptr) as usize + size + SIZEOF_STRUCT_MEM;
                let mem2 = self.heap_ptr.add(ptr_offset) as *mut RTSmallMemItem;
                
                (*mem2).pool_ptr = mem_freed(self as *const RTSmallMem);
                (*mem2).next = (*mem).next;
                (*mem2).prev = (mem as *mut u8).offset_from(self.heap_ptr) as usize;
                (*mem).next = ptr_offset;
                
                if ((*mem2).next as usize) < (self.heap_end as usize) {
                    let next_mem = self.heap_ptr.add((*mem2).next) as *mut RTSmallMemItem;
                    (*next_mem).prev = ptr_offset;
                }
            }
            
            // 标记为已使用
            (*mem).pool_ptr = mem_used(self as *const RTSmallMem);
            
            // 更新lfree
            if mem == self.lfree {
                while self.lfree < self.heap_end && mem_is_used(self.lfree) {
                    self.lfree = self.heap_ptr.add((*self.lfree).next) as *mut RTSmallMemItem;
                }
            }
            
            // 更新统计信息
            self.parent.used += size;
            if self.parent.used > self.parent.max_used {
                self.parent.max_used = self.parent.used;
            }
            
            (mem as *mut u8).add(SIZEOF_STRUCT_MEM)
        }
    }
    
    /// 释放内存
    pub fn free(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            let mem = ptr.sub(SIZEOF_STRUCT_MEM) as *mut RTSmallMemItem;
            
            // 验证内存块
            if (mem as usize) < (self.heap_ptr as usize) || 
               (mem as usize) >= (self.heap_end as usize) ||
               !mem_is_used(mem) {
                return;
            }
            
            // 更新统计信息
            let size = mem_size(self as *const RTSmallMem, mem);
            if self.parent.used >= size {
                self.parent.used -= size;
            }
            
            // 标记为已释放
            (*mem).pool_ptr = mem_freed(self as *const RTSmallMem);
            
            // 更新lfree
            if mem < self.lfree {
                self.lfree = mem;
            }
            
            // 合并相邻块
            plug_holes(self as *mut RTSmallMem, mem);
        }
    }
    
    /// 重分配内存
    pub fn realloc(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8 {
        if ptr.is_null() {
            return self.alloc(new_size);
        }
        
        if new_size == 0 {
            self.free(ptr);
            return ptr::null_mut();
        }
        
        let new_ptr = self.alloc(new_size);
        if !new_ptr.is_null() {
            unsafe {
                let mem = ptr.sub(SIZEOF_STRUCT_MEM) as *mut RTSmallMemItem;
                let old_size = mem_size(self as *const RTSmallMem, mem);
                let copy_size = if old_size < new_size { old_size } else { new_size };
                ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
            }
            self.free(ptr);
        }
        
        new_ptr
    }
    
    /// 获取内存使用统计
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            used: self.parent.used,
            max_used: self.parent.max_used,
            total: self.parent.total,
        }
    }
}

impl Drop for RTSmallMem {
    fn drop(&mut self) {
        if !self.heap_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(std::slice::from_raw_parts_mut(
                    self.heap_ptr, 
                    self.parent.total
                ));
            }
        }
    }
}

/// 内存使用统计
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub used: usize,
    pub max_used: usize,
    pub total: usize,
}

impl MemoryStats {
    pub fn utilization(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.used as f64) / (self.total as f64) * 100.0
        }
    }
    
    pub fn peak_utilization(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.max_used as f64) / (self.total as f64) * 100.0
        }
    }
}

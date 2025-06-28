// 文件：src/RusT-thread/benches/memory_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use std::mem;

/// RT-Thread内存对齐常量
const RT_ALIGN_SIZE: usize = 8;

/// 内存管理的算法常量
const HEAP_MAGIC: u32 = 0x1ea0;

/// 最小内存块大小（适配标准环境）
const MIN_SIZE: usize = 24;
const MIN_SIZE_ALIGNED: usize = (MIN_SIZE + RT_ALIGN_SIZE - 1) & !(RT_ALIGN_SIZE - 1);

/// 内存管理掩码
const MEM_MASK: usize = 0xfffffffe;

/// RT-Thread 内存对象（简化版）
#[repr(C)]
pub struct RTMemory {
    pub algorithm: &'static str,
    pub address: usize,
    pub total: usize,
    pub used: usize,
    pub max_used: usize,
}

/// RT-Thread 小内存块项（简化版）
#[repr(C)]
pub struct RTSmallMemItem {
    pub pool_ptr: usize,
    pub next: usize,
    pub prev: usize,
}

/// RT-Thread 小内存管理对象（简化版）
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

/// 合并相邻的空闲内存块（从RT-Thread代码适配）
fn plug_holes(m: *mut RTSmallMem, mem: *mut RTSmallMemItem) {
    unsafe {
        let heap_ptr = (*m).heap_ptr;
        let heap_end = (*m).heap_end;
        
        // 确保mem在有效范围内
        if (mem as usize) < (heap_ptr as usize) || (mem as usize) >= (heap_end as usize) {
            return;
        }
        
        // 向前插洞
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
        
        // 向后插洞
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

/// RT-Thread内存适配器
pub struct RTThreadMemAdapter {
    small_mem: Box<RTSmallMem>,
    heap_data: Vec<u8>,
}

impl RTThreadMemAdapter {
    /// 创建新的RT-Thread内存管理器
    pub fn new(size: usize) -> Self {
        let mut heap_data = vec![0u8; size];
        let heap_ptr = heap_data.as_mut_ptr();
        
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
            (*mem).next = mem_size_aligned;
            (*mem).prev = 0;
            
            small_mem.lfree = mem;
        }
        
        RTThreadMemAdapter {
            small_mem,
            heap_data,
        }
    }
    
    /// 分配内存（基于RT-Thread算法）
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        if size == 0 {
            return ptr::null_mut();
        }
        
        let size = if size < MIN_SIZE_ALIGNED {
            MIN_SIZE_ALIGNED
        } else {
            (size + SIZEOF_STRUCT_MEM + RT_ALIGN_SIZE - 1) & !(RT_ALIGN_SIZE - 1)
        };
        
        if size > self.small_mem.mem_size_aligned {
            return ptr::null_mut();
        }
        
        unsafe {
            let mut mem = self.small_mem.lfree;
            let mut mem2 = ptr::null_mut();
            
            // 寻找合适的内存块
            while (mem as usize) < (self.small_mem.heap_end as usize) {
                if !mem_is_used(mem) && mem_size(self.small_mem.as_ref(), mem) >= size {
                    break;
                }
                mem = self.small_mem.heap_ptr.add((*mem).next) as *mut RTSmallMemItem;
            }
            
            if (mem as usize) >= (self.small_mem.heap_end as usize) {
                return ptr::null_mut();
            }
            
            // 分割内存块
            let mem_size_val = mem_size(self.small_mem.as_ref(), mem);
            if mem_size_val >= size + SIZEOF_STRUCT_MEM + MIN_SIZE_ALIGNED {
                let ptr_offset = (mem as *mut u8).offset_from(self.small_mem.heap_ptr) as usize + size + SIZEOF_STRUCT_MEM;
                mem2 = self.small_mem.heap_ptr.add(ptr_offset) as *mut RTSmallMemItem;
                
                (*mem2).pool_ptr = mem_freed(self.small_mem.as_ref());
                (*mem2).next = (*mem).next;
                (*mem2).prev = (mem as *mut u8).offset_from(self.small_mem.heap_ptr) as usize;
                (*mem).next = ptr_offset;
                
                if ((*mem2).next as usize) < (self.small_mem.heap_end as usize) {
                    let next_mem = self.small_mem.heap_ptr.add((*mem2).next) as *mut RTSmallMemItem;
                    (*next_mem).prev = ptr_offset;
                }
            }
            
            // 标记为已使用
            (*mem).pool_ptr = mem_used(self.small_mem.as_ref());
            
            // 更新lfree
            if mem == self.small_mem.lfree {
                while self.small_mem.lfree < self.small_mem.heap_end && mem_is_used(self.small_mem.lfree) {
                    self.small_mem.lfree = self.small_mem.heap_ptr.add((*self.small_mem.lfree).next) as *mut RTSmallMemItem;
                }
            }
            
            // 更新统计信息
            self.small_mem.parent.used += size;
            if self.small_mem.parent.used > self.small_mem.parent.max_used {
                self.small_mem.parent.max_used = self.small_mem.parent.used;
            }
            
            (mem as *mut u8).add(SIZEOF_STRUCT_MEM)
        }
    }
    
    /// 释放内存（基于RT-Thread算法）
    pub fn free(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            let mem = ptr.sub(SIZEOF_STRUCT_MEM) as *mut RTSmallMemItem;
            
            // 验证内存块
            if (mem as usize) < (self.small_mem.heap_ptr as usize) || 
               (mem as usize) >= (self.small_mem.heap_end as usize) ||
               !mem_is_used(mem) {
                return;
            }
            
            // 更新统计信息
            let size = mem_size(self.small_mem.as_ref(), mem);
            self.small_mem.parent.used -= size;
            
            // 标记为已释放
            (*mem).pool_ptr = mem_freed(self.small_mem.as_ref());
            
            // 合并相邻块
            if mem < self.small_mem.lfree {
                self.small_mem.lfree = mem;
            }
            
            plug_holes(self.small_mem.as_mut(), mem);
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
                let old_size = mem_size(self.small_mem.as_ref(), mem);
                let copy_size = if old_size < new_size { old_size } else { new_size };
                ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
            }
            self.free(ptr);
        }
        
        new_ptr
    }
    
    /// 获取内存使用统计
    pub fn get_used(&self) -> usize {
        self.small_mem.parent.used
    }
    
    /// 获取最大内存使用
    pub fn get_max_used(&self) -> usize {
        self.small_mem.parent.max_used
    }
    
    /// 获取总内存大小
    pub fn get_total(&self) -> usize {
        self.small_mem.parent.total
    }
}

/// 基准测试函数
fn bench_memory_manager_creation(c: &mut Criterion) {
    c.bench_function("rtthread_memory_manager_creation", |b| {
        b.iter(|| {
            RTThreadMemAdapter::new(64 * 1024)
        });
    });
}

fn bench_single_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtthread_single_allocation");
    
    for size in [16, 32, 64, 128, 256, 512, 1024, 2048].iter() {
        group.bench_with_input(BenchmarkId::new("alloc", size), size, |b, &size| {
            let mut mem = RTThreadMemAdapter::new(64 * 1024);
            b.iter(|| {
                let ptr = mem.alloc(size);
                if !ptr.is_null() {
                    mem.free(ptr);
                }
            });
        });
    }
    group.finish();
}

fn bench_multiple_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtthread_multiple_allocations");
    
    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("alloc_many", count), count, |b, &count| {
            b.iter(|| {
                let mut mem = RTThreadMemAdapter::new(1024 * 1024);
                let mut ptrs = Vec::new();
                
                for _ in 0..count {
                    let ptr = mem.alloc(64);
                    if !ptr.is_null() {
                        ptrs.push(ptr);
                    }
                }
                
                for ptr in ptrs {
                    mem.free(ptr);
                }
            });
        });
    }
    group.finish();
}

fn bench_alloc_free_cycles(c: &mut Criterion) {
    c.bench_function("rtthread_alloc_free_cycles", |b| {
        b.iter(|| {
            let mut mem = RTThreadMemAdapter::new(64 * 1024);
            for _ in 0..100 {
                let ptr = mem.alloc(128);
                if !ptr.is_null() {
                    mem.free(ptr);
                }
            }
        });
    });
}

fn bench_realloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtthread_realloc");
    
    for &(old_size, new_size) in [(64, 128), (128, 64), (64, 256), (256, 64)].iter() {
        group.bench_with_input(
            BenchmarkId::new("realloc", format!("{}to{}", old_size, new_size)), 
            &(old_size, new_size), 
            |b, &(old_size, new_size)| {
                let mut mem = RTThreadMemAdapter::new(64 * 1024);
                b.iter(|| {
                    let ptr = mem.alloc(old_size);
                    if !ptr.is_null() {
                        let new_ptr = mem.realloc(ptr, new_size);
                        if !new_ptr.is_null() {
                            mem.free(new_ptr);
                        }
                    }
                });
            }
        );
    }
    group.finish();
}

fn bench_memory_usage_stats(c: &mut Criterion) {
    c.bench_function("rtthread_memory_usage_stats", |b| {
        let mut mem = RTThreadMemAdapter::new(64 * 1024);
        let mut ptrs = Vec::new();
        
        // 分配一些内存
        for _ in 0..10 {
            let ptr = mem.alloc(1024);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
        
        b.iter(|| {
            let _used = mem.get_used();
            let _max_used = mem.get_max_used();
            let _total = mem.get_total();
        });
        
        // 清理
        for ptr in ptrs {
            mem.free(ptr);
        }
    });
}

fn bench_fragmentation_handling(c: &mut Criterion) {
    c.bench_function("rtthread_fragmentation_handling", |b| {
        b.iter(|| {
            let mut mem = RTThreadMemAdapter::new(64 * 1024);
            let mut ptrs = Vec::new();
            
            // 分配许多小块
            for _ in 0..100 {
                let ptr = mem.alloc(64);
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 释放每隔一个块，创建碎片
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 0 {
                    mem.free(ptr);
                }
            }
            
            // 尝试分配更大的块
            let large_ptr = mem.alloc(1024);
            
            // 清理剩余内存
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 1 {
                    mem.free(ptr);
                }
            }
            
            if !large_ptr.is_null() {
                mem.free(large_ptr);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_memory_manager_creation,
    bench_single_allocation,
    bench_multiple_allocations,
    bench_alloc_free_cycles,
    bench_realloc,
    bench_memory_usage_stats,
    bench_fragmentation_handling
);

criterion_main!(benches);
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rt_thread_mem::{RTThreadAllocator, StdAllocator, MemoryAllocator};

/// 对比不同分配器在相同工作负载下的性能
fn bench_workload_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("workload_comparison");
    
    // 工作负载1：大量小分配
    group.bench_function("small_allocations_rtthread", |b| {
        b.iter(|| {
            let mut allocator = RTThreadAllocator::new(1024 * 1024);
            let mut ptrs = Vec::new();
            
            // 分配1000个小块
            for _ in 0..1000 {
                let size = black_box(32 + (ptrs.len() % 64)); // 32-96字节
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 随机释放一半
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 0 {
                    allocator.deallocate(ptr);
                }
            }
            
            // 再分配一些
            for _ in 0..500 {
                let size = black_box(16 + (ptrs.len() % 32));
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 清理剩余内存
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 1 {
                    allocator.deallocate(ptr);
                }
            }
        });
    });
    
    group.bench_function("small_allocations_std", |b| {
        b.iter(|| {
            let mut allocator = StdAllocator::new();
            let mut ptrs = Vec::new();
            
            // 分配1000个小块
            for _ in 0..1000 {
                let size = black_box(32 + (ptrs.len() % 64)); // 32-96字节
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 随机释放一半
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 0 {
                    allocator.deallocate(ptr);
                }
            }
            
            // 再分配一些
            for _ in 0..500 {
                let size = black_box(16 + (ptrs.len() % 32));
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 清理剩余内存
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 1 {
                    allocator.deallocate(ptr);
                }
            }
        });
    });
    
    // 工作负载2：混合大小分配
    group.bench_function("mixed_allocations_rtthread", |b| {
        b.iter(|| {
            let mut allocator = RTThreadAllocator::new(2 * 1024 * 1024);
            let mut small_ptrs = Vec::new();
            let mut medium_ptrs = Vec::new();
            let mut large_ptrs = Vec::new();
            
            // 分配不同大小的内存块
            for i in 0..100 {
                // 小块 (16-128字节)
                let small_ptr = allocator.allocate(black_box(16 + (i % 112)));
                if !small_ptr.is_null() {
                    small_ptrs.push(small_ptr);
                }
                
                // 中等块 (1KB-8KB)
                if i % 10 == 0 {
                    let medium_ptr = allocator.allocate(black_box(1024 + (i % 7168)));
                    if !medium_ptr.is_null() {
                        medium_ptrs.push(medium_ptr);
                    }
                }
                
                // 大块 (64KB)
                if i % 50 == 0 {
                    let large_ptr = allocator.allocate(black_box(64 * 1024));
                    if !large_ptr.is_null() {
                        large_ptrs.push(large_ptr);
                    }
                }
            }
            
            // 释放所有内存
            for ptr in small_ptrs {
                allocator.deallocate(ptr);
            }
            for ptr in medium_ptrs {
                allocator.deallocate(ptr);
            }
            for ptr in large_ptrs {
                allocator.deallocate(ptr);
            }
        });
    });
    
    group.bench_function("mixed_allocations_std", |b| {
        b.iter(|| {
            let mut allocator = StdAllocator::new();
            let mut small_ptrs = Vec::new();
            let mut medium_ptrs = Vec::new();
            let mut large_ptrs = Vec::new();
            
            // 分配不同大小的内存块
            for i in 0..100 {
                // 小块 (16-128字节)
                let small_ptr = allocator.allocate(black_box(16 + (i % 112)));
                if !small_ptr.is_null() {
                    small_ptrs.push(small_ptr);
                }
                
                // 中等块 (1KB-8KB)
                if i % 10 == 0 {
                    let medium_ptr = allocator.allocate(black_box(1024 + (i % 7168)));
                    if !medium_ptr.is_null() {
                        medium_ptrs.push(medium_ptr);
                    }
                }
                
                // 大块 (64KB)
                if i % 50 == 0 {
                    let large_ptr = allocator.allocate(black_box(64 * 1024));
                    if !large_ptr.is_null() {
                        large_ptrs.push(large_ptr);
                    }
                }
            }
            
            // 释放所有内存
            for ptr in small_ptrs {
                allocator.deallocate(ptr);
            }
            for ptr in medium_ptrs {
                allocator.deallocate(ptr);
            }
            for ptr in large_ptrs {
                allocator.deallocate(ptr);
            }
        });
    });
    
    group.finish();
}

/// 测试不同分配器的内存利用效率
fn bench_memory_utilization(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_utilization");
    
    let heap_sizes = [64 * 1024, 256 * 1024, 1024 * 1024];
    
    for &heap_size in &heap_sizes {
        group.bench_with_input(
            BenchmarkId::new("rtthread_utilization", heap_size),
            &heap_size,
            |b, &heap_size| {
                b.iter(|| {
                    let mut allocator = RTThreadAllocator::new(heap_size);
                    let mut ptrs = Vec::new();
                    let mut total_requested = 0;
                    
                    // 尝试分配直到接近内存上限
                    loop {
                        let size = black_box(64 + (ptrs.len() % 192)); // 64-256字节
                        let ptr = allocator.allocate(size);
                        if ptr.is_null() {
                            break;
                        }
                        ptrs.push(ptr);
                        total_requested += size;
                        
                        // 避免无限循环
                        if total_requested > heap_size * 8 / 10 {
                            break;
                        }
                    }
                    
                    let stats = allocator.stats();
                    let utilization = if heap_size > 0 {
                        (stats.allocated_bytes as f64) / (heap_size as f64)
                    } else {
                        0.0
                    };
                    
                    black_box(utilization);
                    
                    // 清理
                    for ptr in ptrs {
                        allocator.deallocate(ptr);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// 测试连续分配和释放的性能
fn bench_continuous_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("continuous_operations");
    
    group.bench_function("rtthread_continuous", |b| {
        let mut allocator = RTThreadAllocator::new(1024 * 1024);
        
        b.iter(|| {
            let mut ptrs = Vec::with_capacity(1000);
            
            // 连续分配
            for i in 0..1000 {
                let size = black_box(32 + (i % 96));
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 连续释放
            for ptr in ptrs {
                allocator.deallocate(ptr);
            }
        });
    });
    
    group.bench_function("std_continuous", |b| {
        let mut allocator = StdAllocator::new();
        
        b.iter(|| {
            let mut ptrs = Vec::with_capacity(1000);
            
            // 连续分配
            for i in 0..1000 {
                let size = black_box(32 + (i % 96));
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 连续释放
            for ptr in ptrs {
                allocator.deallocate(ptr);
            }
        });
    });
    
    group.finish();
}

/// 测试在低内存条件下的性能
fn bench_low_memory_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("low_memory_stress");
    
    // 使用较小的堆来模拟内存受限环境
    let small_heap_size = 8 * 1024; // 8KB
    
    group.bench_function("rtthread_stress", |b| {
        b.iter(|| {
            let mut allocator = RTThreadAllocator::new(small_heap_size);
            let mut ptrs = Vec::new();
            let mut successful_allocs = 0;
            let mut failed_allocs = 0;
            
            // 尝试分配直到内存耗尽
            for i in 0..1000 {
                let size = black_box(64 + (i % 128)); // 64-192字节
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push((ptr, size));
                    successful_allocs += 1;
                } else {
                    failed_allocs += 1;
                    
                    // 如果分配失败，释放一些内存然后重试
                    if !ptrs.is_empty() {
                        let (old_ptr, _) = ptrs.remove(0);
                        allocator.deallocate(old_ptr);
                        
                        let ptr = allocator.allocate(size);
                        if !ptr.is_null() {
                            ptrs.push((ptr, size));
                            successful_allocs += 1;
                        }
                    }
                }
            }
            
            black_box((successful_allocs, failed_allocs));
            
            // 清理
            for (ptr, _) in ptrs {
                allocator.deallocate(ptr);
            }
        });
    });
    
    group.bench_function("std_stress", |b| {
        b.iter(|| {
            let mut allocator = StdAllocator::new();
            let mut ptrs = Vec::new();
            let mut successful_allocs = 0;
            
            // 标准库分配器没有内存上限，所以我们限制分配次数
            for i in 0..100 {
                let size = black_box(64 + (i % 128));
                let ptr = allocator.allocate(size);
                if !ptr.is_null() {
                    ptrs.push((ptr, size));
                    successful_allocs += 1;
                }
            }
            
            black_box(successful_allocs);
            
            // 清理
            for (ptr, _) in ptrs {
                allocator.deallocate(ptr);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_workload_comparison,
    bench_memory_utilization,
    bench_continuous_operations,
    bench_low_memory_stress
);

criterion_main!(benches);

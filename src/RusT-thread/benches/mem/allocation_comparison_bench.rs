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
    bench_memory_utilization,use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
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
    
    /// 测试内存安全性 - 检查分配器的基本安全行为
    fn bench_memory_safety(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory_safety");
        
        // 测试空指针处理安全性
        group.bench_function("null_ptr_safety_rtthread", |b| {
            let mut allocator = RTThreadAllocator::new(64 * 1024);
            
            b.iter(|| {
                // 测试释放空指针是否安全
                allocator.deallocate(std::ptr::null_mut());
                
                // 测试分配0字节
                let ptr = allocator.allocate(black_box(0));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                }
                
                // 测试重复释放检测（应该安全处理）
                let ptr = allocator.allocate(black_box(64));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                    // 注意：重复释放在实际中是危险的，这里只是测试分配器的健壮性
                }
            });
        });
        
        group.bench_function("null_ptr_safety_std", |b| {
            let mut allocator = StdAllocator::new();
            
            b.iter(|| {
                // 测试释放空指针是否安全
                allocator.deallocate(std::ptr::null_mut());
                
                // 测试分配0字节
                let ptr = allocator.allocate(black_box(0));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                }
                
                // 测试正常分配释放
                let ptr = allocator.allocate(black_box(64));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                }
            });
        });
        
        group.finish();
    }
    
    /// 测试内存使用率 - 评估分配器的空间效率
    fn bench_memory_usage_efficiency(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory_usage_efficiency");
        
        // 测试不同分配模式下的内存使用率
        group.bench_function("usage_rate_rtthread", |b| {
            b.iter(|| {
                let heap_size = 64 * 1024; // 64KB堆
                let mut allocator = RTThreadAllocator::new(heap_size);
                let mut ptrs = Vec::new();
                let mut total_requested = 0usize;
                
                // 分配多个不同大小的块，记录实际请求的内存
                for i in 0..50 {
                    let size = 64 + (i % 64); // 64-128字节
                    let ptr = allocator.allocate(black_box(size));
                    if !ptr.is_null() {
                        ptrs.push(ptr);
                        total_requested += size;
                    }
                }
                
                // 获取分配器统计信息
                let stats = allocator.stats();
                
                // 计算使用率指标
                let utilization_rate = if heap_size > 0 {
                    (stats.allocated_bytes as f64) / (heap_size as f64)
                } else {
                    0.0
                };
                
                let efficiency_rate = if stats.allocated_bytes > 0 {
                    (total_requested as f64) / (stats.allocated_bytes as f64)
                } else {
                    0.0
                };
                
                black_box((utilization_rate, efficiency_rate, stats));
                
                // 清理
                for ptr in ptrs {
                    allocator.deallocate(ptr);
                }
            });
        });
        
        group.bench_function("usage_rate_std", |b| {
            b.iter(|| {
                let mut allocator = StdAllocator::new();
                let mut ptrs = Vec::new();
                let mut total_requested = 0usize;
                
                // 分配多个不同大小的块
                for i in 0..50 {
                    let size = 64 + (i % 64); // 64-128字节
                    let ptr = allocator.allocate(black_box(size));
                    if !ptr.is_null() {
                        ptrs.push(ptr);
                        total_requested += size;
                    }
                }
                
                // 获取分配器统计信息
                let stats = allocator.stats();
                
                // 标准库分配器没有固定堆大小限制，所以只计算效率
                let efficiency_rate = if stats.allocated_bytes > 0 {
                    (total_requested as f64) / (stats.allocated_bytes as f64)
                } else {
                    0.0
                };
                
                black_box((efficiency_rate, stats));
                
                // 清理
                for ptr in ptrs {
                    allocator.deallocate(ptr);
                }
            });
        });
        
        group.finish();
    }
    
    /// 测试内存状态一致性 - 通过统计信息验证分配器的正确性
    fn bench_memory_consistency_check(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory_consistency");
        
        // 测试分配器状态的一致性（通过多轮操作检查累积误差）
        group.bench_function("consistency_check_rtthread", |b| {
            let mut allocator = RTThreadAllocator::new(64 * 1024);
            
            b.iter(|| {
                let initial_stats = allocator.stats();
                let mut ptrs = Vec::new();
                
                // 第一轮：分配20个块
                for i in 0..20 {
                    let size = black_box(64 + (i % 64)); // 64-128字节
                    let ptr = allocator.allocate(size);
                    if !ptr.is_null() {
                        ptrs.push(ptr);
                    }
                }
                
                let after_alloc_stats = allocator.stats();
                
                // 第二轮：释放所有块
                for ptr in ptrs {
                    allocator.deallocate(ptr);
                }
                
                let after_free_stats = allocator.stats();
                
                // 计算状态一致性指标
                let alloc_count_diff = after_alloc_stats.allocated_count - initial_stats.allocated_count;
                let dealloc_count_diff = after_free_stats.deallocated_count - after_alloc_stats.deallocated_count;
                let final_allocated_bytes = after_free_stats.allocated_bytes;
                
                // 理想情况下：分配次数应该等于释放次数，最终分配字节数应该回到初始状态
                let consistency_score = if alloc_count_diff == dealloc_count_diff && final_allocated_bytes <= initial_stats.allocated_bytes {
                    1.0 // 完全一致
                } else {
                    0.0 // 不一致，可能有泄漏或其他问题
                };
                
                black_box((initial_stats, after_alloc_stats, after_free_stats, consistency_score));
            });
        });
        
        group.bench_function("consistency_check_std", |b| {
            let mut allocator = StdAllocator::new();
            
            b.iter(|| {
                let initial_stats = allocator.stats();
                let mut ptrs = Vec::new();
                
                // 第一轮：分配20个块
                for i in 0..20 {
                    let size = black_box(64 + (i % 64)); // 64-128字节
                    let ptr = allocator.allocate(size);
                    if !ptr.is_null() {
                        ptrs.push(ptr);
                    }
                }
                
                let after_alloc_stats = allocator.stats();
                
                // 第二轮：释放所有块
                for ptr in ptrs {
                    allocator.deallocate(ptr);
                }
                
                let after_free_stats = allocator.stats();
                
                // 计算状态一致性指标
                let alloc_count_diff = after_alloc_stats.allocated_count - initial_stats.allocated_count;
                let dealloc_count_diff = after_free_stats.deallocated_count - after_alloc_stats.deallocated_count;
                let final_allocated_bytes = after_free_stats.allocated_bytes;
                
                let consistency_score = if alloc_count_diff == dealloc_count_diff && final_allocated_bytes <= initial_stats.allocated_bytes {
                    1.0
                } else {
                    0.0
                };
                
                black_box((initial_stats, after_alloc_stats, after_free_stats, consistency_score));
            });
        });
        
        group.finish();
    }
    
    /// 测试内存重用效率 - 评估释放后的内存是否能被有效重用
    fn bench_memory_reuse_efficiency(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory_reuse");
        
        group.bench_function("reuse_efficiency_rtthread", |b| {
            b.iter(|| {
                let mut allocator = RTThreadAllocator::new(64 * 1024);
                
                // 第一阶段：分配并释放一批内存，创建可重用的空间
                let mut first_batch = Vec::new();
                for i in 0..50 {
                    let size = black_box(64 + (i % 32)); // 64-96字节
                    let ptr = allocator.allocate(size);
                    if !ptr.is_null() {
                        first_batch.push(ptr);
                    }
                }
                
                let after_first_alloc = allocator.stats();
                
                // 释放第一批内存
                for ptr in first_batch {
                    allocator.deallocate(ptr);
                }
                
                let after_first_free = allocator.stats();
                
                // 第二阶段：再次分配相似大小的内存，测试重用效率
                let mut second_batch = Vec::new();
                for i in 0..50 {
                    let size = black_box(64 + (i % 32)); // 相同大小范围
                    let ptr = allocator.allocate(size);
                    if !ptr.is_null() {
                        second_batch.push(ptr);
                    }
                }
                
                let after_second_alloc = allocator.stats();
                
                // 计算重用效率：如果内存被有效重用，第二次分配的总内存使用应该接近第一次
                let first_peak = after_first_alloc.allocated_bytes;
                let second_peak = after_second_alloc.allocated_bytes;
                let reuse_efficiency = if first_peak > 0 {
                    1.0 - ((second_peak as f64 - first_peak as f64).abs() / first_peak as f64)
                } else {
                    0.0
                };
                
                black_box((after_first_alloc, after_first_free, after_second_alloc, reuse_efficiency));
                
                // 清理第二批
                for ptr in second_batch {
                    allocator.deallocate(ptr);
                }
            });
        });
        
        group.bench_function("reuse_efficiency_std", |b| {
            b.iter(|| {
                let mut allocator = StdAllocator::new();
                
                // 标准库分配器的重用测试（主要测试我们适配器的统计准确性）
                let mut first_batch = Vec::new();
                for i in 0..50 {
                    let size = black_box(64 + (i % 32));
                    let ptr = allocator.allocate(size);
                    if !ptr.is_null() {
                        first_batch.push(ptr);
                    }
                }
                
                let after_first_alloc = allocator.stats();
                
                for ptr in first_batch {
                    allocator.deallocate(ptr);
                }
                
                let after_first_free = allocator.stats();
                
                let mut second_batch = Vec::new();
                for i in 0..50 {
                    let size = black_box(64 + (i % 32));
                    let ptr = allocator.allocate(size);
                    if !ptr.is_null() {
                        second_batch.push(ptr);
                    }
                }
                
                let after_second_alloc = allocator.stats();
                
                // 对于标准库分配器，我们主要关注统计的准确性
                let stats_consistency = (after_first_free.allocated_bytes == 0) as i32 as f64;
                
                black_box((after_first_alloc, after_first_free, after_second_alloc, stats_consistency));
                
                for ptr in second_batch {
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
        bench_low_memory_stress,
        bench_memory_safety,
        bench_memory_usage_efficiency,
        bench_memory_consistency_check,
        bench_memory_reuse_efficiency
    );
    
    criterion_main!(benches);
    
    bench_continuous_operations,
    bench_low_memory_stress
);

criterion_main!(benches);

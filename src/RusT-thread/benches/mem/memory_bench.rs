use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rt_thread_mem::{RTThreadAllocator, StdAllocator, MemoryAllocator};

/// 基准测试：分配器创建
fn bench_allocator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocator_creation");
    
    group.bench_function("rtthread_creation", |b| {
        b.iter(|| {
            RTThreadAllocator::new(black_box(64 * 1024))
        });
    });
    
    group.bench_function("std_creation", |b| {
        b.iter(|| {
            StdAllocator::new()
        });
    });
    
    group.finish();
}

/// 基准测试：单次分配
fn bench_single_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_allocation");
    
    for size in [16, 32, 64, 128, 256, 512, 1024, 2048].iter() {
        group.bench_with_input(BenchmarkId::new("rtthread", size), size, |b, &size| {
            let mut allocator = RTThreadAllocator::new(64 * 1024);
            b.iter(|| {
                let ptr = allocator.allocate(black_box(size));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std", size), size, |b, &size| {
            let mut allocator = StdAllocator::new();
            b.iter(|| {
                let ptr = allocator.allocate(black_box(size));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                }
            });
        });
    }
    
    group.finish();
}

/// 基准测试：多次分配
fn bench_multiple_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_allocations");
    
    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("rtthread", count), count, |b, &count| {
            b.iter(|| {
                let mut allocator = RTThreadAllocator::new(1024 * 1024);
                let mut ptrs = Vec::new();
                
                for _ in 0..count {
                    let ptr = allocator.allocate(black_box(64));
                    if !ptr.is_null() {
                        ptrs.push(ptr);
                    }
                }
                
                for ptr in ptrs {
                    allocator.deallocate(ptr);
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std", count), count, |b, &count| {
            b.iter(|| {
                let mut allocator = StdAllocator::new();
                let mut ptrs = Vec::new();
                
                for _ in 0..count {
                    let ptr = allocator.allocate(black_box(64));
                    if !ptr.is_null() {
                        ptrs.push(ptr);
                    }
                }
                
                for ptr in ptrs {
                    allocator.deallocate(ptr);
                }
            });
        });
    }
    
    group.finish();
}

/// 基准测试：分配-释放循环
fn bench_alloc_free_cycles(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc_free_cycles");
    
    group.bench_function("rtthread", |b| {
        let mut allocator = RTThreadAllocator::new(64 * 1024);
        b.iter(|| {
            for _ in 0..100 {
                let ptr = allocator.allocate(black_box(128));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                }
            }
        });
    });
    
    group.bench_function("std", |b| {
        let mut allocator = StdAllocator::new();
        b.iter(|| {
            for _ in 0..100 {
                let ptr = allocator.allocate(black_box(128));
                if !ptr.is_null() {
                    allocator.deallocate(ptr);
                }
            }
        });
    });
    
    group.finish();
}

/// 基准测试：重分配
fn bench_reallocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("reallocation");
    
    for &(old_size, new_size) in [(64, 128), (128, 64), (64, 256), (256, 64)].iter() {
        let test_name = format!("{}to{}", old_size, new_size);
        
        group.bench_with_input(
            BenchmarkId::new("rtthread", &test_name), 
            &(old_size, new_size), 
            |b, &(old_size, new_size)| {
                let mut allocator = RTThreadAllocator::new(64 * 1024);
                b.iter(|| {
                    let ptr = allocator.allocate(black_box(old_size));
                    if !ptr.is_null() {
                        let new_ptr = allocator.reallocate(ptr, black_box(new_size));
                        if !new_ptr.is_null() {
                            allocator.deallocate(new_ptr);
                        }
                    }
                });
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("std", &test_name), 
            &(old_size, new_size), 
            |b, &(old_size, new_size)| {
                let mut allocator = StdAllocator::new();
                b.iter(|| {
                    let ptr = allocator.allocate(black_box(old_size));
                    if !ptr.is_null() {
                        let new_ptr = allocator.reallocate(ptr, black_box(new_size));
                        if !new_ptr.is_null() {
                            allocator.deallocate(new_ptr);
                        }
                    }
                });
            }
        );
    }
    
    group.finish();
}

/// 基准测试：内存碎片处理
fn bench_fragmentation_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmentation_handling");
    
    group.bench_function("rtthread", |b| {
        b.iter(|| {
            let mut allocator = RTThreadAllocator::new(64 * 1024);
            let mut ptrs = Vec::new();
            
            // 分配许多小块
            for _ in 0..100 {
                let ptr = allocator.allocate(black_box(64));
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 释放每隔一个块，创建碎片
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 0 {
                    allocator.deallocate(ptr);
                }
            }
            
            // 尝试分配更大的块
            let large_ptr = allocator.allocate(black_box(1024));
            
            // 清理剩余内存
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 1 {
                    allocator.deallocate(ptr);
                }
            }
            
            if !large_ptr.is_null() {
                allocator.deallocate(large_ptr);
            }
        });
    });
    
    group.bench_function("std", |b| {
        b.iter(|| {
            let mut allocator = StdAllocator::new();
            let mut ptrs = Vec::new();
            
            // 分配许多小块
            for _ in 0..100 {
                let ptr = allocator.allocate(black_box(64));
                if !ptr.is_null() {
                    ptrs.push(ptr);
                }
            }
            
            // 释放每隔一个块，创建碎片
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 0 {
                    allocator.deallocate(ptr);
                }
            }
            
            // 尝试分配更大的块
            let large_ptr = allocator.allocate(black_box(1024));
            
            // 清理剩余内存
            for (i, &ptr) in ptrs.iter().enumerate() {
                if i % 2 == 1 {
                    allocator.deallocate(ptr);
                }
            }
            
            if !large_ptr.is_null() {
                allocator.deallocate(large_ptr);
            }
        });
    });
    
    group.finish();
}

/// 基准测试：内存使用效率
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    group.bench_function("rtthread_utilization", |b| {
        let mut allocator = RTThreadAllocator::new(64 * 1024);
        let mut ptrs = Vec::new();
        
        // 预分配一些内存
        for _ in 0..10 {
            let ptr = allocator.allocate(1024);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
        
        b.iter(|| {
            let stats = allocator.stats();
            black_box(stats);
        });
        
        // 清理
        for ptr in ptrs {
            allocator.deallocate(ptr);
        }
    });
    
    group.bench_function("std_utilization", |b| {
        let mut allocator = StdAllocator::new();
        let mut ptrs = Vec::new();
        
        // 预分配一些内存
        for _ in 0..10 {
            let ptr = allocator.allocate(1024);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
        
        b.iter(|| {
            let stats = allocator.stats();
            black_box(stats);
        });
        
        // 清理
        for ptr in ptrs {
            allocator.deallocate(ptr);
        }
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_allocator_creation,
    bench_single_allocation,
    bench_multiple_allocations,
    bench_alloc_free_cycles,
    bench_reallocation,
    bench_fragmentation_handling,
    bench_memory_efficiency
);

criterion_main!(benches);

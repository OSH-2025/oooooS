use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;

// 模拟简化的内存分配器
struct SimpleAllocator {
    memory_pool: Vec<u8>,
    allocated_blocks: HashMap<usize, usize>, // address -> size
    free_blocks: Vec<(usize, usize)>, // (address, size)
    next_address: usize,
}

impl SimpleAllocator {
    fn new(pool_size: usize) -> Self {
        let mut free_blocks = Vec::new();
        free_blocks.push((0, pool_size));
        
        Self {
            memory_pool: vec![0; pool_size],
            allocated_blocks: HashMap::new(),
            free_blocks,
            next_address: 0,
        }
    }

    fn allocate(&mut self, size: usize) -> Option<usize> {
        // 查找合适的空闲块
        for (i, &(addr, block_size)) in self.free_blocks.iter().enumerate() {
            if block_size >= size {
                // 分配内存
                self.allocated_blocks.insert(addr, size);
                
                // 更新空闲块列表
                if block_size > size {
                    self.free_blocks[i] = (addr + size, block_size - size);
                } else {
                    self.free_blocks.remove(i);
                }
                
                return Some(addr);
            }
        }
        None
    }

    fn deallocate(&mut self, address: usize) -> bool {
        if let Some(size) = self.allocated_blocks.remove(&address) {
            // 将块添加回空闲列表
            self.free_blocks.push((address, size));
            // 简单版本：不进行块合并优化
            true
        } else {
            false
        }
    }

    fn defragment(&mut self) {
        // 简单的碎片整理：合并相邻的空闲块
        self.free_blocks.sort_by_key(|&(addr, _)| addr);
        
        let mut i = 0;
        while i + 1 < self.free_blocks.len() {
            let (addr1, size1) = self.free_blocks[i];
            let (addr2, size2) = self.free_blocks[i + 1];
            
            if addr1 + size1 == addr2 {
                // 合并相邻块
                self.free_blocks[i] = (addr1, size1 + size2);
                self.free_blocks.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }

    fn available_memory(&self) -> usize {
        self.free_blocks.iter().map(|(_, size)| size).sum()
    }
}

fn bench_allocator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocator_creation");
    
    for size in [1024, 4096, 16384, 65536].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_allocator", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let allocator = black_box(SimpleAllocator::new(size));
                    allocator
                })
            },
        );
    }
    group.finish();
}

fn bench_single_allocation(c: &mut Criterion) {
    c.bench_function("single_allocation", |b| {
        b.iter_batched(
            || SimpleAllocator::new(4096),
            |mut allocator| {
                let result = allocator.allocate(black_box(256));
                black_box(result)
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_multiple_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_allocations");
    
    for allocation_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("allocate_blocks", allocation_count),
            allocation_count,
            |b, &allocation_count| {
                b.iter_batched(
                    || SimpleAllocator::new(65536),
                    |mut allocator| {
                        let mut addresses = Vec::new();
                        for i in 0..allocation_count {
                            if let Some(addr) = allocator.allocate(black_box(64 + (i % 128))) {
                                addresses.push(addr);
                            }
                        }
                        black_box(addresses)
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

fn bench_allocation_deallocation_cycle(c: &mut Criterion) {
    c.bench_function("alloc_dealloc_cycle", |b| {
        b.iter_batched(
            || SimpleAllocator::new(8192),
            |mut allocator| {
                let mut addresses = Vec::new();
                
                // 分配阶段
                for i in 0..20 {
                    if let Some(addr) = allocator.allocate(black_box(128 + (i % 64))) {
                        addresses.push(addr);
                    }
                }
                
                // 释放阶段
                for &addr in &addresses {
                    allocator.deallocate(black_box(addr));
                }
                
                black_box(addresses)
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_fragmentation_scenario(c: &mut Criterion) {
    c.bench_function("fragmentation_handling", |b| {
        b.iter_batched(
            || SimpleAllocator::new(16384),
            |mut allocator| {
                let mut addresses = Vec::new();
                
                // 创建碎片化场景
                for i in 0..50 {
                    if let Some(addr) = allocator.allocate(black_box(64 + (i % 32))) {
                        addresses.push(addr);
                    }
                }
                
                // 释放一些块，创建碎片
                for i in (0..addresses.len()).step_by(2) {
                    allocator.deallocate(addresses[i]);
                }
                
                // 尝试分配大块
                let large_alloc = allocator.allocate(black_box(1024));
                
                black_box((addresses, large_alloc))
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_defragmentation(c: &mut Criterion) {
    c.bench_function("defragmentation", |b| {
        b.iter_batched(
            || {
                let mut allocator = SimpleAllocator::new(8192);
                let mut addresses = Vec::new();
                
                // 创建碎片化状态
                for i in 0..30 {
                    if let Some(addr) = allocator.allocate(64 + (i % 32)) {
                        addresses.push(addr);
                    }
                }
                
                // 释放一些块
                for i in (0..addresses.len()).step_by(3) {
                    allocator.deallocate(addresses[i]);
                }
                
                allocator
            },
            |mut allocator| {
                allocator.defragment();
                black_box(allocator.available_memory())
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_allocator_creation,
    bench_single_allocation,
    bench_multiple_allocations,
    bench_allocation_deallocation_cycle,
    bench_fragmentation_scenario,
    bench_defragmentation
);
criterion_main!(benches); 
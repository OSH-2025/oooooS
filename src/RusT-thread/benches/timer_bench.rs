use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::BinaryHeap;

// 模拟定时器事件
#[derive(Clone, Debug, PartialEq, Eq)]
struct TimerEvent {
    id: u32,
    deadline: u64,
    period: Option<u64>, // None 表示单次定时器，Some 表示周期定时器
    callback_data: u32,
}

impl PartialOrd for TimerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 使用 Reverse 来让 BinaryHeap 成为最小堆（最早截止时间在顶部）
        other.deadline.cmp(&self.deadline)
    }
}

// 模拟系统定时器管理器
struct SystemTimer {
    current_time: u64,
    timer_heap: BinaryHeap<TimerEvent>,
    next_timer_id: u32,
    tick_resolution: u64, // 系统滴答分辨率（微秒）
}

impl SystemTimer {
    fn new(tick_resolution: u64) -> Self {
        Self {
            current_time: 0,
            timer_heap: BinaryHeap::new(),
            next_timer_id: 1,
            tick_resolution,
        }
    }

    // 添加单次定时器
    fn add_oneshot_timer(&mut self, timeout_us: u64, callback_data: u32) -> u32 {
        let timer_id = self.next_timer_id;
        self.next_timer_id += 1;

        let deadline = self.current_time + timeout_us;
        let timer = TimerEvent {
            id: timer_id,
            deadline,
            period: None,
            callback_data,
        };

        self.timer_heap.push(timer);
        timer_id
    }

    // 添加周期定时器
    fn add_periodic_timer(&mut self, period_us: u64, callback_data: u32) -> u32 {
        let timer_id = self.next_timer_id;
        self.next_timer_id += 1;

        let deadline = self.current_time + period_us;
        let timer = TimerEvent {
            id: timer_id,
            deadline,
            period: Some(period_us),
            callback_data,
        };

        self.timer_heap.push(timer);
        timer_id
    }

    // 删除定时器
    fn remove_timer(&mut self, timer_id: u32) -> bool {
        // 简化实现：标记为删除，在处理时跳过
        // 实际实现中可能需要更复杂的数据结构
        let mut temp_timers = Vec::new();
        let mut found = false;

        while let Some(timer) = self.timer_heap.pop() {
            if timer.id != timer_id {
                temp_timers.push(timer);
            } else {
                found = true;
            }
        }

        for timer in temp_timers {
            self.timer_heap.push(timer);
        }

        found
    }

    // 推进系统时间并处理到期的定时器
    fn tick(&mut self, delta_us: u64) -> Vec<TimerEvent> {
        self.current_time += delta_us;
        let mut expired_timers = Vec::new();

        while let Some(timer) = self.timer_heap.peek() {
            if timer.deadline <= self.current_time {
                let mut timer = self.timer_heap.pop().unwrap();
                
                // 如果是周期定时器，重新调度
                if let Some(period) = timer.period {
                    let mut new_timer = timer.clone();
                    new_timer.deadline = self.current_time + period;
                    self.timer_heap.push(new_timer);
                }
                
                expired_timers.push(timer);
            } else {
                break;
            }
        }

        expired_timers
    }

    // 获取下一个定时器的超时时间
    fn next_timeout(&self) -> Option<u64> {
        self.timer_heap.peek().map(|timer| {
            if timer.deadline > self.current_time {
                timer.deadline - self.current_time
            } else {
                0
            }
        })
    }

    // 模拟高精度定时器操作
    fn high_precision_delay(&mut self, delay_us: u64) -> u64 {
        let start_time = self.current_time;
        
        // 模拟忙等待的高精度延时
        let mut elapsed = 0u64;
        while elapsed < delay_us {
            // 模拟一些计算开销
            let computation_overhead = || {
                let mut sum = 0u32;
                for i in 0..100 {
                    sum = sum.wrapping_add(i);
                }
                black_box(sum);
            };
            
            computation_overhead();
            elapsed += self.tick_resolution;
            self.current_time += self.tick_resolution;
        }
        
        self.current_time - start_time
    }

    fn get_timer_count(&self) -> usize {
        self.timer_heap.len()
    }
}

// 模拟中断处理的定时器
struct InterruptTimer {
    interrupt_count: u64,
    last_interrupt_time: u64,
    interrupt_interval: u64,
}

impl InterruptTimer {
    fn new(interval_us: u64) -> Self {
        Self {
            interrupt_count: 0,
            last_interrupt_time: 0,
            interrupt_interval: interval_us,
        }
    }

    fn handle_interrupt(&mut self, current_time: u64) -> bool {
        if current_time - self.last_interrupt_time >= self.interrupt_interval {
            self.interrupt_count += 1;
            self.last_interrupt_time = current_time;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.interrupt_count = 0;
        self.last_interrupt_time = 0;
    }
}

fn bench_timer_creation(c: &mut Criterion) {
    c.bench_function("timer_creation", |b| {
        b.iter(|| {
            let timer = black_box(SystemTimer::new(1000)); // 1ms 分辨率
            timer
        })
    });
}

fn bench_add_oneshot_timers(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_oneshot_timers");
    
    for timer_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("oneshot_timers", timer_count),
            timer_count,
            |b, &timer_count| {
                b.iter_batched(
                    || SystemTimer::new(1000),
                    |mut timer| {
                        for i in 0..timer_count {
                            timer.add_oneshot_timer(
                                black_box(1000 + (i as u64 * 100)), 
                                black_box(i as u32)
                            );
                        }
                        timer
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

fn bench_add_periodic_timers(c: &mut Criterion) {
    c.bench_function("add_periodic_timers", |b| {
        b.iter_batched(
            || SystemTimer::new(1000),
            |mut timer| {
                for i in 0..20 {
                    timer.add_periodic_timer(
                        black_box(5000 + (i as u64 * 1000)), 
                        black_box(i as u32)
                    );
                }
                timer
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_timer_tick_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_tick_processing");
    
    for active_timers in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_ticks", active_timers),
            active_timers,
            |b, &active_timers| {
                b.iter_batched(
                    || {
                        let mut timer = SystemTimer::new(1000);
                        // 添加一些即将到期的定时器
                        for i in 0..active_timers {
                            timer.add_oneshot_timer(1000 + (i as u64 * 100), i as u32);
                        }
                        timer
                    },
                    |mut timer| {
                        // 模拟系统滴答，处理到期定时器
                        let expired = timer.tick(black_box(5000));
                        black_box(expired)
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

fn bench_timer_removal(c: &mut Criterion) {
    c.bench_function("timer_removal", |b| {
        b.iter_batched(
            || {
                let mut timer = SystemTimer::new(1000);
                let mut timer_ids = Vec::new();
                
                // 添加一些定时器
                for i in 0..50 {
                    let id = timer.add_oneshot_timer(10000 + (i as u64 * 100), i as u32);
                    timer_ids.push(id);
                }
                
                (timer, timer_ids)
            },
            |(mut timer, timer_ids)| {
                // 删除一半的定时器
                for &timer_id in timer_ids.iter().step_by(2) {
                    timer.remove_timer(black_box(timer_id));
                }
                timer
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_high_precision_delay(c: &mut Criterion) {
    let mut group = c.benchmark_group("high_precision_delay");
    
    for delay_us in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("precision_delay", delay_us),
            delay_us,
            |b, &delay_us| {
                b.iter_batched(
                    || SystemTimer::new(10), // 高精度：10微秒分辨率
                    |mut timer| {
                        let actual_delay = timer.high_precision_delay(black_box(delay_us));
                        black_box(actual_delay)
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

fn bench_interrupt_timer_handling(c: &mut Criterion) {
    c.bench_function("interrupt_timer_handling", |b| {
        b.iter_batched(
            || {
                let interrupt_timer = InterruptTimer::new(1000); // 1ms 中断间隔
                (interrupt_timer, 0u64)
            },
            |(mut interrupt_timer, mut current_time)| {
                let mut interrupt_count = 0;
                
                // 模拟100个时间单位的中断处理
                for _ in 0..100 {
                    current_time += 100; // 每次增加100微秒
                    if interrupt_timer.handle_interrupt(black_box(current_time)) {
                        interrupt_count += 1;
                    }
                }
                
                black_box(interrupt_count)
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_mixed_timer_operations(c: &mut Criterion) {
    c.bench_function("mixed_timer_operations", |b| {
        b.iter_batched(
            || SystemTimer::new(1000),
            |mut timer| {
                let mut timer_ids = Vec::new();
                
                // 混合操作：添加、删除、处理
                for i in 0..20 {
                    // 添加定时器
                    let id1 = timer.add_oneshot_timer(1000 + (i as u64 * 50), i as u32);
                    let id2 = timer.add_periodic_timer(2000 + (i as u64 * 100), i as u32 + 100);
                    timer_ids.push(id1);
                    timer_ids.push(id2);
                    
                    // 偶尔删除一些定时器
                    if i % 5 == 0 && !timer_ids.is_empty() {
                        timer.remove_timer(timer_ids.remove(0));
                    }
                    
                    // 处理时间推进
                    timer.tick(100);
                }
                
                // 最终处理
                let final_expired = timer.tick(5000);
                black_box((timer.get_timer_count(), final_expired))
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_timer_heap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_heap_operations");
    
    // 测试堆操作的性能随定时器数量的变化
    for heap_size in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("heap_operations", heap_size),
            heap_size,
            |b, &heap_size| {
                b.iter_batched(
                    || {
                        let mut timer = SystemTimer::new(1000);
                        // 预填充堆
                        for i in 0..heap_size {
                            timer.add_oneshot_timer(
                                black_box(10000 + (i as u64 * 10)), 
                                black_box(i as u32)
                            );
                        }
                        timer
                    },
                    |mut timer| {
                        // 执行一系列堆操作
                        for _ in 0..10 {
                            timer.add_oneshot_timer(black_box(15000), black_box(999));
                            timer.tick(100);
                        }
                        timer.get_timer_count()
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_timer_creation,
    bench_add_oneshot_timers,
    bench_add_periodic_timers,
    bench_timer_tick_processing,
    bench_timer_removal,
    bench_high_precision_delay,
    bench_interrupt_timer_handling,
    bench_mixed_timer_operations,
    bench_timer_heap_operations
);
criterion_main!(benches); 
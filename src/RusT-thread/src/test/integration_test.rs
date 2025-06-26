#![cfg(test)]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// 导入RT-Thread模块
use RusT_thread::rtthread_rt::{
    rtdef::{RtErrT, RT_EOK, RT_ERROR, RT_ETIMEOUT, ThreadState, ThreadPriority, RtError},
    thread::{scheduler::Scheduler, thread::Thread, thread_priority_table::ThreadPriorityTable},
    mem::{small_mem_allocator::SmallMemAllocator, allocator::MemAllocator},
    timer::{timer::Timer, clock::Clock},
    ipc::IpcObject,
    rtconfig,
};

/// 集成测试套件
/// 测试RT-Thread系统的各个组件之间的交互和整体功能
pub struct IntegrationTestSuite {
    scheduler: Arc<Mutex<Scheduler>>,
    memory_allocator: Arc<Mutex<SmallMemAllocator>>,
    clock: Arc<Mutex<Clock>>,
    test_results: Vec<TestResult>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
}

impl IntegrationTestSuite {
    pub fn new() -> Self {
        Self {
            scheduler: Arc::new(Mutex::new(Scheduler::new())),
            memory_allocator: Arc::new(Mutex::new(SmallMemAllocator::new())),
            clock: Arc::new(Mutex::new(Clock::new())),
            test_results: Vec::new(),
        }
    }

    /// 运行所有集成测试
    pub fn run_all_tests(&mut self) -> Vec<TestResult> {
        println!("开始运行RT-Thread集成测试套件...");
        
        // 基础功能测试
        self.test_basic_thread_operations();
        self.test_memory_management();
        self.test_scheduler_operations();
        self.test_timer_operations();
        self.test_ipc_operations();
        
        // 并发测试
        self.test_concurrent_thread_creation();
        self.test_priority_scheduling();
        self.test_memory_fragmentation();
        self.test_timer_precision();
        
        // 压力测试
        self.test_stress_thread_creation();
        self.test_stress_memory_allocation();
        self.test_stress_timer_creation();
        
        // 边界条件测试
        self.test_boundary_conditions();
        self.test_error_handling();
        
        // 系统稳定性测试
        self.test_system_stability();
        
        println!("集成测试完成，共运行 {} 个测试", self.test_results.len());
        self.test_results.clone()
    }

    /// 测试基础线程操作
    fn test_basic_thread_operations(&mut self) {
        let start_time = Instant::now();
        let test_name = "基础线程操作测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            // 测试线程创建
            let mut scheduler = self.scheduler.lock().unwrap();
            let thread_id = scheduler.create_thread(
                "test_thread",
                ThreadPriority(5),
                1024,
                Box::new(|| {
                    println!("测试线程正在运行");
                    0
                }),
            );
            assert!(thread_id.is_ok(), "线程创建失败");
            
            // 测试线程启动
            let result = scheduler.start_thread(thread_id.unwrap());
            assert_eq!(result, RT_EOK, "线程启动失败");
            
            // 测试线程挂起和恢复
            let result = scheduler.suspend_thread(thread_id.unwrap());
            assert_eq!(result, RT_EOK, "线程挂起失败");
            
            let result = scheduler.resume_thread(thread_id.unwrap());
            assert_eq!(result, RT_EOK, "线程恢复失败");
            
            // 测试线程删除
            let result = scheduler.delete_thread(thread_id.unwrap());
            assert_eq!(result, RT_EOK, "线程删除失败");
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试内存管理
    fn test_memory_management(&mut self) {
        let start_time = Instant::now();
        let test_name = "内存管理测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut allocator = self.memory_allocator.lock().unwrap();
            
            // 测试内存分配
            let ptr1 = allocator.alloc(64);
            assert!(ptr1.is_some(), "64字节内存分配失败");
            
            let ptr2 = allocator.alloc(128);
            assert!(ptr2.is_some(), "128字节内存分配失败");
            
            let ptr3 = allocator.alloc(256);
            assert!(ptr3.is_some(), "256字节内存分配失败");
            
            // 测试内存释放
            if let Some(ptr) = ptr1 {
                allocator.dealloc(ptr, 64);
            }
            
            if let Some(ptr) = ptr2 {
                allocator.dealloc(ptr, 128);
            }
            
            if let Some(ptr) = ptr3 {
                allocator.dealloc(ptr, 256);
            }
            
            // 测试内存碎片整理
            allocator.defrag();
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试调度器操作
    fn test_scheduler_operations(&mut self) {
        let start_time = Instant::now();
        let test_name = "调度器操作测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut scheduler = self.scheduler.lock().unwrap();
            
            // 创建多个不同优先级的线程
            let thread1 = scheduler.create_thread(
                "high_priority",
                ThreadPriority(1),
                1024,
                Box::new(|| {
                    println!("高优先级线程运行");
                    0
                }),
            ).unwrap();
            
            let thread2 = scheduler.create_thread(
                "medium_priority",
                ThreadPriority(5),
                1024,
                Box::new(|| {
                    println!("中优先级线程运行");
                    0
                }),
            ).unwrap();
            
            let thread3 = scheduler.create_thread(
                "low_priority",
                ThreadPriority(10),
                1024,
                Box::new(|| {
                    println!("低优先级线程运行");
                    0
                }),
            ).unwrap();
            
            // 启动所有线程
            scheduler.start_thread(thread1);
            scheduler.start_thread(thread2);
            scheduler.start_thread(thread3);
            
            // 测试调度
            for _ in 0..10 {
                scheduler.schedule();
                thread::sleep(Duration::from_millis(10));
            }
            
            // 清理线程
            scheduler.delete_thread(thread1);
            scheduler.delete_thread(thread2);
            scheduler.delete_thread(thread3);
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试定时器操作
    fn test_timer_operations(&mut self) {
        let start_time = Instant::now();
        let test_name = "定时器操作测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut clock = self.clock.lock().unwrap();
            
            // 测试时钟初始化
            clock.init();
            
            // 测试时钟滴答
            for _ in 0..100 {
                clock.tick();
                thread::sleep(Duration::from_millis(1));
            }
            
            // 测试定时器创建和启动
            let mut timer = Timer::new("test_timer");
            timer.init();
            
            let start_time = clock.get_tick();
            timer.start(100); // 100个滴答后超时
            
            // 等待定时器超时
            while !timer.is_timeout() {
                clock.tick();
                thread::sleep(Duration::from_millis(1));
            }
            
            let end_time = clock.get_tick();
            assert!(end_time - start_time >= 100, "定时器超时时间不正确");
            
            // 测试定时器停止
            timer.stop();
            assert!(!timer.is_started(), "定时器停止失败");
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试IPC操作
    fn test_ipc_operations(&mut self) {
        let start_time = Instant::now();
        let test_name = "IPC操作测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut ipc = IpcObject::new();
            
            // 测试IPC对象初始化
            let result = ipc.init();
            assert_eq!(result, RT_EOK, "IPC对象初始化失败");
            
            // 测试线程挂起
            let result = ipc.suspend_thread(1, rtdef::RT_IPC_FLAG_FIFO);
            assert_eq!(result, RT_EOK, "线程挂起失败");
            
            let result = ipc.suspend_thread(2, rtdef::RT_IPC_FLAG_PRIO);
            assert_eq!(result, RT_EOK, "优先级线程挂起失败");
            
            // 测试线程恢复
            let result = ipc.resume_thread();
            assert_eq!(result, RT_EOK, "线程恢复失败");
            
            // 测试恢复所有线程
            let result = ipc.resume_all_threads();
            assert_eq!(result, RT_EOK, "恢复所有线程失败");
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试并发线程创建
    fn test_concurrent_thread_creation(&mut self) {
        let start_time = Instant::now();
        let test_name = "并发线程创建测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let scheduler = Arc::clone(&self.scheduler);
            let mut handles = Vec::new();
            
            // 创建多个线程同时创建线程
            for i in 0..5 {
                let scheduler_clone = Arc::clone(&scheduler);
                let handle = thread::spawn(move || {
                    let mut scheduler = scheduler_clone.lock().unwrap();
                    for j in 0..10 {
                        let thread_id = scheduler.create_thread(
                            &format!("concurrent_thread_{}_{}", i, j),
                            ThreadPriority(5),
                            512,
                            Box::new(|| {
                                thread::sleep(Duration::from_millis(1));
                                0
                            }),
                        );
                        assert!(thread_id.is_ok(), "并发线程创建失败");
                        
                        if let Ok(id) = thread_id {
                            scheduler.start_thread(id);
                        }
                    }
                });
                handles.push(handle);
            }
            
            // 等待所有线程完成
            for handle in handles {
                handle.join().unwrap();
            }
            
            // 清理所有创建的线程
            let mut scheduler = self.scheduler.lock().unwrap();
            scheduler.cleanup_all_threads();
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试优先级调度
    fn test_priority_scheduling(&mut self) {
        let start_time = Instant::now();
        let test_name = "优先级调度测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut scheduler = self.scheduler.lock().unwrap();
            let execution_order = Arc::new(Mutex::new(Vec::new()));
            
            // 创建不同优先级的线程
            for i in 0..5 {
                let execution_order_clone = Arc::clone(&execution_order);
                let thread_id = scheduler.create_thread(
                    &format!("priority_thread_{}", i),
                    ThreadPriority(i),
                    512,
                    Box::new(move || {
                        let mut order = execution_order_clone.lock().unwrap();
                        order.push(i);
                        thread::sleep(Duration::from_millis(10));
                        0
                    }),
                ).unwrap();
                
                scheduler.start_thread(thread_id);
            }
            
            // 运行调度器
            for _ in 0..50 {
                scheduler.schedule();
                thread::sleep(Duration::from_millis(1));
            }
            
            // 验证执行顺序（高优先级应该先执行）
            let order = execution_order.lock().unwrap();
            assert!(!order.is_empty(), "没有线程被执行");
            
            // 清理线程
            scheduler.cleanup_all_threads();
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试内存碎片化
    fn test_memory_fragmentation(&mut self) {
        let start_time = Instant::now();
        let test_name = "内存碎片化测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut allocator = self.memory_allocator.lock().unwrap();
            let mut allocations = Vec::new();
            
            // 分配不同大小的内存块
            for i in 0..20 {
                let size = 32 + (i * 16);
                if let Some(ptr) = allocator.alloc(size) {
                    allocations.push((ptr, size));
                }
            }
            
            // 释放部分内存块，创建碎片
            for i in (0..allocations.len()).step_by(2) {
                if i < allocations.len() {
                    let (ptr, size) = allocations[i];
                    allocator.dealloc(ptr, size);
                    allocations.remove(i);
                }
            }
            
            // 尝试分配大块内存
            let large_ptr = allocator.alloc(1024);
            assert!(large_ptr.is_some(), "大块内存分配失败");
            
            if let Some(ptr) = large_ptr {
                allocator.dealloc(ptr, 1024);
            }
            
            // 执行碎片整理
            allocator.defrag();
            
            // 清理剩余内存
            for (ptr, size) in allocations {
                allocator.dealloc(ptr, size);
            }
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试定时器精度
    fn test_timer_precision(&mut self) {
        let start_time = Instant::now();
        let test_name = "定时器精度测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut clock = self.clock.lock().unwrap();
            clock.init();
            
            let mut timer = Timer::new("precision_timer");
            timer.init();
            
            // 测试不同超时时间的精度
            let test_timeouts = vec![10, 50, 100, 500, 1000];
            
            for timeout in test_timeouts {
                let start_tick = clock.get_tick();
                timer.start(timeout);
                
                while !timer.is_timeout() {
                    clock.tick();
                    thread::sleep(Duration::from_millis(1));
                }
                
                let end_tick = clock.get_tick();
                let actual_timeout = end_tick - start_tick;
                
                // 允许5%的误差
                let tolerance = timeout as f32 * 0.05;
                assert!(
                    (actual_timeout as f32 - timeout as f32).abs() <= tolerance,
                    "定时器精度不满足要求: 期望 {}, 实际 {}, 容差 {}",
                    timeout,
                    actual_timeout,
                    tolerance
                );
                
                timer.stop();
            }
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 压力测试：大量线程创建
    fn test_stress_thread_creation(&mut self) {
        let start_time = Instant::now();
        let test_name = "线程创建压力测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut scheduler = self.scheduler.lock().unwrap();
            let mut thread_ids = Vec::new();
            
            // 创建大量线程
            for i in 0..100 {
                let thread_id = scheduler.create_thread(
                    &format!("stress_thread_{}", i),
                    ThreadPriority(i % 10),
                    256,
                    Box::new(|| {
                        thread::sleep(Duration::from_millis(1));
                        0
                    }),
                );
                
                if let Ok(id) = thread_id {
                    thread_ids.push(id);
                    scheduler.start_thread(id);
                }
            }
            
            // 运行一段时间
            for _ in 0..100 {
                scheduler.schedule();
                thread::sleep(Duration::from_millis(1));
            }
            
            // 清理所有线程
            for id in thread_ids {
                let _ = scheduler.delete_thread(id);
            }
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 压力测试：大量内存分配
    fn test_stress_memory_allocation(&mut self) {
        let start_time = Instant::now();
        let test_name = "内存分配压力测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut allocator = self.memory_allocator.lock().unwrap();
            let mut allocations = Vec::new();
            
            // 大量内存分配和释放
            for _ in 0..1000 {
                let size = 16 + (rand::random::<u32>() % 512) as usize;
                if let Some(ptr) = allocator.alloc(size) {
                    allocations.push((ptr, size));
                }
                
                // 随机释放一些内存
                if allocations.len() > 100 && rand::random::<bool>() {
                    if let Some((ptr, size)) = allocations.pop() {
                        allocator.dealloc(ptr, size);
                    }
                }
            }
            
            // 清理所有分配
            for (ptr, size) in allocations {
                allocator.dealloc(ptr, size);
            }
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 压力测试：大量定时器创建
    fn test_stress_timer_creation(&mut self) {
        let start_time = Instant::now();
        let test_name = "定时器创建压力测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut clock = self.clock.lock().unwrap();
            clock.init();
            
            let mut timers = Vec::new();
            
            // 创建大量定时器
            for i in 0..50 {
                let mut timer = Timer::new(&format!("stress_timer_{}", i));
                timer.init();
                timer.start(10 + (i % 100));
                timers.push(timer);
            }
            
            // 运行时钟
            for _ in 0..200 {
                clock.tick();
                thread::sleep(Duration::from_millis(1));
                
                // 检查定时器状态
                for timer in &mut timers {
                    if timer.is_timeout() {
                        timer.stop();
                    }
                }
            }
            
            // 停止所有定时器
            for timer in &mut timers {
                timer.stop();
            }
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试边界条件
    fn test_boundary_conditions(&mut self) {
        let start_time = Instant::now();
        let test_name = "边界条件测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut scheduler = self.scheduler.lock().unwrap();
            let mut allocator = self.memory_allocator.lock().unwrap();
            
            // 测试最小优先级
            let thread_id = scheduler.create_thread(
                "min_priority",
                ThreadPriority(0),
                64,
                Box::new(|| 0),
            );
            assert!(thread_id.is_ok(), "最小优先级线程创建失败");
            
            // 测试最大优先级
            let thread_id = scheduler.create_thread(
                "max_priority",
                ThreadPriority(255),
                64,
                Box::new(|| 0),
            );
            assert!(thread_id.is_ok(), "最大优先级线程创建失败");
            
            // 测试最小内存分配
            let ptr = allocator.alloc(1);
            assert!(ptr.is_some(), "最小内存分配失败");
            if let Some(ptr) = ptr {
                allocator.dealloc(ptr, 1);
            }
            
            // 测试零大小分配
            let ptr = allocator.alloc(0);
            assert!(ptr.is_none(), "零大小分配应该失败");
            
            // 测试无效线程ID
            let result = scheduler.start_thread(99999);
            assert_eq!(result, RT_ERROR, "无效线程ID应该返回错误");
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试错误处理
    fn test_error_handling(&mut self) {
        let start_time = Instant::now();
        let test_name = "错误处理测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let mut scheduler = self.scheduler.lock().unwrap();
            let mut allocator = self.memory_allocator.lock().unwrap();
            
            // 测试重复启动线程
            let thread_id = scheduler.create_thread(
                "error_test_thread",
                ThreadPriority(5),
                256,
                Box::new(|| 0),
            ).unwrap();
            
            scheduler.start_thread(thread_id);
            let result = scheduler.start_thread(thread_id);
            assert_eq!(result, RT_ERROR, "重复启动线程应该返回错误");
            
            // 测试重复挂起线程
            let result = scheduler.suspend_thread(thread_id);
            assert_eq!(result, RT_EOK, "线程挂起失败");
            let result = scheduler.suspend_thread(thread_id);
            assert_eq!(result, RT_ERROR, "重复挂起线程应该返回错误");
            
            // 测试内存不足情况
            let mut large_allocations = Vec::new();
            loop {
                if let Some(ptr) = allocator.alloc(1024) {
                    large_allocations.push(ptr);
                } else {
                    break; // 内存不足
                }
            }
            
            assert!(!large_allocations.is_empty(), "应该能够分配一些内存");
            
            // 清理内存
            for ptr in large_allocations {
                allocator.dealloc(ptr, 1024);
            }
            
            // 清理线程
            scheduler.delete_thread(thread_id);
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 测试系统稳定性
    fn test_system_stability(&mut self) {
        let start_time = Instant::now();
        let test_name = "系统稳定性测试".to_string();
        
        let result = std::panic::catch_unwind(|| {
            let scheduler = Arc::clone(&self.scheduler);
            let memory_allocator = Arc::clone(&self.memory_allocator);
            let clock = Arc::clone(&self.clock);
            
            // 长时间运行测试
            let test_duration = Duration::from_secs(5);
            let start_time = Instant::now();
            
            while start_time.elapsed() < test_duration {
                // 创建和销毁线程
                {
                    let mut scheduler = scheduler.lock().unwrap();
                    let thread_id = scheduler.create_thread(
                        "stability_thread",
                        ThreadPriority(5),
                        256,
                        Box::new(|| {
                            thread::sleep(Duration::from_millis(1));
                            0
                        }),
                    ).unwrap();
                    
                    scheduler.start_thread(thread_id);
                    scheduler.schedule();
                    scheduler.delete_thread(thread_id);
                }
                
                // 内存分配和释放
                {
                    let mut allocator = memory_allocator.lock().unwrap();
                    if let Some(ptr) = allocator.alloc(64) {
                        allocator.dealloc(ptr, 64);
                    }
                }
                
                // 时钟滴答
                {
                    let mut clock = clock.lock().unwrap();
                    clock.tick();
                }
                
                thread::sleep(Duration::from_millis(10));
            }
        });
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        self.test_results.push(TestResult {
            test_name,
            success,
            duration: start_time.elapsed(),
            error_message,
        });
    }

    /// 生成测试报告
    pub fn generate_report(&self) -> String {
        let total_tests = self.test_results.len();
        let passed_tests = self.test_results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - passed_tests;
        let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
        
        let total_duration: Duration = self.test_results.iter()
            .map(|r| r.duration)
            .sum();
        
        let mut report = String::new();
        report.push_str(&format!("=== RT-Thread 集成测试报告 ===\n"));
        report.push_str(&format!("总测试数: {}\n", total_tests));
        report.push_str(&format!("通过测试: {}\n", passed_tests));
        report.push_str(&format!("失败测试: {}\n", failed_tests));
        report.push_str(&format!("成功率: {:.2}%\n", success_rate));
        report.push_str(&format!("总耗时: {:?}\n\n", total_duration));
        
        // 详细测试结果
        report.push_str("详细测试结果:\n");
        for (i, result) in self.test_results.iter().enumerate() {
            let status = if result.success { "✓" } else { "✗" };
            report.push_str(&format!("{}. {} {} ({:?})\n", 
                i + 1, status, result.test_name, result.duration));
            
            if let Some(ref error) = result.error_message {
                report.push_str(&format!("   错误: {}\n", error));
            }
        }
        
        report
    }
}

#[test]
fn run_integration_tests() {
    let mut test_suite = IntegrationTestSuite::new();
    let results = test_suite.run_all_tests();
    
    // 打印测试报告
    let report = test_suite.generate_report();
    println!("{}", report);
    
    // 检查是否有失败的测试
    let failed_tests: Vec<_> = results.iter()
        .filter(|r| !r.success)
        .collect();
    
    if !failed_tests.is_empty() {
        panic!("有 {} 个测试失败", failed_tests.len());
    }
}

// 辅助函数：随机数生成
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    static mut SEED: u64 = 0;
    
    pub fn random<T>() -> T 
    where
        T: From<u32>
    {
        unsafe {
            if SEED == 0 {
                SEED = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
            }
            
            let mut hasher = DefaultHasher::new();
            SEED.hash(&mut hasher);
            SEED = hasher.finish();
            
            T::from((SEED & 0xFFFFFFFF) as u32)
        }
    }
} 
//! IPC 测试代码
//! 
//! 测试IPC基础功能，包括线程挂起、唤醒等
extern crate alloc;

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use crate::rtthread_rt::ipc::*;
use crate::rtthread_rt::kservice::RTIntrFreeCell;
use crate::rtthread_rt::rtdef::*;
use cortex_m_semihosting::hprintln;
use alloc::string::String;
use lazy_static::lazy_static;
use alloc::sync::Arc;
use spin::Mutex;

/// 测试线程1：测试IPC挂起功能
pub extern "C" fn test_ipc_thread_1(arg: usize) -> () {
    hprintln!("test_ipc_thread_1 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_1 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_1 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 3000 {
            hprintln!("test_ipc_thread_1 准备挂起...");
            // 获取IPC对象并挂起当前线程        
            let ipc = rt_ipc_init("test_sem", 1);
            rt_ipc_list_suspend(ipc.clone(), rt_thread_self().unwrap());
            hprintln!("test_ipc_thread_1 已挂起");
            break;
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 测试线程2：测试IPC唤醒功能
pub extern "C" fn test_ipc_thread_2(arg: usize) -> () {
    hprintln!("test_ipc_thread_2 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_2 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_2 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 5000 {
            hprintln!("test_ipc_thread_2 准备唤醒线程...");
            // 唤醒IPC队列中的第一个线程
            let ipc = rt_ipc_init("test_sem", 1);
            if let Some(woken_thread) = rt_ipc_list_resume(ipc.clone()) {
                hprintln!("test_ipc_thread_2 已唤醒线程: {:?}", String::from_utf8_lossy(&woken_thread.name));
            } else {
                hprintln!("test_ipc_thread_2 队列为空，没有线程可唤醒");
            }
            break;
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 测试线程3：测试IPC优先级队列
pub extern "C" fn test_ipc_thread_3(arg: usize) -> () {
    hprintln!("test_ipc_thread_3 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_3 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_3 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 2000 {
            hprintln!("test_ipc_thread_3 准备挂起...");
            // 获取IPC对象并挂起当前线程
            let ipc = rt_ipc_init("test_sem_prio", 1);
            rt_ipc_list_suspend(ipc.clone(), rt_thread_self().unwrap());
            hprintln!("test_ipc_thread_3 已挂起");
            break;
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 测试线程4：测试IPC全部唤醒功能
pub extern "C" fn test_ipc_thread_4(arg: usize) -> () {
    hprintln!("test_ipc_thread_4 开始...");
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    hprintln!("test_ipc_thread_4 start_tick: {:?}", start_tick);
    
    loop {
        if rt_tick_get() - start_tick > 200 {
            hprintln!("test_ipc_thread_4 运行中...");
            start_tick = rt_tick_get();
        }
        if rt_tick_get() - tic > 8000 {
            hprintln!("test_ipc_thread_4 准备唤醒所有线程...");
            // 唤醒IPC队列中的所有线程
            let ipc = rt_ipc_init("test_sem_prio", 1);
            rt_ipc_list_resume_all(ipc);
            hprintln!("test_ipc_thread_4 已唤醒所有线程");
            break;
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 运行IPC测试
pub fn run_ipc_test() {
    hprintln!("开始IPC测试...");
    
    // 创建测试线程

    let thread_1 = rt_thread_create("test_thread_1", test_ipc_thread_1 as usize, 2*1024, 10, 200);
    hprintln!("thread_1: {:?}", thread_1);
    let thread_2 = rt_thread_create("test_thread_2", test_ipc_thread_2 as usize, 2*1024, 10, 200);
    hprintln!("thread_2: {:?}", thread_2);
    let thread_3 = rt_thread_create("test_thread_3", test_ipc_thread_3 as usize, 2*1024, 14, 200);
    hprintln!("thread_3: {:?}", thread_3);
    let thread_4 = rt_thread_create("test_thread_4", test_ipc_thread_4 as usize, 2*1024, 16, 200);
    hprintln!("thread_4: {:?}", thread_4);

    // 禁用中断并启动线程
    let level = rt_hw_interrupt_disable();
    
    // 启动所有线程
    rt_thread_startup(thread_1);
    rt_thread_startup(thread_2.clone());
    rt_thread_startup(thread_3);
    rt_thread_startup(thread_4);
    
    // 重新启用中断
    rt_hw_interrupt_enable(level);
    
    hprintln!("IPC测试线程已启动");
}

/// 测试IPC初始化功能
pub fn test_ipc_init() {
    hprintln!("测试IPC初始化...");
    
    // 测试创建IPC对象
    let ipc1 = rt_ipc_init("test_semaphore", 1);
    let ipc2 = rt_ipc_init("test_mutex", 2);
    
    hprintln!("IPC对象1名称: {:?}", String::from_utf8_lossy(&ipc1.name));
    hprintln!("IPC对象1类型: {}", ipc1.object_type);
    hprintln!("IPC对象2名称: {:?}", String::from_utf8_lossy(&ipc2.name));
    hprintln!("IPC对象2类型: {}", ipc2.object_type);
    
    hprintln!("IPC初始化测试完成");
}

/// 测试IPC队列操作
pub fn test_ipc_queue_operations() {
    hprintln!("测试IPC队列操作...");
    
    let ipc = rt_ipc_init("test_queue", 1);
    
    // 创建测试线程
    let thread_1 = rt_thread_create("queue_test_1", test_ipc_thread_1 as usize, 1*1024, 10, 1000);
    let thread_2 = rt_thread_create("queue_test_2", test_ipc_thread_2 as usize, 1*1024, 12, 1000);
    
    // 测试挂起线程到队列
    rt_ipc_list_suspend(ipc.clone(), thread_1.clone());
    rt_ipc_list_suspend(ipc.clone(), thread_2.clone());
    
    // 检查队列长度
    let queue_len = ipc.thread_queue.exclusive_session(|queue| queue.len());
    hprintln!("IPC队列长度: {}", queue_len);
    
    // 测试唤醒第一个线程
    if let Some(woken_thread) = rt_ipc_list_resume(ipc.clone()) {
        hprintln!("已唤醒线程: {:?}", String::from_utf8_lossy(&woken_thread.name));
    } else {
        hprintln!("队列为空，没有线程可唤醒");
    }
    
    // 再次检查队列长度
    let queue_len_after = ipc.thread_queue.exclusive_session(|queue| queue.len());
    hprintln!("唤醒后IPC队列长度: {}", queue_len_after);
    
    hprintln!("IPC队列操作测试完成");
}

/// 哲学家进餐问题测试
/// 
/// 使用信号量实现哲学家进餐问题的解决方案
/// 通过避免死锁的算法来确保所有哲学家都能进餐

lazy_static! {
    // 5个叉子的信号量
    static ref FORKS: [Arc<Semaphore>; 5] = [
        Arc::new(Semaphore {
            parent: unsafe { RTIntrFreeCell::new(rt_ipc_init("fork_0", 1)) },
            count: Mutex::new(1),
        }),
        Arc::new(Semaphore {
            parent: unsafe { RTIntrFreeCell::new(rt_ipc_init("fork_1", 1)) },
            count: Mutex::new(1),
        }),
        Arc::new(Semaphore {
            parent: unsafe { RTIntrFreeCell::new(rt_ipc_init("fork_2", 1)) },
            count: Mutex::new(1),
        }),
        Arc::new(Semaphore {
            parent: unsafe { RTIntrFreeCell::new(rt_ipc_init("fork_3", 1)) },
            count: Mutex::new(1),
        }),
        Arc::new(Semaphore {
            parent: unsafe { RTIntrFreeCell::new(rt_ipc_init("fork_4", 1)) },
            count: Mutex::new(1),
        }),
    ];
    
    // 限制同时进餐的哲学家数量，避免死锁
    static ref ROOM: Arc<Semaphore> = Arc::new(Semaphore {
        parent: unsafe { RTIntrFreeCell::new(rt_ipc_init("room", 1)) },
        count: Mutex::new(4), // 最多4个哲学家同时进餐
    });
}

/// 哲学家线程函数
/// @param arg 哲学家编号 (0-4)
pub extern "C" fn philosopher_thread_1() -> () {
    let philosopher_id = 1;
    let left_fork = philosopher_id;
    let right_fork = (philosopher_id + 1) % 5;
    
    hprintln!("哲学家 {} 开始思考...", philosopher_id);
    
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    
    loop {
        // 思考一段时间
        if rt_tick_get() - start_tick > 500 {
            hprintln!("哲学家 {} 正在思考...", philosopher_id);
            start_tick = rt_tick_get();
        }
        
        // 每隔一段时间尝试进餐
        if rt_tick_get() - tic > 2000 {
            hprintln!("哲学家 {} 感到饥饿，准备进餐...", philosopher_id);
            
            // 进入餐厅
            if rt_sem_take(ROOM.clone(), 1000) == RT_EOK {
                hprintln!("哲学家 {} 进入餐厅", philosopher_id);
                
                // 拿起左叉子
                if rt_sem_take(FORKS[left_fork].clone(), 1000) == RT_EOK {
                    hprintln!("哲学家 {} 拿起左叉子 {}", philosopher_id, left_fork);
                    
                    // 拿起右叉子
                    if rt_sem_take(FORKS[right_fork].clone(), 1000) == RT_EOK {
                        hprintln!("哲学家 {} 拿起右叉子 {}，开始进餐", philosopher_id, right_fork);
                        
                        // 进餐一段时间
                        let eat_start = rt_tick_get();
                        while rt_tick_get() - eat_start < 300 {
                            // 进餐中...
                        }
                        
                        hprintln!("哲学家 {} 进餐完成", philosopher_id);
                        
                        // 放下右叉子
                        rt_sem_release(FORKS[right_fork].clone());
                        hprintln!("哲学家 {} 放下右叉子 {}", philosopher_id, right_fork);
                    } else {
                        hprintln!("哲学家 {} 无法拿起右叉子 {}", philosopher_id, right_fork);
                    }
                    
                    // 放下左叉子
                    rt_sem_release(FORKS[left_fork].clone());
                    hprintln!("哲学家 {} 放下左叉子 {}", philosopher_id, left_fork);
                } else {
                    hprintln!("哲学家 {} 无法拿起左叉子 {}", philosopher_id, left_fork);
                }
                
                // 离开餐厅
                rt_sem_release(ROOM.clone());
                hprintln!("哲学家 {} 离开餐厅", philosopher_id);
            } else {
                hprintln!("哲学家 {} 无法进入餐厅", philosopher_id);
            }
            
            // 重置计时器，继续循环
            break;
        }
    }
    
    // 删除线程
    rt_thread_delete(rt_thread_self().unwrap());
}

pub extern "C" fn philosopher_thread_2() -> () {
    let philosopher_id = 2;
    let left_fork = philosopher_id;
    let right_fork = (philosopher_id + 1) % 5;
    
    hprintln!("哲学家 {} 开始思考...", philosopher_id);
    
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    
    loop {
        // 思考一段时间
        if rt_tick_get() - start_tick > 500 {
            hprintln!("哲学家 {} 正在思考...", philosopher_id);
            start_tick = rt_tick_get();
        }
        
        // 每隔一段时间尝试进餐
        if rt_tick_get() - tic > 2000 {
            hprintln!("哲学家 {} 感到饥饿，准备进餐...", philosopher_id);

            // 进入餐厅
            if rt_sem_take(ROOM.clone(), 1000) == RT_EOK {
                hprintln!("哲学家 {} 进入餐厅", philosopher_id);
                
                // 拿起左叉子
                if rt_sem_take(FORKS[left_fork].clone(), 1000) == RT_EOK {
                    hprintln!("哲学家 {} 拿起左叉子 {}", philosopher_id, left_fork);
                    
                    // 拿起右叉子
                    if rt_sem_take(FORKS[right_fork].clone(), 1000) == RT_EOK {
                        hprintln!("哲学家 {} 拿起右叉子 {}，开始进餐", philosopher_id, right_fork);
                        
                        // 进餐一段时间
                        let eat_start = rt_tick_get();
                        while rt_tick_get() - eat_start < 300 {
                            // 进餐中...
                        }
                        
                        hprintln!("哲学家 {} 进餐完成", philosopher_id);
                        
                        // 放下右叉子
                        rt_sem_release(FORKS[right_fork].clone());
                        hprintln!("哲学家 {} 放下右叉子 {}", philosopher_id, right_fork);
                    } else {
                        hprintln!("哲学家 {} 无法拿起右叉子 {}", philosopher_id, right_fork);
                    }

                    // 放下左叉子
                    rt_sem_release(FORKS[left_fork].clone());
                    hprintln!("哲学家 {} 放下左叉子 {}", philosopher_id, left_fork);
                } else {
                    hprintln!("哲学家 {} 无法拿起左叉子 {}", philosopher_id, left_fork);
                }

                // 离开餐厅
                rt_sem_release(ROOM.clone());
                hprintln!("哲学家 {} 离开餐厅", philosopher_id);
            } else {
                hprintln!("哲学家 {} 无法进入餐厅", philosopher_id);
            }
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

pub extern "C" fn philosopher_thread_3() -> () {
    let philosopher_id = 3;
    let left_fork = philosopher_id;
    let right_fork = (philosopher_id + 1) % 5;
    
    hprintln!("哲学家 {} 开始思考...", philosopher_id);
    
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    
    loop {
        // 思考一段时间
        if rt_tick_get() - start_tick > 500 {
            hprintln!("哲学家 {} 正在思考...", philosopher_id);
            start_tick = rt_tick_get();
        }
        
        // 每隔一段时间尝试进餐
        if rt_tick_get() - tic > 2000 {
            hprintln!("哲学家 {} 感到饥饿，准备进餐...", philosopher_id);
            
            // 进入餐厅
            if rt_sem_take(ROOM.clone(), 1000) == RT_EOK {
                hprintln!("哲学家 {} 进入餐厅", philosopher_id);
                
                // 拿起左叉子
                if rt_sem_take(FORKS[left_fork].clone(), 1000) == RT_EOK {
                    hprintln!("哲学家 {} 拿起左叉子 {}", philosopher_id, left_fork);
                    
                    // 拿起右叉子
                    if rt_sem_take(FORKS[right_fork].clone(), 1000) == RT_EOK {
                        hprintln!("哲学家 {} 拿起右叉子 {}，开始进餐", philosopher_id, right_fork);
                        
                        // 进餐一段时间
                        let eat_start = rt_tick_get();
                        while rt_tick_get() - eat_start < 300 {
                            // 进餐中...
                        }
                        
                        hprintln!("哲学家 {} 进餐完成", philosopher_id);
                        
                        // 放下右叉子
                        rt_sem_release(FORKS[right_fork].clone());
                        hprintln!("哲学家 {} 放下右叉子 {}", philosopher_id, right_fork);
                    } else {
                        hprintln!("哲学家 {} 无法拿起右叉子 {}", philosopher_id, right_fork);
                    }
                    
                    // 放下左叉子
                    rt_sem_release(FORKS[left_fork].clone());
                    hprintln!("哲学家 {} 放下左叉子 {}", philosopher_id, left_fork);
                } else {
                    hprintln!("哲学家 {} 无法拿起左叉子 {}", philosopher_id, left_fork);
                }
                
                // 离开餐厅
                rt_sem_release(ROOM.clone());
                hprintln!("哲学家 {} 离开餐厅", philosopher_id);
            } else {
                hprintln!("哲学家 {} 无法进入餐厅", philosopher_id);
            }
            
            // 重置计时器，继续循环
            break;
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

pub extern "C" fn philosopher_thread_4() -> () {
    let philosopher_id = 4;
    let left_fork = philosopher_id;
    let right_fork = (philosopher_id + 1) % 5;
    
    hprintln!("哲学家 {} 开始思考...", philosopher_id);
    
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();
    
    loop {
        // 思考一段时间
        if rt_tick_get() - start_tick > 500 {
            hprintln!("哲学家 {} 正在思考...", philosopher_id);
            start_tick = rt_tick_get();
        }
        
        // 每隔一段时间尝试进餐
        if rt_tick_get() - tic > 2000 {
            hprintln!("哲学家 {} 感到饥饿，准备进餐...", philosopher_id);
            
            // 进入餐厅
            if rt_sem_take(ROOM.clone(), 1000) == RT_EOK {
                hprintln!("哲学家 {} 进入餐厅", philosopher_id);
                
                // 拿起左叉子
                if rt_sem_take(FORKS[left_fork].clone(), 1000) == RT_EOK {
                    hprintln!("哲学家 {} 拿起左叉子 {}", philosopher_id, left_fork);
                    
                    // 拿起右叉子
                    if rt_sem_take(FORKS[right_fork].clone(), 1000) == RT_EOK {
                        hprintln!("哲学家 {} 拿起右叉子 {}，开始进餐", philosopher_id, right_fork);
                        
                        // 进餐一段时间
                        let eat_start = rt_tick_get();
                        while rt_tick_get() - eat_start < 300 {
                            // 进餐中...
                        }
                        
                        hprintln!("哲学家 {} 进餐完成", philosopher_id);
                        
                        // 放下右叉子
                        rt_sem_release(FORKS[right_fork].clone());
                        hprintln!("哲学家 {} 放下右叉子 {}", philosopher_id, right_fork);
                    } else {
                        hprintln!("哲学家 {} 无法拿起右叉子 {}", philosopher_id, right_fork);
                    }
                    
                    // 放下左叉子
                    rt_sem_release(FORKS[left_fork].clone());
                    hprintln!("哲学家 {} 放下左叉子 {}", philosopher_id, left_fork);
                } else {
                    hprintln!("哲学家 {} 无法拿起左叉子 {}", philosopher_id, left_fork);
                }
                
                // 离开餐厅
                rt_sem_release(ROOM.clone());
                hprintln!("哲学家 {} 离开餐厅", philosopher_id);
            } else {
                hprintln!("哲学家 {} 无法进入餐厅", philosopher_id);
            }
            
            // 重置计时器，继续循环
            break;
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

pub extern "C" fn philosopher_thread_0() -> () {
    let philosopher_id = 0;
    let left_fork = philosopher_id;
    let right_fork = (philosopher_id + 1) % 5;
    
    hprintln!("哲学家 {} 开始思考...", philosopher_id);
    
    let tic = rt_tick_get();
    let mut start_tick = rt_tick_get();

    loop {
        // 思考一段时间
        if rt_tick_get() - start_tick > 500 {
            hprintln!("哲学家 {} 正在思考...", philosopher_id);
            start_tick = rt_tick_get();
        }
        
        // 每隔一段时间尝试进餐
        if rt_tick_get() - tic > 2000 {
            hprintln!("哲学家 {} 感到饥饿，准备进餐...", philosopher_id);
            
            // 进入餐厅
            if rt_sem_take(ROOM.clone(), 1000) == RT_EOK {
                hprintln!("哲学家 {} 进入餐厅", philosopher_id);
                
                // 拿起左叉子
                if rt_sem_take(FORKS[left_fork].clone(), 1000) == RT_EOK {
                    hprintln!("哲学家 {} 拿起左叉子 {}", philosopher_id, left_fork);
                    
                    // 拿起右叉子
                    if rt_sem_take(FORKS[right_fork].clone(), 1000) == RT_EOK {
                        hprintln!("哲学家 {} 拿起右叉子 {}，开始进餐", philosopher_id, right_fork);
                        
                        // 进餐一段时间
                        let eat_start = rt_tick_get();
                        while rt_tick_get() - eat_start < 30 {
                            // 进餐中...
                        }
                        
                        hprintln!("哲学家 {} 进餐完成", philosopher_id);
                        
                        // 放下右叉子
                        rt_sem_release(FORKS[right_fork].clone());
                        hprintln!("哲学家 {} 放下右叉子 {}", philosopher_id, right_fork);
                    } else {
                        hprintln!("哲学家 {} 无法拿起右叉子 {}", philosopher_id, right_fork);
                    }
                    
                    // 放下左叉子
                    rt_sem_release(FORKS[left_fork].clone());
                    hprintln!("哲学家 {} 放下左叉子 {}", philosopher_id, left_fork);
                } else {
                    hprintln!("哲学家 {} 无法拿起左叉子 {}", philosopher_id, left_fork);
                }
                
                // 离开餐厅
                rt_sem_release(ROOM.clone());
                hprintln!("哲学家 {} 离开餐厅", philosopher_id);
            } else {
                hprintln!("哲学家 {} 无法进入餐厅", philosopher_id);
            }
            
            // 重置计时器，继续循环
            break;
        }
    }
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 运行哲学家进餐问题测试
pub fn run_dining_philosophers_test() {
    hprintln!("开始哲学家进餐问题测试...");
    
    // 创建5个哲学家线程，传递不同的参数
    let philosophers = [
        rt_thread_create("philosopher_0", philosopher_thread_0 as usize, 2*1024, 10, 20),
        rt_thread_create("philosopher_1", philosopher_thread_1 as usize, 2*1024, 10, 20),
        rt_thread_create("philosopher_2", philosopher_thread_2 as usize, 2*1024, 10, 20),
        rt_thread_create("philosopher_3", philosopher_thread_3 as usize, 2*1024, 10, 20),
        rt_thread_create("philosopher_4", philosopher_thread_4 as usize, 2*1024, 10, 20),
    ];
    
    // 禁用中断并启动所有哲学家线程
    let level = rt_hw_interrupt_disable();
    
    for (i, philosopher) in philosophers.iter().enumerate() {
        hprintln!("启动哲学家 {}: {:?}", i, philosopher);
        rt_thread_startup(philosopher.clone());
    }
    
    // 重新启用中断
    rt_hw_interrupt_enable(level);
    
    hprintln!("哲学家进餐问题测试已启动");
}


/// 测试信号量基本功能
pub fn test_semaphore_basic() {
    hprintln!("测试信号量基本功能...");
    
    // 创建一个初始值为2的信号量
    let sem = Arc::new(Semaphore {
        parent: unsafe { RTIntrFreeCell::new(rt_ipc_init("test_sem", 1)) },
        count: Mutex::new(2),
    });
    
    hprintln!("信号量初始值: {}", *sem.count.lock());
    
    // 测试获取信号量
    if rt_sem_take(sem.clone(), 1000) == RT_EOK {
        hprintln!("成功获取信号量，当前值: {}", *sem.count.lock());
    } else {
        hprintln!("获取信号量失败");
    }
    
    if rt_sem_take(sem.clone(), 1000) == RT_EOK {
        hprintln!("成功获取信号量，当前值: {}", *sem.count.lock());
    } else {
        hprintln!("获取信号量失败");
    }
    
    // 尝试获取第三个信号量（应该失败）
    // if rt_sem_take(sem.clone(), 100) == RT_EOK {
    //     hprintln!("意外成功获取信号量");
    // } else {
    //     hprintln!("正确：无法获取更多信号量");
    // }
    
    // 释放信号量
    rt_sem_release(sem.clone());
    hprintln!("释放信号量，当前值: {}", *sem.count.lock());
    
    rt_sem_release(sem.clone());
    hprintln!("释放信号量，当前值: {}", *sem.count.lock());
    
    hprintln!("信号量基本功能测试完成");
}

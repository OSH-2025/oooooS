//! 性能测试模块
//! 
//! 通过随机数据衡量法测试系统性能

use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::hardware::*;
use cortex_m_semihosting::hprintln;

extern crate alloc;
use core::str;
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use lazy_static::lazy_static;
use crate::rtthread_rt::kservice::RTIntrFreeCell;

/// ANSI颜色代码（如果semihosting支持）
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

/// 随机数生成器
struct RandomGenerator {
    seed: u32,
}

impl RandomGenerator {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// 生成下一个随机数 (线性同余法)
    pub fn next(&mut self) -> u32 {
        // 简单的线性同余随机数生成器
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        self.seed
    }

    /// 根据概率生成事件
    /// probability: 0-100 表示概率百分比
    pub fn generate_event(&mut self, probability: u32) -> bool {
        let random = self.next() % 100;
        random < probability
    }
}

/// 事件结构体
#[derive(Clone, Debug)]
struct Event {
    id: u32,
    generation_tick: u32,
    processing_tick: u32,
    completion_tick: u32,
    priority: u8,
}

impl Event {
    pub fn new(id: u32, priority: u8) -> Self {
        Self {
            id,
            generation_tick: rt_tick_get(),
            processing_tick: 0,
            completion_tick: 0,
            priority,
        }
    }

    /// 计算响应时间 (从生成到开始处理)
    pub fn response_time(&self) -> u32 {
        self.processing_tick - self.generation_tick
    }

    /// 计算处理时间 (从开始处理到完成)
    pub fn processing_time(&self) -> u32 {
        self.completion_tick - self.processing_tick
    }

    /// 计算总时间 (从生成到完成)
    pub fn total_time(&self) -> u32 {
        self.completion_tick - self.generation_tick
    }
}

// 全局事件队列和计数器
lazy_static! {
    static ref EVENT_QUEUE: RTIntrFreeCell<Vec<Event>> = unsafe { RTIntrFreeCell::new(Vec::new()) };
    static ref COMPLETED_EVENTS: RTIntrFreeCell<Vec<Event>> = unsafe { RTIntrFreeCell::new(Vec::new()) };
    static ref EVENT_COUNTER: AtomicU32 = AtomicU32::new(0);
    static ref COMPLETED_COUNTER: AtomicU32 = AtomicU32::new(0);
}


/// 评价性能并返回星级
fn rate_performance_stars(value: f32, excellent: f32, poor: f32) -> (&'static str, &'static str) {
    if value <= excellent {
        (GREEN, "★★★★★ (优秀)")
    } else if value <= (excellent + poor) / 3.0 {
        (GREEN, "★★★★☆ (良好)")
    } else if value <= 2.0 * (excellent + poor) / 3.0 {
        (YELLOW, "★★★☆☆ (一般)")
    } else if value <= poor {
        (YELLOW, "★★☆☆☆ (较差)")
    } else {
        (RED, "★☆☆☆☆ (需优化)")
    }
}

/// 打印单个性能条形图
fn print_bar_chart(label: &str, color: &str, value: f32, max_value: f32) {
    let bar_width = 30;
    let bar_len = ((value / max_value) * bar_width as f32) as usize;
    
    hprintln!("{}{} │{}{} {:.2}ms", 
             color, label, 
             "█".repeat(bar_len.min(bar_width)), 
             RESET, value);
}






// 目标生成事件数
const TARGET_EVENT_COUNT: u32 = 100;

/// 事件生成线程入口函数
pub extern "C" fn event_generator_entry(arg: usize) -> () {
    let mut rng = RandomGenerator::new(rt_tick_get() as u32);
    let event_probability = 10; // 20% 概率生成事件
    
    hprintln!("事件生成器启动，目标生成 {} 个事件", TARGET_EVENT_COUNT);
    
    while EVENT_COUNTER.load(Ordering::SeqCst) < TARGET_EVENT_COUNT {
        if rng.generate_event(event_probability) {
            // 生成一个新事件
            let event_id = EVENT_COUNTER.fetch_add(1, Ordering::SeqCst);
            
            // 随机优先级 (1-10)
            let priority = (rng.next() % 10 + 1) as u8;
            
            let event = Event::new(event_id, priority);
            // hprintln!("生成事件 #{} 优先级: {}", event_id, priority);
            // 根据优先级使用不同颜色
            let priority_color = if priority >= 7 {
                RED // 高优先级
            } else if priority >= 4 {
                YELLOW // 中优先级
            } else {
                GREEN // 低优先级
            };
            
            hprintln!("{}◆ 生成事件 #{} {}[优先级: {}]{}", 
                     BLUE, event_id, priority_color, priority, RESET);
            
            // 将事件添加到队列
            EVENT_QUEUE.exclusive_access().push(event);
        }
    }
    
    hprintln!("事件生成器停止，已生成 {} 个事件", EVENT_COUNTER.load(Ordering::SeqCst));
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 事件处理线程入口函数 (高优先级)
pub extern "C" fn high_priority_processor_entry(arg: usize) -> () {
    hprintln!("{}▲ 高优先级处理器启动{}", RED, RESET);
    
    while COMPLETED_COUNTER.load(Ordering::SeqCst) < TARGET_EVENT_COUNT {
        let event_opt = {
            let mut queue = EVENT_QUEUE.exclusive_access();
            
            // 查找优先级 7-10 的事件
            let pos = queue.iter().position(|e| e.priority >= 8);
            let event = pos.map(|i| queue.remove(i));
            
            event
        };
        
        if let Some(mut event) = event_opt {
            // 记录开始处理的时间
            event.processing_tick = rt_tick_get();
            hprintln!("{}▲ 高优先级处理器处理事件 #{}{}", RED, event.id, RESET);
            
            // 模拟处理时间 (优先级越高处理越快)
            let processing_time = 200 - event.priority as u32;
            // rt_thread_sleep(rt_thread_self().unwrap(), processing_time);
            
            // 记录完成时间
            event.completion_tick = rt_tick_get() + processing_time;
            
            // 添加到已完成事件列表并增加计数器
            let level = rt_hw_interrupt_disable();
            COMPLETED_EVENTS.exclusive_access().push(event);
            rt_hw_interrupt_enable(level);
            
            COMPLETED_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        // else if EVENT_COUNTER.load(Ordering::SeqCst) == TARGET_EVENT_COUNT {
        //     break;
        // }
    }
    
    hprintln!("{}✓ 高优先级处理器停止{}", RED, RESET);
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 事件处理线程入口函数 (中优先级)
pub extern "C" fn medium_priority_processor_entry(arg: usize) -> () {
    hprintln!("{}■ 中优先级处理器启动{}", YELLOW, RESET);
    
    while COMPLETED_COUNTER.load(Ordering::SeqCst) < TARGET_EVENT_COUNT {
        let event_opt = {
            let mut queue = EVENT_QUEUE.exclusive_access();
            // 查找优先级 4-6 的事件
            let pos = queue.iter().position(|e| e.priority >= 4 && e.priority <= 7);
            let event = pos.map(|i| queue.remove(i));

            event
        };

        if let Some(mut event) = event_opt {
            // 记录开始处理的时间
            event.processing_tick = rt_tick_get();
            hprintln!("{}■ 中优先级处理器处理事件 #{}{}", YELLOW, event.id, RESET);
            
            // 模拟处理时间 (优先级越高处理越快)
            let processing_time = 30 - event.priority as u32 * 2;
            // rt_thread_sleep(rt_thread_self().unwrap(), processing_time);
            
            // 记录完成时间
            event.completion_tick = rt_tick_get() + processing_time;
            
            // 添加到已完成事件列表并增加计数器
            let level = rt_hw_interrupt_disable();
            COMPLETED_EVENTS.exclusive_access().push(event);
            rt_hw_interrupt_enable(level);
            
            COMPLETED_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        // else if EVENT_COUNTER.load(Ordering::SeqCst) == TARGET_EVENT_COUNT {
        //     break;
        // }
    }
    
    hprintln!("{}✓ 中优先级处理器停止{}", YELLOW, RESET);
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 事件处理线程入口函数 (低优先级)
pub extern "C" fn low_priority_processor_entry(arg: usize) -> () {
    hprintln!("{}● 低优先级处理器启动{}", GREEN, RESET);
    
    while COMPLETED_COUNTER.load(Ordering::SeqCst) < TARGET_EVENT_COUNT {
        let event_opt = {
            let mut queue = EVENT_QUEUE.exclusive_access();
            
            // 查找优先级 1-3 的事件
            let pos = queue.iter().position(|e| e.priority >= 1 && e.priority <= 3);
            let event = pos.map(|i| queue.remove(i));
            
            event
        };
        
        if let Some(mut event) = event_opt {
            // 记录开始处理的时间
            event.processing_tick = rt_tick_get();
            hprintln!("{}● 低优先级处理器处理事件 #{}{}", GREEN, event.id, RESET);
            
            // 模拟处理时间 (优先级越高处理越快)
            let processing_time = 50 - event.priority as u32 * 5;
            // rt_thread_sleep(rt_thread_self().unwrap(), processing_time);
            
            // 记录完成时间
            event.completion_tick = rt_tick_get() + processing_time;
            
            // 添加到已完成事件列表并增加计数器
            let level = rt_hw_interrupt_disable();
            COMPLETED_EVENTS.exclusive_access().push(event);
            rt_hw_interrupt_enable(level);
            
            COMPLETED_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        
    }
    
    hprintln!("{}✓ 低优先级处理器停止{}", GREEN, RESET);
    rt_thread_delete(rt_thread_self().unwrap());
}

/// 结果分析线程入口函数
pub extern "C" fn result_analyzer_entry(arg: usize) -> () {
    hprintln!("结果分析器启动");
    
    rt_thread_suspend(rt_thread_self().unwrap());
    

    // 分析结果
    let events = COMPLETED_EVENTS.exclusive_access();
    let total_events = events.len();
    
    if total_events == 0 {
        hprintln!("没有完成的事件");
        return;
    }
    
    // 计算平均响应时间
    let mut total_response_time = 0;
    let mut total_processing_time = 0;
    let mut total_time = 0;
    
    // 按优先级分组的统计
    let mut high_count = 0;
    let mut high_response_time = 0;
    let mut medium_count = 0;
    let mut medium_response_time = 0;
    let mut low_count = 0;
    let mut low_response_time = 0;
    
    for event in events.iter() {
        total_response_time += event.response_time();
        total_processing_time += event.processing_time();
        total_time += event.total_time();
        
        // 按优先级分组
        if event.priority >= 8 {
            high_count += 1;
            high_response_time += event.response_time();
        } else if event.priority >= 4 {
            medium_count += 1;
            medium_response_time += event.response_time();
        } else {
            low_count += 1;
            low_response_time += event.response_time();
        }
    }

    let avg_response_time = rt_tick_to_ms(total_response_time) as f32 / total_events as f32;
    let avg_processing_time = rt_tick_to_ms(total_processing_time) as f32 / total_events as f32;
    let avg_total_time = rt_tick_to_ms(total_time) as f32 / total_events as f32;
    
    // 打印分割线
    hprintln!("\n{}{}══════════════════════════════════════════════════{}", BOLD, CYAN, RESET);
    hprintln!("{}{}             性能测试最终结果报告             {}", BOLD, CYAN, RESET);
    hprintln!("{}{}══════════════════════════════════════════════════{}", BOLD, CYAN, RESET);
    
    hprintln!("\n{}{}✓ 测试成功完成！{}", BOLD, GREEN, RESET);
    hprintln!("{}📊 总事件数: {}{}", BOLD, total_events, RESET);
    hprintln!("\n{}关键性能指标:{}", BOLD, RESET);
    hprintln!("平均响应时间: {:.2} ms", avg_response_time);
    hprintln!("平均处理时间: {} ms", avg_processing_time);
    hprintln!("平均总时间: {} ms", avg_total_time);
    
    
    // 按优先级输出结果
    if high_count > 0 {
        let high_avg = rt_tick_to_ms(high_response_time) as f32 / high_count as f32;
        hprintln!("高优先级事件 (8-10): {} 个, 平均响应时间: {} ms", 
                 high_count, high_avg);
    }
    
    if medium_count > 0 {
        let medium_avg = rt_tick_to_ms(medium_response_time) as f32 / medium_count as f32;
        hprintln!("中优先级事件 (4-7): {} 个, 平均响应时间: {} ms", 
                 medium_count, medium_avg);
    }
    
    if low_count > 0 {
        let low_avg = rt_tick_to_ms(low_response_time) as f32 / low_count as f32;
        hprintln!("低优先级事件 (1-3): {} 个, 平均响应时间: {} ms", 
                 low_count, low_avg);
    }
    
    hprintln!("结果分析器停止");
    hprintln!("测试完成");

}

/// 运行性能测试
pub fn run_performance_test() {
    // 显示ASCII艺术标题
    hprintln!("{}{}╔══════════════════════════════════════╗{}", BOLD, CYAN, RESET);
    hprintln!("{}{}║     实时操作系统性能测试系统         ║{}", BOLD, CYAN, RESET);
    hprintln!("{}{}╚══════════════════════════════════════╝{}", BOLD, CYAN, RESET);

    hprintln!("{}{}⚡⚡⚡ 开始实时系统性能测试 ⚡⚡⚡{}", BOLD, MAGENTA, RESET);
    
    // 重置测试状态
    EVENT_COUNTER.store(0, Ordering::SeqCst);
    COMPLETED_COUNTER.store(0, Ordering::SeqCst);
    EVENT_QUEUE.exclusive_access().clear();
    COMPLETED_EVENTS.exclusive_access().clear();
    
    // 创建事件生成器线程 (中等优先级)
    let generator = rt_thread_create(
        "event_gen", 
        event_generator_entry as usize, 
        2*1024, 
        15, 
        20
    );
    
    // 创建高优先级处理器线程
    let high_processor = rt_thread_create(
        "high_proc", 
        high_priority_processor_entry as usize, 
        2*1024, 
        10, 
        20
    );
    
    // 创建中优先级处理器线程
    let medium_processor = rt_thread_create(
        "med_proc", 
        medium_priority_processor_entry as usize, 
        2*1024, 
        15, 
        20
    );
    
    // 创建低优先级处理器线程
    let low_processor = rt_thread_create(
        "low_proc", 
        low_priority_processor_entry as usize, 
        2*1024, 
        20, 
        20
    );
    
    // 创建结果分析器线程 (最低优先级)
    let analyzer = rt_thread_create(
        "analyzer", 
        result_analyzer_entry as usize, 
        2*1024, 
        25, 
        100
    );
    
    // 启动所有线程
    hprintln!("性能测试线程已启动");
    let level = rt_hw_interrupt_disable();
    set_mfq_scheduling();
    rt_thread_startup(generator);
    rt_thread_startup(high_processor);
    rt_thread_startup(medium_processor);
    rt_thread_startup(low_processor);
    rt_thread_startup(analyzer.clone());
    rt_hw_interrupt_enable(level);
    
    while COMPLETED_COUNTER.load(Ordering::SeqCst) < TARGET_EVENT_COUNT {
       
    }

    rt_thread_resume(analyzer.clone());
    rt_thread_suspend(rt_thread_self().unwrap());

} 
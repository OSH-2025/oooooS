//! 基于DWT的高精度中断延迟测试
//! 
//! 使用CPU周期计数器测量从中断触发到中断处理函数执行的精确时间

use::core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_semihosting::hprintln;
use cortex_m::peripheral::{DWT, DCB, NVIC};
use cortex_m::Peripherals;
use cortex_m_rt::exception;

// 添加对device.x的中断支持
#[cfg(feature = "stm32f4xx")]
use stm32f4xx_hal::pac::Interrupt;

// 存储测试数据的静态变量
pub static START_CYCLES: AtomicU32 = AtomicU32::new(0);
pub static HANDLER_CYCLES: AtomicU32 = AtomicU32::new(0);
pub static TEST_RUNNING: AtomicU32 = AtomicU32::new(0);
pub static INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);

// 初始化DWT
fn init_dwt() {
    // 使用cortex_m库提供的安全接口
    let mut p = Peripherals::take().unwrap();
    
    // 启用DWT功能
    p.DCB.enable_trace();
    // 重置并启用周期计数器
    unsafe {
        p.DWT.cyccnt.write(10);
        // p.DWT.ctrl.modify(|w| w.cyccntena().set_bit());
    }

    p.DWT.enable_cycle_counter();
    
    // 验证计数器是否正常工作
    let start = p.DWT.cyccnt.read();
    // 小延时
    for _ in 0..100 {
        cortex_m::asm::nop();
    }
    let end = p.DWT.cyccnt.read();
    
    hprintln!("DWT初始化: 开始={}, 结束={}, 差值={}", start, end, end - start);
}

// 使用SysTick中断作为测试中断源
// pub fn SysTick_Handler() {
//     if TEST_RUNNING.load(Ordering::SeqCst) == 1 {
//         // 记录中断处理开始时的周期计数
//         let current_cycles = unsafe { core::ptr::read_volatile(0xE0001004 as *const u32) };
//         HANDLER_CYCLES.store(current_cycles, Ordering::SeqCst);
//         INTERRUPT_COUNT.fetch_add(1, Ordering::SeqCst);
//     }
// }

// 触发SysTick中断进行测试
fn trigger_test_interrupt() {
    // 获取外设
    let mut p = Peripherals::take().unwrap();
    
    // 标记测试开始
    TEST_RUNNING.store(1, Ordering::SeqCst);
    
    // 记录触发时间
    let current_cycles = p.DWT.cyccnt.read();
    START_CYCLES.store(current_cycles, Ordering::SeqCst);
    
    // 配置并触发SysTick中断
    let reload_value = 1; // 较小的值，以便快速触发
    p.SYST.set_reload(reload_value);
    p.SYST.clear_current();
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

// 执行高精度中断延迟测试
pub fn run_precise_interrupt_latency_test() {
    hprintln!("开始高精度中断延迟测试...");
    
    // 初始化DWT
    init_dwt();
    
    // 重置计数器
    START_CYCLES.store(0, Ordering::SeqCst);
    HANDLER_CYCLES.store(0, Ordering::SeqCst);
    INTERRUPT_COUNT.store(0, Ordering::SeqCst);
    TEST_RUNNING.store(0, Ordering::SeqCst);
    
    // 触发测试中断
    trigger_test_interrupt();
    
    // 等待中断处理完成
    while INTERRUPT_COUNT.load(Ordering::SeqCst) == 0 {
        // 空循环等待
        cortex_m::asm::nop();
    }
    
    // 禁用SysTick中断
    let mut p = unsafe { Peripherals::steal() };
    p.SYST.disable_counter();
    p.SYST.disable_interrupt();
    
    // 获取结果
    let start_cycles = START_CYCLES.load(Ordering::SeqCst);
    let handler_cycles = HANDLER_CYCLES.load(Ordering::SeqCst);
    
    // 计算延迟（处理周期计数器溢出的可能性）
    let latency_cycles = if handler_cycles >= start_cycles {
        handler_cycles - start_cycles
    } else {
        (0xFFFFFFFF - start_cycles) + handler_cycles + 1
    };
    
    // 标记测试结束
    TEST_RUNNING.store(0, Ordering::SeqCst);
    
    // 输出结果
    hprintln!("触发时间: {} 周期", start_cycles);
    hprintln!("处理时间: {} 周期", handler_cycles);
    hprintln!("中断延迟: {} CPU周期", latency_cycles);
    
    // 假设CPU频率为16MHz（与您的配置匹配）
    let cpu_freq_mhz = 16;
    let latency_ns = (latency_cycles as f32 / cpu_freq_mhz as f32) * 1000.0;
    hprintln!("中断延迟: {:.2} 纳秒", latency_ns);
}

// 多次测试取平均值
pub fn run_average_precise_latency_test(times: u32) {
    hprintln!("开始多次高精度中断延迟测试...");
    
    // 初始化DWT
    init_dwt();
    
    let mut total_cycles: u64 = 0;
    
    for i in 0..times {
        // 重置计数器
        START_CYCLES.store(0, Ordering::SeqCst);
        HANDLER_CYCLES.store(0, Ordering::SeqCst);
        INTERRUPT_COUNT.store(0, Ordering::SeqCst);
        TEST_RUNNING.store(0, Ordering::SeqCst);
        
        // 触发测试中断
        trigger_test_interrupt();
        
        // 等待中断处理完成
        while INTERRUPT_COUNT.load(Ordering::SeqCst) == 0 {
            // 空循环等待
            cortex_m::asm::nop();
        }
        
        // 禁用SysTick中断
        let mut p = unsafe { Peripherals::steal() };
        p.SYST.disable_counter();
        p.SYST.disable_interrupt();
        
        // 获取结果
        let start_cycles = START_CYCLES.load(Ordering::SeqCst);
        let handler_cycles = HANDLER_CYCLES.load(Ordering::SeqCst);
        
        // 计算延迟（处理周期计数器溢出的可能性）
        let latency_cycles = if handler_cycles >= start_cycles {
            handler_cycles - start_cycles
        } else {
            (0xFFFFFFFF - start_cycles) + handler_cycles + 1
        };
        
        // 累加延迟
        total_cycles += latency_cycles as u64;
        
        // 标记测试结束
        TEST_RUNNING.store(0, Ordering::SeqCst);
        
        // 等待一小段时间再进行下一次测试
        let mut delay = 1000000;
        while delay > 0 {
            delay -= 1;
            cortex_m::asm::nop();
        }
    }
    
    // 计算平均延迟
    let average_cycles = total_cycles / times as u64;
    
    // 输出结果
    hprintln!("平均中断延迟: {} CPU周期", average_cycles);
    
    // 假设CPU频率为16MHz（与您的配置匹配）
    let cpu_freq_mhz = 16;
    let average_latency_ns = (average_cycles as f32 / cpu_freq_mhz as f32) * 1000.0;
    hprintln!("平均中断延迟: {:.2} 纳秒", average_latency_ns);
}

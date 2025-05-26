//! 线程上下文切换模块
//! 
//! 使用内联汇编实现线程上下文保存与恢复

use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

// 寄存器常量定义
const SCB_VTOR: u32 = 0xE000ED08;           // Vector Table Offset Register
const NVIC_INT_CTRL: u32 = 0xE000ED04;      // interrupt control state register
const NVIC_SYSPRI2: u32 = 0xE000ED20;       // system priority register (2)
const NVIC_PENDSV_PRI: u32 = 0xFFFF0000;    // PendSV and SysTick priority value (lowest)
const NVIC_PENDSVSET: u32 = 0x10000000;     // value to trigger PendSV exception

// 线程切换标志
pub static THREAD_SWITCH_FLAG: AtomicBool = AtomicBool::new(false);

// 当前线程栈指针
pub static CURRENT_THREAD_SP: AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());

// 下一个要切换到的线程栈指针
pub static NEXT_THREAD_SP: AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());

/// 初始化上下文切换机制
pub fn init() {
    unsafe {
        // 设置PendSV中断为最低优先级
        let nvic_syspri2 = NVIC_SYSPRI2 as *mut u32;
        let temp = core::ptr::read_volatile(nvic_syspri2);
        core::ptr::write_volatile(nvic_syspri2, temp | NVIC_PENDSV_PRI);
    }
}

/// 触发上下文切换（通过PendSV中断）
#[inline]
pub fn trigger_context_switch() {
    unsafe {
        let nvic_int_ctrl = NVIC_INT_CTRL as *mut u32;
        core::ptr::write_volatile(nvic_int_ctrl, NVIC_PENDSVSET);
    }
}

/// 线程上下文切换
/// 
/// # 参数
/// 
/// * `from_sp`: 当前线程的栈指针
/// * `to_sp`: 目标线程的栈指针
#[inline]
pub fn rt_hw_context_switch(from_sp: *mut u32, to_sp: *mut u32) {
    // 设置线程切换标志
    THREAD_SWITCH_FLAG.store(true, Ordering::SeqCst);
    
    // 设置当前线程和目标线程的栈指针
    CURRENT_THREAD_SP.store(from_sp, Ordering::SeqCst);
    NEXT_THREAD_SP.store(to_sp, Ordering::SeqCst);
    
    // 触发PendSV中断
    trigger_context_switch();
}

/// 中断中的线程上下文切换
/// 
/// 与 rt_hw_context_switch 功能相同，但用于中断中的上下文切换
/// 
/// # 参数
/// 
/// * `from_sp`: 当前线程的栈指针
/// * `to_sp`: 目标线程的栈指针
#[inline]
pub fn rt_hw_context_switch_interrupt(from_sp: *mut u32, to_sp: *mut u32) {
    // 在 Cortex-M4 中，中断中的上下文切换与普通上下文切换相同
    rt_hw_context_switch(from_sp, to_sp)
}

/// 直接切换到指定线程（不保存当前上下文）
pub fn rt_hw_context_switch_to(to_sp: *mut u32) {
    // 设置目标线程的栈指针
    NEXT_THREAD_SP.store(to_sp, Ordering::SeqCst);
    
    // 清除当前线程的栈指针
    CURRENT_THREAD_SP.store(core::ptr::null_mut(), Ordering::SeqCst);
    
    // 设置线程切换标志
    THREAD_SWITCH_FLAG.store(true, Ordering::SeqCst);
    
    // 设置PendSV中断为最低优先级
    unsafe {
        let nvic_syspri2 = NVIC_SYSPRI2 as *mut u32;
        let temp = core::ptr::read_volatile(nvic_syspri2);
        core::ptr::write_volatile(nvic_syspri2, temp | NVIC_PENDSV_PRI);
    }
    
    // 触发PendSV中断
    trigger_context_switch();
    
    // 恢复MSP（主栈指针）
    unsafe {
        let vtor = core::ptr::read_volatile(SCB_VTOR as *const u32);
        let reset_sp = core::ptr::read_volatile(vtor as *const u32);
        asm!("msr msp, {0}", in(reg) reset_sp);
    }
    
    // 启用中断
    unsafe {
        asm!("cpsie f");
        asm!("cpsie i");
        asm!("dsb");
        asm!("isb");
    }
    
    // 程序不应该到达这里
    loop {}
}

/// PendSV中断处理函数（使用内联汇编实现上下文切换）
#[unsafe(no_mangle)]
pub unsafe extern "C" fn PendSV_Handler() {
    // 保存所有寄存器，恢复所有寄存器，完成上下文切换
    unsafe {
        asm!(
            // 禁用中断
            "mrs r2, PRIMASK",
            "cpsid i",
            
            // 获取线程切换标志
            "ldr r0, =THREAD_SWITCH_FLAG",
            "ldr r1, [r0]",
            "cbz r1, 2f", // 如果标志为0，跳转到标签2（退出）
            
            // 清除线程切换标志
            "movs r1, #0",
            "str r1, [r0]",
            
            // 获取当前线程栈指针
            "ldr r0, =CURRENT_THREAD_SP",
            "ldr r1, [r0]",
            "cbz r1, 1f", // 如果为空，跳转到标签1（切换到目标线程）
            
            // 获取PSP（进程栈指针）
            "mrs r1, psp",
            
            // 检查是否需要保存FPU寄存器
            "tst lr, #0x10",
            "it eq",
            "vstmdbeq r1!, {{d8-d15}}", // 如果需要，保存FPU寄存器
            
            // 保存核心寄存器（r4-r11）
            "stmfd r1!, {{r4-r11}}",
            
            // 检查是否需要保存FPU状态
            "tst lr, #0x10",
            "it eq",
            "moveq r4, #1", // FPU标志
            "it ne",
            "movne r4, #0",
            
            // 保存FPU标志
            "stmfd r1!, {{r4}}",
            
            // 更新当前线程栈指针
            "ldr r0, [r0]",
            "str r1, [r0]",
            
            // 切换到目标线程
            "1:",
            "ldr r1, =NEXT_THREAD_SP",
            "ldr r1, [r1]",
            "ldr r1, [r1]", // 获取目标线程栈值
            
            // 恢复FPU标志
            "ldmfd r1!, {{r3}}",
            
            // 恢复核心寄存器（r4-r11）
            "ldmfd r1!, {{r4-r11}}",
            
            // 如果需要，恢复FPU寄存器
            "cmp r3, #0",
            "it ne",
            "vldmiane r1!, {{d8-d15}}",
            
            // 更新PSP
            "msr psp, r1",
            
            // 处理FPU状态标志
            "orr lr, lr, #0x10", // 默认清除FPCA位
            "cmp r3, #0",
            "it ne",
            "bicne lr, lr, #0x10", // 如果需要，设置FPCA位
            
            // 退出
            "2:",
            "msr PRIMASK, r2", // 恢复中断状态
            
            // 确保使用PSP返回
            "orr lr, lr, #0x04",
            "bx lr",
            
            options(noreturn)
        );
    }
}

/// 硬件错误处理函数
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HardFault_Handler() {
    unsafe {
        asm!(
            // 获取当前上下文
            "mrs r0, msp",         // 尝试从MSP获取上下文
            "tst lr, #4",          // 检查EXC_RETURN[2]
            "bne 1f",
            "mrs r0, psp",         // 如果不是MSP，则从PSP获取
            "1:",
            
            // 保存r4-r11寄存器
            "stmfd r0!, {{r4-r11}}",
            
            // 保存LR
            "stmfd r0!, {{lr}}",
            
            // 更新栈指针
            "tst lr, #4",         // 检查EXC_RETURN[2]
            "bne 2f",
            "msr msp, r0",        // 更新MSP
            "b 3f",
            "2:",
            "msr psp, r0",        // 更新PSP
            "3:",
            
            // 调用硬件错误处理函数
            "bl handle_hardfault",
            
            // 返回
            "orr lr, lr, #0x04",  // 确保使用PSP返回
            "bx lr",
            
            options(noreturn)
        );
    }
}

/// 硬件错误处理函数
#[unsafe(no_mangle)]
pub unsafe extern "C" fn handle_hardfault() {
    // 在这里添加错误处理逻辑
    loop {}
} 
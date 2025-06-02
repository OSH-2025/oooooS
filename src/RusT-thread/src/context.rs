//! 线程上下文切换模块
//! 
//! 使用内联汇编实现线程上下文保存与恢复，复用RT-Thread的实现思路

use cortex_m;
use cortex_m_rt;
use core::arch::asm;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_semihosting::hprintln;

// 寄存器常量定义
const SCB_VTOR: u32 = 0xE000ED08;           // Vector Table Offset Register
const NVIC_INT_CTRL: u32 = 0xE000ED04;      // interrupt control state register
const NVIC_SYSPRI2: u32 = 0xE000ED20;       // system priority register (2)
const NVIC_PENDSV_PRI: u32 = 0xFFFF0000;    // PendSV and SysTick priority value (lowest)
const NVIC_PENDSVSET: u32 = 0x10000000;     // value to trigger PendSV exception

// 线程切换标志 - 使用原子类型增强线程安全
// static THREAD_SWITCH_FLAG: AtomicU32 = AtomicU32::new(0);
static CURRENT_THREAD_SP: AtomicU32 = AtomicU32::new(0);
static NEXT_THREAD_SP: AtomicU32 = AtomicU32::new(0);

// 兼容性导出 - 以便汇编代码引用
#[unsafe(no_mangle)]
pub static mut rt_thread_switch_interrupt_flag: u32 = 0;
#[unsafe(no_mangle)]
pub static mut rt_interrupt_from_thread: *mut u32 = 0 as *mut u32;
#[unsafe(no_mangle)]
pub static mut rt_interrupt_to_thread: *mut u32 = 0 as *mut u32;

/// 初始化上下文切换机制
pub fn init() {
    unsafe {
        // 设置PendSV中断为最低优先级
        let nvic_syspri2 = NVIC_SYSPRI2 as *mut u32;
        let temp = core::ptr::read_volatile(nvic_syspri2);
        core::ptr::write_volatile(nvic_syspri2, temp | NVIC_PENDSV_PRI);
        
        // 确保PendSV中断已启用
        let nvic_iser = 0xE000E100 as *mut u32;  // NVIC ISER0
        let pendsvbit = 1 << 14;  // PendSV位于位置14
        let current = core::ptr::read_volatile(nvic_iser);
        core::ptr::write_volatile(nvic_iser, current | pendsvbit);
        
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
    // hprintln!("rt_hw_context_switch: from_sp: {:#x}, to_sp: {:#x}", unsafe { *from_sp }, unsafe { *to_sp });
    // 同步更新内部变量和兼容变量
    update_thread_vars(from_sp, to_sp);
    
    // 触发PendSV中断
    unsafe {
        asm!(
            "ldr r0, ={nvic_int_ctrl}",
            "ldr r1, ={nvic_pendsvset}",
            "str r1, [r0]",
            nvic_int_ctrl = const NVIC_INT_CTRL,
            nvic_pendsvset = const NVIC_PENDSVSET,
            out("r0") _,
            out("r1") _,
            options(nostack, preserves_flags)
        );
    }
}

/// 中断中的线程上下文切换 (在Cortex-M4上与普通切换相同)
#[inline]
pub fn rt_hw_context_switch_interrupt(from_sp: *mut u32, to_sp: *mut u32) {
    rt_hw_context_switch(from_sp, to_sp)
}

/// 直接切换到指定线程（不保存当前上下文）
pub fn rt_hw_context_switch_to(to_sp: *mut u32) {
    // 设置目标线程指针和标志
    NEXT_THREAD_SP.store(unsafe { *to_sp }, Ordering::SeqCst);
    CURRENT_THREAD_SP.store(0, Ordering::SeqCst);
    // THREAD_SWITCH_FLAG.store(1, Ordering::SeqCst);
    
    // 同步更新兼容变量
    unsafe {
        rt_interrupt_to_thread = to_sp;
        rt_interrupt_from_thread = 0 as *mut u32;
        rt_thread_switch_interrupt_flag = 1;
    }
    // 设置PendSV优先级和触发PendSV
    unsafe {
        // 设置PendSV优先级
        let nvic_syspri2 = NVIC_SYSPRI2 as *mut u32;
        let temp = core::ptr::read_volatile(nvic_syspri2);
        core::ptr::write_volatile(nvic_syspri2, temp | NVIC_PENDSV_PRI);
        
        // 触发PendSV
        let nvic_int_ctrl = NVIC_INT_CTRL as *mut u32;
        core::ptr::write_volatile(nvic_int_ctrl, NVIC_PENDSVSET);

        // 恢复MSP
        let vtor = core::ptr::read_volatile(SCB_VTOR as *const u32);
        let reset_sp = core::ptr::read_volatile(vtor as *const u32);

        asm!(
            // 恢复MSP（主栈指针）
            "msr msp, {0}",
            
            // 启用中断
            "cpsie f",
            "cpsie i",
            "dsb",
            "isb",
            
            in(reg) reset_sp
        );
        
    }
        
    // 如果到达这里，说明出现了问题
    hprintln!("ERROR: should not reach here!");
    loop {}
}

/// PendSV中断处理函数 - 进行实际的上下文切换
#[cortex_m_rt::exception]
unsafe fn PendSV()  {

    // 保存编译器的帧指针
    let saved_r7: u32;
    let r0: u32;
    unsafe {
        asm!(
            "mov {0}, r7",
            out(reg) saved_r7,
        );
    }

    unsafe {
        asm!(
            // 保存中断状态
            "mrs r2, PRIMASK",
            "cpsid i",      // 关闭中断
            
            // 获取切换标志
            "ldr r0, =rt_thread_switch_interrupt_flag",
            "ldr r1, [r0]",
            "cbz r1, 2f",         // 如果标志为0，跳到退出

            // 清除切换标志
            "mov r1, #0",
            "str r1, [r0]",
            
            // 获取当前线程栈指针
            "ldr r0, =rt_interrupt_from_thread",
            "ldr r1, [r0]",    
            "cbz r1, 1f",         // 如果为0，跳到恢复目标线程
            "ldr r1, [r1]",

            // 保存当前线程上下文
            "mrs r1, psp",        // 获取PSP
        );

        // FPU寄存器保存 (如果需要)
        #[cfg(feature = "fpu")]
        asm!(
            "tst lr, #0x10",
            "it eq",
            "vstmdbeq r1!, {{d8-d15}}",
        );

        // 保存通用寄存器
        asm!(
            "stmfd r1!, {{r4-r11}}",
        );

        // FPU标志保存
        #[cfg(feature = "fpu")]
        asm!(
            "mov r4, #0",
            "tst lr, #0x10",
            "it eq",
            "moveq r4, #1",
            "stmfd r1!, {{r4}}",
        );

        asm!(
            // 更新线程栈指针
            "ldr r0, [r0]",
            "str r1, [r0]",

            // 切换到目标线程
            "1:",
            "ldr r1, =rt_interrupt_to_thread",
            "ldr r1, [r1]",        
            "ldr r1, [r1]",        // 此时r1保存的是栈顶的地址 

        );

        // 恢复FPU标志
        #[cfg(feature = "fpu")]
        asm!(
            "ldmfd r1!, {{r3}}",
        );

        asm!(
            // 恢复通用寄存器
            "ldmfd r1!, {{r4-r11}}",
        );
        
        // 恢复FPU寄存器 (如果需要)
        #[cfg(feature = "fpu")]
        asm!(
            "cmp r3, #0",
            "it ne",
            "vldmiane r1!, {{d8-d15}}",
        );

        asm!(
            // 更新PSP
            "msr psp, r1",
            // 确保使用PSP返回
            "ldr lr, =0xFFFFFFFD",
        );

        // 处理FPU状态
        #[cfg(feature = "fpu")]
        asm!(
            "orr lr, lr, #0x10",  // 默认清除FPCA
            "cmp r3, #0",
            "it ne",
            "bicne lr, lr, #0x10", // 如果需要，设置FPCA
        );

        asm!(
            // 退出
            "2:",
            "msr PRIMASK, r2",    // 恢复中断状态
        );

        asm!(
            // 恢复编译器的帧指针
            "mov r7, {0}",
            in(reg) saved_r7,
        );

        asm!(
            "bx lr",
            options(noreturn)
        );
    }
}

/// 更新线程变量 - 同时更新原子变量和兼容变量
fn update_thread_vars(from_sp: *mut u32, to_sp: *mut u32) {
    // let prev_flag = THREAD_SWITCH_FLAG.load(Ordering::SeqCst);
    let prev_flag: u32;
    unsafe {
        prev_flag = rt_thread_switch_interrupt_flag;
    }
    // 只有在没有切换进行时才更新from_sp
    if prev_flag == 0 {
        // THREAD_SWITCH_FLAG.store(1, Ordering::SeqCst);
        CURRENT_THREAD_SP.store(unsafe { *from_sp }, Ordering::SeqCst);
        
        // 更新兼容变量
        unsafe {
            rt_thread_switch_interrupt_flag = 1;
            rt_interrupt_from_thread = from_sp;
            // hprintln!("update_thread_vars: from_sp: {:#x}", from_sp);
        }
    }
    
    // 始终更新to_sp
    NEXT_THREAD_SP.store(unsafe { *to_sp }, Ordering::SeqCst);
    
    // 更新兼容变量
    unsafe {
        rt_interrupt_to_thread = to_sp;
    }
}

/// 硬件错误处理函数
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HardFault_Handler() {
    #[cfg(feature = "debug")]
    hprintln!("HardFault_Handler entered");
    
    unsafe {
        asm!(
            // 获取上下文
            "mrs r0, msp",
            "tst lr, #4",
            "it eq",
            "mrseq r0, psp",
            
            // 保存寄存器
            "stmfd r0!, {{r4-r11}}",
            "stmfd r0!, {{lr}}",
            
            // 更新栈指针
            "tst lr, #4",
            "it eq",
            "msreq msp, r0",
            "it ne",
            "msrne psp, r0",
            
            // 调用处理函数
            "bl rt_hw_hard_fault_exception",
            
            // 返回
            "orr lr, lr, #0x04",
            "bx lr",
            
            options(noreturn)
        );
    }
    hprintln!("HardFault_Handler exited");
}

/// 硬件错误处理实现
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rt_hw_hard_fault_exception(context: *mut core::ffi::c_void) {
    #[cfg(feature = "debug")]
    hprintln!("HARDFAULT! System halted.");
    
    loop {}
}

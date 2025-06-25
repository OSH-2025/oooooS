//! 异常处理相关函数-Cortex-M4
//! 
//! 定义了异常处理函数

#![warn(unused_imports)]

use cortex_m_rt::{exception};
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use crate::rtthread_rt::rtdef::*;
use cortex_m_semihosting::hprintln;
use core::fmt;
use core::fmt::Debug;
use crate::rtthread_rt::hardware::cpuport::StackFrame;

// 异常钩子
static mut RT_EXCEPTION_HOOK: Option<unsafe fn(context: *mut core::ffi::c_void) -> i32> = None;

// 异常信息结构体
#[repr(C)]
pub struct ExceptionInfo {
    pub exc_return: u32,
    pub stack_frame: StackFrame,
}

// 安装异常钩子
pub unsafe fn rt_hw_exception_install(hook: unsafe fn(context: *mut core::ffi::c_void) -> i32) {
    RT_EXCEPTION_HOOK = Some(hook);
}

// 寄存器地址常量
const SCB_CFSR: *const u32 = 0xE000ED28 as *const u32;
const SCB_HFSR: *const u32 = 0xE000ED2C as *const u32;
const SCB_MMAR: *const u32 = 0xE000ED34 as *const u32;
const SCB_BFAR: *const u32 = 0xE000ED38 as *const u32;
const SCB_CFSR_MFSR: *const u8 = 0xE000ED28 as *const u8;
const SCB_CFSR_BFSR: *const u8 = 0xE000ED29 as *const u8;
const SCB_CFSR_UFSR: *const u16 = 0xE000ED2A as *const u16;


/// 异常处理
#[exception]
fn MemoryManagement() -> ! {
    unsafe {
        mem_manage_fault_track();
    }
    loop {}
}

#[exception]
fn BusFault() -> ! {
    unsafe {
        bus_fault_track();
    }
    loop {}
}

#[exception]
fn UsageFault() -> ! {
    unsafe {
        usage_fault_track();
    }
    loop {}
}

#[exception] 
unsafe fn DefaultHandler(irqn: i16) {
    unsafe {
        hard_fault_track();
    }
    loop {}
}


// 打印函数（可根据实际平台替换为rt_kprintf等）
fn print(msg: &str) {
    // 这里简单用Rust的打印，嵌入式可替换为串口输出
    #[cfg(not(target_os = "none"))]
    {
        println!("{}", msg);
    }
}
fn print_fmt(args: core::fmt::Arguments) {
    #[cfg(not(target_os = "none"))]
    {
        use core::fmt::Write;
        let _ = std::io::stdout().write_fmt(args);
    }
}
macro_rules! kprintf {
    ($($arg:tt)*) => {
        print_fmt(format_args!($($arg)*));
    };
}

// usage fault 跟踪
unsafe fn usage_fault_track() {
    let ufsr = core::ptr::read_volatile(SCB_CFSR_UFSR);
    kprintf!("usage fault:\n");
    kprintf!("SCB_CFSR_UFSR:0x{:02X} ", ufsr);
    if ufsr & (1 << 0) != 0 { kprintf!("UNDEFINSTR "); }
    if ufsr & (1 << 1) != 0 { kprintf!("INVSTATE "); }
    if ufsr & (1 << 2) != 0 { kprintf!("INVPC "); }
    if ufsr & (1 << 3) != 0 { kprintf!("NOCP "); }
    if ufsr & (1 << 8) != 0 { kprintf!("UNALIGNED "); }
    if ufsr & (1 << 9) != 0 { kprintf!("DIVBYZERO "); }
    kprintf!("\n");
}

// bus fault 跟踪
unsafe fn bus_fault_track() {
    let bfsr = core::ptr::read_volatile(SCB_CFSR_BFSR);
    kprintf!("bus fault:\n");
    kprintf!("SCB_CFSR_BFSR:0x{:02X} ", bfsr);
    if bfsr & (1 << 0) != 0 { kprintf!("IBUSERR "); }
    if bfsr & (1 << 1) != 0 { kprintf!("PRECISERR "); }
    if bfsr & (1 << 2) != 0 { kprintf!("IMPRECISERR "); }
    if bfsr & (1 << 3) != 0 { kprintf!("UNSTKERR "); }
    if bfsr & (1 << 4) != 0 { kprintf!("STKERR "); }
    if bfsr & (1 << 7) != 0 {
        let bfar = core::ptr::read_volatile(SCB_BFAR);
        kprintf!("SCB->BFAR:{:08X}\n", bfar);
    } else {
        kprintf!("\n");
    }
}

// mem manage fault 跟踪
unsafe fn mem_manage_fault_track() {
    let mfsr = core::ptr::read_volatile(SCB_CFSR_MFSR);
    kprintf!("mem manage fault:\n");
    kprintf!("SCB_CFSR_MFSR:0x{:02X} ", mfsr);
    if mfsr & (1 << 0) != 0 { kprintf!("IACCVIOL "); }
    if mfsr & (1 << 1) != 0 { kprintf!("DACCVIOL "); }
    if mfsr & (1 << 3) != 0 { kprintf!("MUNSTKERR "); }
    if mfsr & (1 << 4) != 0 { kprintf!("MSTKERR "); }
    if mfsr & (1 << 7) != 0 {
        let mmar = core::ptr::read_volatile(SCB_MMAR);
        kprintf!("SCB->MMAR:{:08X}\n", mmar);
    } else {
        kprintf!("\n");
    }
}

// hard fault 跟踪
unsafe fn hard_fault_track() {
    let hfsr = core::ptr::read_volatile(SCB_HFSR);
    if hfsr & (1 << 1) != 0 {
        kprintf!("failed vector fetch\n");
    }
    if hfsr & (1 << 30) != 0 {
        let bfsr = core::ptr::read_volatile(SCB_CFSR_BFSR);
        let mfsr = core::ptr::read_volatile(SCB_CFSR_MFSR);
        let ufsr = core::ptr::read_volatile(SCB_CFSR_UFSR);
        if bfsr != 0 { bus_fault_track(); }
        if mfsr != 0 { mem_manage_fault_track(); }
        if ufsr != 0 { usage_fault_track(); }
    }
    if hfsr & (1 << 31) != 0 {
        kprintf!("debug event\n");
    }
}

// 详细异常寄存器打印和故障跟踪
pub unsafe fn rt_hw_hard_fault_exception(exception_info: *mut ExceptionInfo) {
    let context = &(*exception_info).stack_frame;
    // 调用异常钩子
    if let Some(hook) = RT_EXCEPTION_HOOK {
        let result = hook(&context.exception_stack_frame as *const _ as *mut core::ffi::c_void);
        if result == 0 { // 假定0为RT_EOK
            return;
        }
    }
    kprintf!("psr: 0x{:08x}\n", context.exception_stack_frame.psr);
    kprintf!("r00: 0x{:08x}\n", context.exception_stack_frame.r0);
    kprintf!("r01: 0x{:08x}\n", context.exception_stack_frame.r1);
    kprintf!("r02: 0x{:08x}\n", context.exception_stack_frame.r2);
    kprintf!("r03: 0x{:08x}\n", context.exception_stack_frame.r3);
    kprintf!("r04: 0x{:08x}\n", context.r4);
    kprintf!("r05: 0x{:08x}\n", context.r5);
    kprintf!("r06: 0x{:08x}\n", context.r6);
    kprintf!("r07: 0x{:08x}\n", context.r7);
    kprintf!("r08: 0x{:08x}\n", context.r8);
    kprintf!("r09: 0x{:08x}\n", context.r9);
    kprintf!("r10: 0x{:08x}\n", context.r10);
    kprintf!("r11: 0x{:08x}\n", context.r11);
    kprintf!("r12: 0x{:08x}\n", context.exception_stack_frame.r12);
    kprintf!(" lr: 0x{:08x}\n", context.exception_stack_frame.lr);
    kprintf!(" pc: 0x{:08x}\n", context.exception_stack_frame.pc);
    // 线程名等可根据实际RTOS API补充
    if ((*exception_info).exc_return & (1 << 2)) != 0 {
        kprintf!("hard fault on thread: <thread_name_placeholder>\n\n");
        // list_thread(); // 需要RTOS支持，可补充
    } else {
        kprintf!("hard fault on handler\n\n");
    }
    // FPU状态
    if ((*exception_info).exc_return & 0x10) == 0 {
        kprintf!("FPU active!\r\n");
    }
    hard_fault_track();
    loop {}
}




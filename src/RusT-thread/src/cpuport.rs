#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

// 类型定义
pub type RtUint32 = u32;
pub type RtUint8 = u8;
pub type RtErr = i32;

// 异常栈帧
#[repr(C)]
pub struct ExceptionStackFrame {
    pub r0: RtUint32,
    pub r1: RtUint32,
    pub r2: RtUint32,
    pub r3: RtUint32,
    pub r12: RtUint32,
    pub lr: RtUint32,
    pub pc: RtUint32,
    pub psr: RtUint32,
}

#[repr(C)]
pub struct StackFrame {
    #[cfg(feature = "fpu")] // FPU相关
    pub flag: RtUint32,
    pub r4: RtUint32,
    pub r5: RtUint32,
    pub r6: RtUint32,
    pub r7: RtUint32,
    pub r8: RtUint32,
    pub r9: RtUint32,
    pub r10: RtUint32,
    pub r11: RtUint32,
    pub exception_stack_frame: ExceptionStackFrame,
}

// 栈初始化函数
pub unsafe fn rt_hw_stack_init(
    tentry: usize,
    parameter: *mut u8,
    stack_addr: *mut RtUint8,
    texit: usize,
) -> *mut RtUint8 {
    // 栈顶对齐到8字节
    let mut stk = stack_addr.add(core::mem::size_of::<RtUint32>()) as usize;
    stk &= !0x7;
    stk -= core::mem::size_of::<StackFrame>();
    let stack_frame = stk as *mut StackFrame;

    // 初始化所有寄存器为0xdeadbeef
    let p = stack_frame as *mut RtUint32;
    for i in 0..(core::mem::size_of::<StackFrame>() / 4) {
        ptr::write(p.add(i), 0xdeadbeef);
    }

    // 填充异常栈帧
    (*stack_frame).exception_stack_frame.r0 = parameter as RtUint32;
    (*stack_frame).exception_stack_frame.r1 = 0;
    (*stack_frame).exception_stack_frame.r2 = 0;
    (*stack_frame).exception_stack_frame.r3 = 0;
    (*stack_frame).exception_stack_frame.r12 = 0;
    (*stack_frame).exception_stack_frame.lr = texit as RtUint32;
    (*stack_frame).exception_stack_frame.pc = tentry as RtUint32;
    (*stack_frame).exception_stack_frame.psr = 0x01000000;

    #[cfg(feature = "fpu")]
    {
        (*stack_frame).flag = 0;
    }

    stk as *mut RtUint8
}


/* exception and interrupt handler table */
#[unsafe(no_mangle)]
pub static mut rt_interrupt_from_thread: RtUint32 = 0;
#[unsafe(no_mangle)]
pub static mut rt_interrupt_to_thread: RtUint32 = 0;
#[unsafe(no_mangle)]
pub static mut rt_thread_switch_interrupt_flag: RtUint32 = 0;

// 异常钩子
static mut RT_EXCEPTION_HOOK: Option<unsafe fn(context: *mut core::ffi::c_void) -> RtErr> = None;

// 异常信息结构体
#[repr(C)]
pub struct ExceptionInfo {
    pub exc_return: RtUint32,
    pub stack_frame: StackFrame,
}

// 安装异常钩子
pub unsafe fn rt_hw_exception_install(hook: unsafe fn(context: *mut core::ffi::c_void) -> RtErr) {
    RT_EXCEPTION_HOOK = Some(hook);
}

// CPU关机
#[unsafe(no_mangle)]
pub extern "C" fn rt_hw_cpu_shutdown() {
    // 这里用Rust的panic代替死循环断言
    panic!("shutdown...");
}

// CPU重启
const SCB_AIRCR: *mut u32 = 0xE000ED0C as *mut u32;
const SCB_RESET_VALUE: u32 = 0x05FA0004;

#[unsafe(no_mangle)]
pub extern "C" fn rt_hw_cpu_reset() {
    unsafe {
        core::ptr::write_volatile(SCB_AIRCR, SCB_RESET_VALUE);
    }
}

// FPU支持相关结构体（简化版）
#[cfg(feature = "fpu")]
#[repr(C)]
pub struct ExceptionStackFrameFpu {
    pub r0: RtUint32,
    pub r1: RtUint32,
    pub r2: RtUint32,
    pub r3: RtUint32,
    pub r12: RtUint32,
    pub lr: RtUint32,
    pub pc: RtUint32,
    pub psr: RtUint32,
    pub s0: RtUint32,
    pub s1: RtUint32,
    pub s2: RtUint32,
    pub s3: RtUint32,
    pub s4: RtUint32,
    pub s5: RtUint32,
    pub s6: RtUint32,
    pub s7: RtUint32,
    pub s8: RtUint32,
    pub s9: RtUint32,
    pub s10: RtUint32,
    pub s11: RtUint32,
    pub s12: RtUint32,
    pub s13: RtUint32,
    pub s14: RtUint32,
    pub s15: RtUint32,
    pub fpscr: RtUint32,
    pub no_name: RtUint32,
}

#[cfg(feature = "fpu")]
#[repr(C)]
pub struct StackFrameFpu {
    pub flag: RtUint32,
    pub r4: RtUint32,
    pub r5: RtUint32,
    pub r6: RtUint32,
    pub r7: RtUint32,
    pub r8: RtUint32,
    pub r9: RtUint32,
    pub r10: RtUint32,
    pub r11: RtUint32,
    pub s16: RtUint32,
    pub s17: RtUint32,
    pub s18: RtUint32,
    pub s19: RtUint32,
    pub s20: RtUint32,
    pub s21: RtUint32,
    pub s22: RtUint32,
    pub s23: RtUint32,
    pub s24: RtUint32,
    pub s25: RtUint32,
    pub s26: RtUint32,
    pub s27: RtUint32,
    pub s28: RtUint32,
    pub s29: RtUint32,
    pub s30: RtUint32,
    pub s31: RtUint32,
    pub exception_stack_frame: ExceptionStackFrameFpu,
}

// FFS实现
#[inline]
pub fn __rt_ffs(value: i32) -> i32 {
    if value == 0 {
        0
    } else {
        value.trailing_zeros() as i32 + 1
    }
}

// 寄存器地址常量
const SCB_CFSR: *const u32 = 0xE000ED28 as *const u32;
const SCB_HFSR: *const u32 = 0xE000ED2C as *const u32;
const SCB_MMAR: *const u32 = 0xE000ED34 as *const u32;
const SCB_BFAR: *const u32 = 0xE000ED38 as *const u32;
const SCB_CFSR_MFSR: *const u8 = 0xE000ED28 as *const u8;
const SCB_CFSR_BFSR: *const u8 = 0xE000ED29 as *const u8;
const SCB_CFSR_UFSR: *const u16 = 0xE000ED2A as *const u16;

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


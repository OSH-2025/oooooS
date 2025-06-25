// ! CPU接口-Cortex-M4
// ! 
// ! 定义了异常栈帧、栈帧、CPU关机、CPU重启
// ! 并给出了对应的hw_stack_init函数(栈初始化函数)

#![warn(unused_imports)]

use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use crate::rtthread_rt::rtdef::*;
use cortex_m_semihosting::hprintln;
use core::fmt;
use core::fmt::Debug;

// 异常栈帧
#[repr(C)]
pub struct ExceptionStackFrame {
    pub r0: u32,    // 线程入口参数
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,    // 线程退出处理函数
    pub pc: u32,    // 线程入口函数
    pub psr: u32,   // xPSR, 必须设置Thumb位
}

impl Debug for ExceptionStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExceptionStackFrame {{ r0: {:x}, r1: {:x}, r2: {:x}, r3: {:x}, r12: {:x}, lr: {:x}, pc: {:x}, psr: {:x} }}", self.r0, self.r1, self.r2, self.r3, self.r12, self.lr, self.pc, self.psr)
    }
}

#[repr(C)]
pub struct StackFrame {
    #[cfg(feature = "fpu")]
    pub flag: u32,  // FPU相关
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub exception_stack_frame: ExceptionStackFrame,
}

impl Debug for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "fpu")]
        {
            write!(f, "StackFrame {{ flag: {:x}, r4: {:x}, r5: {:x}, r6: {:x}, r7: {:x}, r8: {:x}, r9: {:x}, r10: {:x}, r11: {:x}, exception_stack_frame: {:?} }}", 
                self.flag, self.r4, self.r5, self.r6, self.r7, self.r8, self.r9, self.r10, self.r11, self.exception_stack_frame)
        }
        #[cfg(not(feature = "fpu"))]
        {
            write!(f, "StackFrame {{ r4: {:x}, r5: {:x}, r6: {:x}, r7: {:x}, r8: {:x}, r9: {:x}, r10: {:x}, r11: {:x}, exception_stack_frame: {:?} }}", 
                self.r4, self.r5, self.r6, self.r7, self.r8, self.r9, self.r10, self.r11, self.exception_stack_frame)
        }
    }
}

// 栈初始化函数
pub unsafe fn rt_hw_stack_init(
    tentry: usize,
    parameter: *mut u8,
    stack_addr: usize,
    texit: usize,
) -> usize {
    // 栈顶对齐到8字节
    let mut stk = stack_addr + core::mem::size_of::<u32>();
    stk &= !0x7;
    stk -= core::mem::size_of::<StackFrame>();

    // hprintln!("rt_hw_stack_init: stk: {:x}", stk);
    let stack_frame = stk as *mut StackFrame;

    // 初始化所有寄存器为0xdeadbeef
    let p = stack_frame as *mut u32;
    for i in 0..(core::mem::size_of::<StackFrame>() / 4) {
        ptr::write(p.add(i), 0xdeadbeef);
    }

    // 填充异常栈帧
    (*stack_frame).exception_stack_frame.r0 = parameter as u32;
    (*stack_frame).exception_stack_frame.r1 = 0;
    (*stack_frame).exception_stack_frame.r2 = 0;
    (*stack_frame).exception_stack_frame.r3 = 0;
    (*stack_frame).exception_stack_frame.r12 = 0;
    (*stack_frame).exception_stack_frame.lr = texit as u32;
    (*stack_frame).exception_stack_frame.pc = tentry as u32;
    (*stack_frame).exception_stack_frame.psr = 0x01000000;

    #[cfg(feature = "fpu")]
    {
        (*stack_frame).flag = 0;
    }

    // hprintln!("stack_frame: {:?}", *stack_frame);

    stk
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
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,
    pub pc: u32,
    pub psr: u32,
    pub s0: u32,
    pub s1: u32,
    pub s2: u32,
    pub s3: u32,
    pub s4: u32,
    pub s5: u32,
    pub s6: u32,
    pub s7: u32,
    pub s8: u32,
    pub s9: u32,
    pub s10: u32,
    pub s11: u32,
    pub s12: u32,
    pub s13: u32,
    pub s14: u32,
    pub s15: u32,
    pub fpscr: u32,
    pub no_name: u32,
}

#[cfg(feature = "fpu")]
#[repr(C)]
pub struct StackFrameFpu {
    pub flag: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub s16: u32,
    pub s17: u32,
    pub s18: u32,
    pub s19: u32,
    pub s20: u32,
    pub s21: u32,
    pub s22: u32,
    pub s23: u32,
    pub s24: u32,
    pub s25: u32,
    pub s26: u32,
    pub s27: u32,
    pub s28: u32,
    pub s29: u32,
    pub s30: u32,
    pub s31: u32,
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

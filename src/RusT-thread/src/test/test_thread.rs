/// 测试线程
/// 

//-------------测试1：上下文切换--------------------------------

use crate::rtthread::thread::*;
use crate::context::*;
use crate::cpuport::*;
use cortex_m_semihosting::hprintln;
use core::arch::asm;
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicU32, Ordering};

pub extern "C" fn thread1(arg: usize) -> () {
    hprintln!("thread1: {}", arg);
    unsafe {
        let sp1_val = SP1 as u32;
        let sp2_val = SP2 as u32;
        hprintln!("sp1_val: {:#x}", sp1_val);
        hprintln!("sp2_val: {:#x}", sp2_val);
        // 切换到线程2
        rt_hw_context_switch(&raw const SP1 as *mut u32, &raw const SP2 as *mut u32);
    }
}

pub extern "C" fn thread2(arg: usize) -> () {
    hprintln!("thread2: {}", arg);
    unsafe {
        let sp1_val = SP1 as u32;
        let sp2_val = SP2 as u32;
        hprintln!("sp1_val: {:#x}", sp1_val);
        hprintln!("sp2_val: {:#x}", sp2_val);
        rt_hw_context_switch(&raw const SP2 as *mut u32, &raw const SP1 as *mut u32);
    }
}

pub fn test_func_pointer() {
    // print
    hprintln!("test_func_pointer");
    let func_ptr = thread1 as usize;
    hprintln!("func_ptr: {:#x}", func_ptr);

    // try to call the function
    unsafe {
        let func: fn(usize) -> () = core::mem::transmute(func_ptr);
        hprintln!("Calling thread1...");
        func(12384);
    }
    hprintln!("test_func_pointer done");
}

pub fn test_thread_context_switch() {
    hprintln!("test_thread_context_switch");
    // 创建线程栈1
    let kernel_stack1 = KernelStack::new(KERNEL_STACK_SIZE);
    // hprintln!("kernel_stack1: {:#x}", kernel_stack1.bottom());
    hprintln!("kernel_stack1: {:#x}", kernel_stack1.top());
    // 初始化线程1的栈
    unsafe {
        let sp = rt_hw_stack_init(
            thread1 as usize,
            0 as *mut u8,
            kernel_stack1.top() as usize,
            0 as usize
        );


        hprintln!("sp: {:#x}", sp);

        // 切换到线程1
        rt_hw_context_switch_to(&raw const sp as *mut u32);
    }
    hprintln!("kernel_stack1: switch");
}

// 删除 lazy_static 块
static mut SP1: usize = 0;
static mut SP2: usize = 0;

pub fn test_thread_context_switch_from_to() {
    hprintln!("test_thread_context_switch_from_to");
    // 创建线程栈1
    let kernel_stack1 = KernelStack::new(KERNEL_STACK_SIZE);
    // 创建线程栈2
    let kernel_stack2 = KernelStack::new(KERNEL_STACK_SIZE);
    // 初始化栈
    unsafe {
        SP1 = rt_hw_stack_init(
            thread1 as usize,
            0 as *mut u8,
            kernel_stack1.top() as usize,
            0 as usize
        );

        SP2 = rt_hw_stack_init(
            thread2 as usize,
            0 as *mut u8,
            kernel_stack2.top() as usize,
            0 as usize
        );

        // 切换到线程1
        rt_hw_context_switch_to(&raw const SP1 as *mut u32);
    }
}
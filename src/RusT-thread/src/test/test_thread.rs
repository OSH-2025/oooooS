/// 测试线程
/// 

//-------------测试1：上下文切换--------------------------------

use crate::rtthread::thread::*;
use crate::context::*;
use crate::cpuport::*;
use cortex_m_semihosting::hprintln;
use core::arch::asm;


pub extern "C" fn thread1(arg: usize) -> () {
    let mut i = 0;
    hprintln!("thread1: {}", i);
    // loop {
    //     i += 1;
    //     hprintln!("thread1: {}", i);
    // }
}

pub extern "C" fn thread2(arg: usize) -> ! {
    let mut i = 0;
    loop {
        i += 1;
        hprintln!("thread2: {}", i);
    }
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
        rt_hw_context_switch_to(&sp as *const usize as *mut u32);
    }
    hprintln!("kernel_stack1: switch");
}




/// 测试线程
/// 

//-------------测试1：上下文切换--------------------------------


use cortex_m_semihosting::hprintln;
// use core::arch::asm;
// use lazy_static::lazy_static;
// use core::sync::atomic::{AtomicU32, Ordering};
use crate::rtthread_rt::rtdef::*;
use crate::rtthread_rt::rtconfig::*;
use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::hardware::*;

pub extern "C" fn thread1(arg: usize) -> () {
    let mut i = arg;
    hprintln!("thread1: {}", i);
    rt_hw_context_switch(&raw const SP1 as *mut u32, &raw const SP2 as *mut u32);
    i += 1;
    hprintln!("thread1: {}", i);
    rt_hw_context_switch(&raw const SP1 as *mut u32, &raw const SP2 as *mut u32);
    i += 1;
    hprintln!("thread1: {}", i);
    rt_hw_context_switch(&raw const SP1 as *mut u32, &raw const SP2 as *mut u32);
}

pub extern "C" fn thread2(arg: usize) -> () {
    let mut i = arg;
    hprintln!("thread2: {}", i);
    rt_hw_context_switch(&raw const SP2 as *mut u32, &raw const SP1 as *mut u32);
    i += 1;
    hprintln!("thread2: {}", i);
    rt_hw_context_switch(&raw const SP2 as *mut u32, &raw const SP1 as *mut u32);
    i += 1;
    hprintln!("thread2: {}", i);
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

// 测试线程创建和基本操作
pub fn test_thread_create_and_control() {
    hprintln!("开始测试线程创建和控制...");
    
    // 测试线程创建
    let thread = rt_thread_create(
        "test1",
        thread1 as usize,
        KERNEL_STACK_SIZE,
        5,
        10
    );
    // 验证线程创建是否成功
    assert!(thread.inner.exclusive_access().stat.get_stat() == (ThreadState::Init as u8), "线程创建状态错误");
    assert!(thread.inner.exclusive_access().current_priority == 5, "线程优先级设置错误");
    
    hprintln!("线程创建成功");
    // 测试线程启动
    let result = rt_thread_startup(thread.clone());
    hprintln!("线程启动结果: {:?}", result);
    assert!(result == RT_EOK, "线程启动失败");
    
    hprintln!("线程创建和控制测试完成");
}

// 测试线程挂起和恢复
pub fn test_thread_suspend_resume() {
    hprintln!("开始测试线程挂起和恢复...");
    
    // 创建测试线程
    let thread = rt_thread_create(
        "test2",
        thread2 as usize,
        KERNEL_STACK_SIZE,
        6,
        10
    );
    
    // 启动线程
    rt_thread_startup(thread.clone());
    
    // 测试挂起
    let result = rt_thread_suspend(thread.clone());
    assert!(result == RT_EOK, "线程挂起失败");
    assert!(thread.inner.exclusive_access().stat.get_stat() == (ThreadState::Suspend as u8), "线程挂起状态错误");
    
    // 测试恢复
    let result = rt_thread_resume(thread.clone());
    assert!(result == RT_EOK, "线程恢复失败");
    
    hprintln!("线程挂起和恢复测试完成");
}

// 测试线程睡眠
pub fn test_thread_sleep() {
    hprintln!("开始测试线程睡眠...");
    
    // 创建测试线程
    let thread = rt_thread_create(
        "test3",
        thread1 as usize,
        KERNEL_STACK_SIZE,
        7,
        10
    );
    
    // 启动线程
    rt_thread_startup(thread.clone());
    
    // 测试睡眠
    let result = rt_thread_sleep(thread.clone(), 100);
    assert!(result == RT_EOK, "线程睡眠失败");
    
    hprintln!("线程睡眠测试完成");
}

// 测试线程优先级修改
pub fn test_thread_priority() {
    hprintln!("开始测试线程优先级修改...");
    
    // 创建测试线程
    let thread = rt_thread_create(
        "test4",
        thread2 as usize,
        KERNEL_STACK_SIZE,
        8,
        10
    );
    
    // 启动线程
    rt_thread_startup(thread.clone());
    
    // 测试优先级修改
    let result = rt_thread_control(thread.clone(), RT_THREAD_CTRL_CHANGE_PRIORITY, 3);
    assert!(result == RT_EOK, "线程优先级修改失败");
    assert!(thread.inner.exclusive_access().current_priority == 3, "线程优先级未正确更新");
    
    hprintln!("线程优先级修改测试完成");
}

// 运行所有线程测试
pub fn run_all_thread_tests() {
    hprintln!("开始运行所有线程测试...");
    
    test_thread_create_and_control();
    test_thread_suspend_resume();
    test_thread_sleep();
    test_thread_priority();
    test_thread_context_switch();
    test_thread_context_switch_from_to();
    
    hprintln!("所有线程测试完成！");
}
use cortex_m_semihosting::hprintln;

/// 测试内存管理异常
/// 通过访问无效内存地址触发
pub fn test_memory_management() {
    hprintln!("Testing Memory Management Exception...");
    unsafe {
        // 尝试访问一个无效的内存地址
        let _ptr = 0x0 as *const u32;
        let _value = *_ptr; // 这里会触发 MemoryManagement 异常
    }
}

/// 测试总线错误异常
/// 通过访问未对齐的内存地址触发
pub fn test_bus_fault() {
    hprintln!("Testing Bus Fault Exception...");
    unsafe {
        // 尝试访问一个未对齐的内存地址
        let ptr = 0x1 as *const u32;
        let _value = *ptr; // 这里会触发 BusFault 异常
    }
}

/// 测试用法错误异常
/// 通过执行未定义的指令触发
pub fn test_usage_fault() {
    hprintln!("Testing Usage Fault Exception...");
    unsafe {
        // 执行未定义的指令
        core::arch::asm!("udf #0");
    }
}



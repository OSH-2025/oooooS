/// 简化的小内存管理测试模块
/// 从main.rs移过来的测试内容

use crate::mem::mem::*;
use cortex_m_semihosting::hprintln;

/// 运行简化的小内存管理测试
pub fn run_simple_mem_tests() {
    hprintln!("🧪 开始简化的小内存管理测试");
    hprintln!("==============================");
    
    test_simple_mem_basic();
    
    hprintln!("🎯 简化测试完成！");
    hprintln!("==============================\n");
}

/// 基本的内存管理测试
pub fn test_simple_mem_basic() {
    hprintln!("=== 测试基本内存管理 ===");
    
    hprintln!("创建测试缓冲区...");
    let mut test_heap = [0u8; 1024]; // 1KB 缓冲区
    hprintln!("测试缓冲区创建完成");
    
    unsafe {
        hprintln!("初始化小内存管理器...");
        let mem_ptr = rt_smem_init("simple_test", test_heap.as_mut_ptr(), 1024);
        
        if mem_ptr.is_null() {
            hprintln!("❌ 内存管理器初始化失败");
        } else {
            hprintln!("✅ 内存管理器初始化成功");
            
            // 尝试分配内存
            hprintln!("尝试分配32字节...");
            let ptr = rt_smem_alloc(mem_ptr, 32);
            if !ptr.is_null() {
                hprintln!("✅ 成功分配32字节");
                
                // 释放内存
                hprintln!("释放内存...");
                rt_smem_free(ptr);
                hprintln!("✅ 内存释放成功");
            } else {
                hprintln!("❌ 内存分配失败");
            }
        }
    }
    
    hprintln!("=== 基本内存管理测试完成 ===\n");
} 
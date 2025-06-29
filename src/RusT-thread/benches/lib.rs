//! RT-Thread 内存管理基准测试库
//! 
//! 这是一个独立的基准测试项目，用于测试 RT-Thread 内存管理算法的性能

pub mod memory;
pub mod allocator;

// 重新导出主要的类型和函数
pub use memory::*;
pub use allocator::*;

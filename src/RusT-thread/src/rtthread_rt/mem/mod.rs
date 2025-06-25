//! RT-Thread内存管理模块
//! 
//! 本模块提供了RT-Thread的内存管理功能。
//! 通过特性支持多种分配器实现：
//! 
//! - good_memory_allocator: 使用good_memory_allocator分配器
//! - buddy_system_allocator: 使用buddy_system_allocator分配器
//! 
//! 本模块还提供了与RT-Thread兼容的内存管理API。

pub mod allocator;  
pub mod small_mem_impl;
pub mod object;
pub mod small_mem_allocator;
pub mod safelist;


pub use self::small_mem_impl::{rt_smem_init, rt_smem_detach, rt_smem_alloc, rt_smem_realloc, rt_smem_free};
pub use self::small_mem_allocator::{MemAllocator, rt_mem_info};
pub use self::object::{rt_object_init, rt_object_detach, OBJECT_REGISTRY};
pub use self::safelist::{SafeRTList};
pub use self::allocator::{init_heap};
#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use lazy_static::lazy_static;
use spin::Mutex;

mod rtdef;
mod irq;
mod context;
mod rtthread;
mod kservice;
mod mem;
mod rtconfig;


#[entry]
fn main() -> ! {

    hprintln!("Hello, world!");

    loop {
        asm::nop();
    }    

/*
// // 定义一个测试用的堆内存区域
// #[repr(align(8))]
// struct TestHeap {
//     heap_memory: [u8; 8192], // 8KB大小的测试堆
// }

// // 使用lazy_static和Mutex替代可变静态引用
// lazy_static! {
//     static ref TEST_HEAP: Mutex<TestHeap> = Mutex::new(TestHeap { heap_memory: [0; 8192] });
// }

// #[entry]
// fn main() -> ! {
//     // 打印启动消息
//     let _ = hprintln!("Testing RT-Thread Memory Management in Rust...");
    
//     // 安全地访问TEST_HEAP，不再需要unsafe
//     let mem_pool = {
//         let mut test_heap = TEST_HEAP.lock();
//         mem::rt_mem_init(
//             "test_heap",
//             test_heap.heap_memory.as_mut_ptr(),
//             test_heap.heap_memory.len()
//         )
//     };
    
//     if mem_pool.is_null() {
//         let _ = hprintln!("Failed to initialize memory pool!");
//         loop {
//             asm::nop();
//         }
//     }
    
//     let _ = hprintln!("Memory pool initialized successfully!");
    
//     // 测试内存分配
//     let mut ptr1 = unsafe { mem::rt_malloc(mem_pool, 100) };
//     if ptr1.is_null() {
//         let _ = hprintln!("Failed to allocate 100 bytes!");
//     } else {
//         let _ = hprintln!("Successfully allocated 100 bytes at: {:p}", ptr1);
        
//         // 写入一些数据
//         unsafe {
//             for i in 0..100 {
//                 *ptr1.add(i) = i as u8;
//             }
            
//             // 读取并验证
//             let mut success = true;
//             for i in 0..100 {
//                 if *ptr1.add(i) != i as u8 {
//                     let _ = hprintln!("Data verification failed at index {}!", i);
//                     success = false;
//                     break;
//                 }
//             }
            
//             if success {
//                 let _ = hprintln!("Data verification successful!");
//             }
//         }
//     }
    
//     // 测试第二次分配
//     let ptr2 = unsafe { mem::rt_malloc(mem_pool, 200) };
//     if ptr2.is_null() {
//         let _ = hprintln!("Failed to allocate 200 bytes!");
//     } else {
//         let _ = hprintln!("Successfully allocated 200 bytes at: {:p}", ptr2);
//     }
    
//     // 测试内存重分配
//     if !ptr1.is_null() {
//         let new_ptr = unsafe { mem::rt_realloc(mem_pool, ptr1, 150) };
//         if new_ptr.is_null() {
//             let _ = hprintln!("Failed to reallocate memory!");
//         } else {
//             let _ = hprintln!("Successfully reallocated to 150 bytes at: {:p}", new_ptr);
            
//             // 检查数据是否保留
//             unsafe {
//                 let mut success = true;
//                 for i in 0..100 {
//                     if *new_ptr.add(i) != i as u8 {
//                         let _ = hprintln!("Data after reallocation is incorrect at index {}!", i);
//                         success = false;
//                         break;
//                     }
//                 }
                
//                 if success {
//                     let _ = hprintln!("Data preserved after reallocation!");
//                 }
//             }
            
//             // 使用新指针
//             ptr1 = new_ptr;
//         }
//     }
    
//     // 测试内存释放
//     if !ptr1.is_null() {
//         unsafe { mem::rt_free(ptr1) };
//         let _ = hprintln!("Freed memory at: {:p}", ptr1);
//     }
    
//     if !ptr2.is_null() {
//         unsafe { mem::rt_free(ptr2) };
//         let _ = hprintln!("Freed memory at: {:p}", ptr2);
//     }
    
//     // 再次分配，看是否能重用刚释放的内存
//     let ptr3 = unsafe { mem::rt_malloc(mem_pool, 50) };
//     if ptr3.is_null() {
//         let _ = hprintln!("Failed to allocate 50 bytes after freeing!");
//     } else {
//         let _ = hprintln!("Successfully allocated 50 bytes at: {:p} after freeing", ptr3);
//         unsafe { mem::rt_free(ptr3) };
//     }
    
//     let _ = hprintln!("Memory test completed!");

赵于洋 2025年5月11日*/
    
    // 进入无限循环

    // fn test_hook() {
    //     hprintln!("test hook");
    // }

    // irq::rt_interrupt_enter();
    // irq::rt_interrupt_leave();
    // irq::rt_interrupt_enter_sethook(test_hook);
    // irq::rt_interrupt_enter();
    // irq::rt_interrupt_leave();
}

//! 内核栈
//! 
//! 定义了内核栈的结构体和相关函数

#![warn(unused_imports)]

use lazy_static::lazy_static;

extern crate alloc;
use alloc::vec::Vec;
use spin::Mutex;

use crate::rtthread_rt::rtdef::*;
use crate::rtthread_rt::thread::*;
use crate::rtthread_rt::kservice::RTIntrFreeCell;
use crate::rtthread_rt::hardware::*;
use crate::rtthread_rt::timer::*;
use crate::rtthread_rt::rtconfig::*;

use core::fmt::Debug;
use alloc::sync::Arc;
use alloc::alloc::{
    alloc,
    dealloc,
    Layout,
};
use cortex_m_semihosting::hprintln;


/// 内核栈结构体
/// 注意：内核栈的地址是向下增长的，即栈底在高地址（更大），栈顶在低地址（更小）
pub struct KernelStack {
    /// 内核栈的底部地址
    bottom: usize,
    /// 内核栈的大小，单位是字节
    size: usize,
    /// 内核栈的栈指针
    stack_pointer: usize,
}

impl KernelStack {
    /// 创建一个新的内核栈
    pub fn new(size: usize) -> Self {
        //! fixme:功能不稳定：若size = 200 时会停在Alloc。
        //! size = 1024可正常工作。
        // hprintln!("KernelStack::new: enter");
        let bottom = unsafe {
            alloc(Layout::from_size_align(size, size).unwrap()) as usize
        };
        // hprintln!("KernelStack::new: bottom: {}", bottom);
        KernelStack { bottom, size , stack_pointer: 0 }
    }

    /// 创建一个空的内核栈
    pub fn new_empty() -> Self {
        KernelStack { bottom: 0, size: 0 , stack_pointer: 0 }
    }

    /// 获取内核栈的大小
    pub fn size(&self) -> usize {
        self.size
    }

    /// 获取内核栈的底部地址
    pub fn bottom(&self) -> usize {
        self.bottom
    }

    /// 获取内核栈的顶部地址
    pub fn top(&self) -> usize {
        self.bottom + self.size
    }

    /// 获取内核栈的栈指针的原始地址
    pub fn stack_pointer(&self) -> usize {
        &self.stack_pointer as *const usize as usize
    }

}

/// 内核栈的析构函数
impl Drop for KernelStack {
    fn drop(&mut self) {
        if self.bottom != 0 {
            unsafe {
                dealloc(
                    self.bottom as _,
                    Layout::from_size_align(self.size, self.size).unwrap(),
                );
            }
        }
    }
}
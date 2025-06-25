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


pub struct KernelStack {
    bottom: usize,
    size: usize,
}

impl KernelStack {
    pub fn new(size: usize) -> Self {
        // ! fixme:功能不稳定：若size = 200 时会停在Alloc。
        // ! size = 1024可正常工作。
        // hprintln!("KernelStack::new: enter");
        let bottom = unsafe {
            alloc(Layout::from_size_align(size, size).unwrap()) as usize
        };
        // hprintln!("KernelStack::new: bottom: {}", bottom);
        KernelStack { bottom, size }
    }

    pub fn new_empty() -> Self {
        KernelStack { bottom: 0, size: 0 }
    }


    pub fn size(&self) -> usize {
        self.size
    }

    pub fn bottom(&self) -> usize {
        self.bottom
    }

    pub fn top(&self) -> usize {
        self.bottom + self.size
    }

    pub fn init(&self,entry: usize,parameter: usize,texit: usize) {
        unsafe {
            
        }
    }
}

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
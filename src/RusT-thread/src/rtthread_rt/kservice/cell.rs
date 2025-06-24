#![warn(unused_imports)]        // 未使用的导入
/// 内核基础服务


// #![warn(warnings)]
/// 自旋锁
/// 
/// 用于保护共享资源 
/// 
/// 使用示例
/// 
/// static DATA: SpinLock<u32> = SpinLock::new(0);
/// 
/// 安全地访问和修改
/// let mut data = DATA.lock();
/// *data += 1;
/// DATA.unlock();

use core::cell::{RefCell, RefMut};
use core::ops::{Deref, DerefMut};
use crate::rtthread_rt::hardware::{rt_hw_interrupt_disable, rt_hw_interrupt_enable};


/// 中断安全的FreeCell
/// 
/// 用于保护共享资源
/// 
/// 使用示例
/// 
/// static DATA: RTIntrFreeCell<u32> = RTIntrFreeCell::new(0);
/// 
/// 安全地访问和修改
/// let mut data = DATA.exclusive_access();
/// *data += 1;
/// 
pub struct RTIntrFreeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for RTIntrFreeCell<T> {}

pub struct RTIntrRefMut<'a, T> {
    inner: Option<RefMut<'a, T>>,
    level: u32,
}

impl<T> RTIntrFreeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RTIntrRefMut<'_, T> {
        let level = rt_hw_interrupt_disable();
        // let level = 0;
        RTIntrRefMut {
            inner: Some(self.inner.borrow_mut()),
            level,
        }
    }

    pub fn exclusive_session<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&mut T) -> V,
    {
        let mut inner = self.exclusive_access();
        f(inner.deref_mut())
    }
}

impl<'a, T> Drop for RTIntrRefMut<'a, T> {
    fn drop(&mut self) {
        self.inner = None;
        rt_hw_interrupt_enable(self.level);
    }
}

impl<'a, T> Deref for RTIntrRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap().deref()
    }
}

impl<'a, T> DerefMut for RTIntrRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap().deref_mut()
    }
}






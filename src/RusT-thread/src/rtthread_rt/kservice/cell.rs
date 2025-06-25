// ! 内核服务相关函数
// ! 
// ! 定义了中断安全的Cell

#![warn(unused_imports)]


use core::cell::{RefCell, RefMut};
use core::ops::{Deref, DerefMut};
use crate::rtthread_rt::hardware::{rt_hw_interrupt_disable, rt_hw_interrupt_enable};


/// 中断安全的Cell
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


/// 与RTIntrFreeCell配套的RefMut
/// 可以更方便地访问和修改共享资源
pub struct RTIntrRefMut<'a, T> {
    inner: Option<RefMut<'a, T>>,
    level: u32,
}

impl<T> RTIntrFreeCell<T> {
    /// 创建一个中断安全的Cell
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// 获取一个中断安全的RefMut
    /// 
    /// 使用示例:
    /// 
    /// let mut data = DATA.exclusive_access();
    /// 
    pub fn exclusive_access(&self) -> RTIntrRefMut<'_, T> {
        let level = rt_hw_interrupt_disable();
        // let level = 0;
        RTIntrRefMut {
            inner: Some(self.inner.borrow_mut()),
            level,
        }
    }

    /// 在独占访问期间执行一个闭包
    /// 
    /// 使用示例:
    /// ```rust
    /// DATA.exclusive_session(|data| {
    ///     *data += 1; 
    /// });
    /// ```
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






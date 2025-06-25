//! 内核服务相关函数
//! 
//! 定义了中断安全的Cell

#![warn(unused_imports)]


use core::cell::{RefCell, RefMut};
use core::ops::{Deref, DerefMut};
use core::ptr;
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

    /// 获取内部数据的原始指针
    /// 
    /// 注意：返回的指针在RTIntrFreeCell的生命周期内有效
    /// 使用此方法需要确保在访问指针时不会发生数据竞争
    /// 
    /// 使用示例:
    /// ```rust
    /// let ptr = DATA.as_ptr();
    /// unsafe {
    ///     // 使用ptr进行安全的内存操作
    /// }
    /// ```
    pub fn as_ptr(&self) -> *const T {
        // 获取RefCell内部数据的地址
        // 这里我们通过借用RefCell来获取内部数据的地址
        // 注意：这不会触发中断禁用，因为只是获取地址
        self.inner.as_ptr()
    }

    /// 获取内部数据的可变原始指针
    /// 
    /// 注意：返回的指针在RTIntrFreeCell的生命周期内有效
    /// 使用此方法需要确保在访问指针时不会发生数据竞争
    /// 建议在使用可变指针时先调用exclusive_access()来禁用中断
    /// 
    /// 使用示例:
    /// ```rust
    /// let mut_ptr = DATA.as_mut_ptr();
    /// let _guard = DATA.exclusive_access(); // 禁用中断
    /// unsafe {
    ///     // 使用mut_ptr进行安全的内存操作
    /// }
    /// ```
    pub fn as_mut_ptr(&self) -> *mut T {
        // 获取RefCell内部数据的可变地址
        // 注意：这不会触发中断禁用，因为只是获取地址
        self.inner.as_ptr() as *mut T
    }

    /// 获取指定字段的原始指针
    /// 
    /// 使用示例:
    /// ```rust
    /// struct MyStruct {
    ///     value: u32,
    ///     flag: bool,
    /// }
    /// 
    /// static DATA: RTIntrFreeCell<MyStruct> = RTIntrFreeCell::new(MyStruct { value: 0, flag: false });
    /// 
    /// // 获取value字段的地址
    /// let value_ptr = DATA.field_ptr(|s| &s.value);
    /// ```
    pub fn field_ptr<F, U>(&self, field_accessor: F) -> *const U
    where
        F: FnOnce(&T) -> &U,
    {
        let data_ptr = self.as_ptr();
        unsafe {
            // 通过临时借用获取字段地址
            let temp_ref = &*data_ptr;
            let field_ref = field_accessor(temp_ref);
            ptr::addr_of!(*field_ref)
        }
    }

    /// 获取指定字段的可变原始指针
    /// 
    /// 注意：使用此方法时需要确保不会发生数据竞争
    /// 建议在使用前调用exclusive_access()来禁用中断
    /// 
    /// 使用示例:
    /// ```rust
    /// struct MyStruct {
    ///     value: u32,
    ///     flag: bool,
    /// }
    /// 
    /// static DATA: RTIntrFreeCell<MyStruct> = RTIntrFreeCell::new(MyStruct { value: 0, flag: false });
    /// 
    /// // 获取value字段的可变地址
    /// let value_mut_ptr = DATA.field_mut_ptr(|s| &mut s.value);
    /// ```
    pub fn field_mut_ptr<F, U>(&self, field_accessor: F) -> *mut U
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        let data_mut_ptr = self.as_mut_ptr();
        unsafe {
            // 通过临时可变借用获取字段地址
            let temp_mut_ref = &mut *data_mut_ptr;
            let field_mut_ref = field_accessor(temp_mut_ref);
            ptr::addr_of_mut!(*field_mut_ref)
        }
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






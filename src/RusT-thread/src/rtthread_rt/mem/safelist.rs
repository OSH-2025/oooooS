use core::cell::UnsafeCell;
use crate::rtthread_rt::mem::object::*;
extern crate alloc;
use alloc::vec::Vec;

/// 安全的对象列表实现，不使用裸指针在线程间共享
#[derive(Default)]
pub struct SafeRTList {
    // 存储对象ID
    objects: UnsafeCell<Vec<usize>>,
}
impl SafeRTList {
    /// 创建一个新的安全链表
    pub const fn new() -> Self {
        Self {
            objects: UnsafeCell::new(Vec::new()),
        }
    }

    /// 添加对象到链表，返回对象ID
    pub fn add(&self, object: *mut RTObject) -> usize {
        // 注册对象并获取ID
        let id = OBJECT_REGISTRY.register(object);
        
        // 将ID添加到对象列表
        unsafe {
            let objects = &mut *self.objects.get();
            objects.push(id);
        }
        
        id
    }

    /// 从链表中移除对象
    pub fn remove(&self, object: *mut RTObject) {
        // 查找对象ID
        if let Some(id) = OBJECT_REGISTRY.find_id(object) {
            // 从映射表中移除
            OBJECT_REGISTRY.unregister(id);
            
            // 从对象列表中移除
            unsafe {
                let objects = &mut *self.objects.get();
                if let Some(index) = objects.iter().position(|&x| x == id) {
                    objects.remove(index);
                }
            }
        }
    }

    /// 检查对象是否在链表中
    pub fn contains(&self, object: *mut RTObject) -> bool {
        if let Some(id) = OBJECT_REGISTRY.find_id(object) {
            unsafe {
                let objects = &*self.objects.get();
                objects.contains(&id)
            }
        } else {
            false
        }
    }

    /// 获取链表中的对象数量
    pub fn len(&self) -> usize {
        unsafe {
            let objects = &*self.objects.get();
            objects.len()
        }
    }

    /// 遍历链表中的所有对象
    pub fn for_each<F>(&self, mut f: F) where F: FnMut(*mut RTObject) {
        unsafe {
            let objects = &*self.objects.get();
            for &id in objects.iter() {
                let ptr = OBJECT_REGISTRY.get_object(id);
                if !ptr.is_null() {
                    f(ptr);
                }
            }
        }
    }
}

/// 允许在多线程环境中使用
unsafe impl Sync for SafeRTList {} 

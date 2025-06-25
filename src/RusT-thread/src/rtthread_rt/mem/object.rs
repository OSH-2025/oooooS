use crate::rtthread_rt::rtconfig::RT_NAME_MAX;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::ptr;
use crate::rtthread_rt::mem::safelist::SafeRTList;
extern crate alloc;
use alloc::vec::Vec;

/// Object structure
#[repr(C)]
pub struct RTObject {
    /// 用来储存对象的名称的数组
    pub name: [u8; RT_NAME_MAX as usize],
    /// 对象的类型
    pub obj_type: u8,
    /// 对象的标志状态
    pub flag: u8,
}


// 简单的ID生成器和对象存储系统
static NEXT_OBJECT_ID: AtomicUsize = AtomicUsize::new(1); // 每次生成一个新的对象，这个ID就会加1



// 对象映射表（因为这是单线程环境中的全局变量，不需要线程安全）
pub struct ObjectRegistry {
    // 存储对象和ID的映射关系
    id_to_ptr: UnsafeCell<Vec<*mut RTObject>>,
}

unsafe impl Sync for ObjectRegistry {}

impl ObjectRegistry {
    pub const fn new() -> Self {
        Self {
            id_to_ptr: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn register(&self, object: *mut RTObject) -> usize {
        let id = NEXT_OBJECT_ID.fetch_add(1, Ordering::SeqCst);
        
        unsafe {
            let vec = &mut *self.id_to_ptr.get();
            
            // 确保向量有足够空间
            if vec.len() <= id {
                vec.resize(id + 1, ptr::null_mut());
            }
            
            // 存储对象指针
            vec[id] = object;
        }
        
        id
    }
    
    pub fn unregister(&self, id: usize) {
        unsafe {
            let vec = &mut *self.id_to_ptr.get();
            if id < vec.len() {
                vec[id] = ptr::null_mut();
            }
        }
    }
    
    pub fn get_object(&self, id: usize) -> *mut RTObject {
        unsafe {
            let vec = &*self.id_to_ptr.get();
            if id < vec.len() {
                vec[id]
            } else {
                ptr::null_mut()
            }
        }
    }
    
    pub fn find_id(&self, object: *mut RTObject) -> Option<usize> {
        unsafe {
            let vec = &*self.id_to_ptr.get();
            for (id, &ptr) in vec.iter().enumerate() {
                if ptr == object {
                    return Some(id);
                }
            }
            None
        }
    }
}

// 全局对象注册表
pub static OBJECT_REGISTRY: ObjectRegistry = ObjectRegistry::new();


// 全局对象链表（静态常量初始化）
static OBJECT_LIST: SafeRTList = SafeRTList::new();


/// Initialize an object - 公开此函数
pub fn rt_object_init(object: *mut RTObject, obj_type: u8, name: &str) {
    unsafe {
        // Copy name with limited length
        ptr::write_bytes(&mut (*object).name, 0, (*object).name.len());
        let name_bytes = name.as_bytes();
        let copy_len = core::cmp::min(name_bytes.len(), (*object).name.len());
        ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            (*object).name.as_mut_ptr(),
            copy_len
        );
        
        // Set type and flags
        (*object).obj_type = obj_type;
        (*object).flag = 0;
        
        // 添加到安全链表
        OBJECT_LIST.add(object);
    }
}

/// Detach an object - 公开此函数
pub fn rt_object_detach(object: *mut RTObject) {
    // 从安全链表中移除
    OBJECT_LIST.remove(object);
}

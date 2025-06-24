use crate::rtthread_rt::rtconfig::RT_NAME_MAX;

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

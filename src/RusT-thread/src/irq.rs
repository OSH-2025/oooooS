pub use super::rthw::interrupt_disable;
use super::rtdef::{self, rt_uint8_t};
// use super::rtthread;

// 保持原有的宏定义
// #[cfg(feature = "hook")]
// pub const __on_rt_interrupt_enter_hook: fn() = rt_interrupt_enter_hook;
// #[cfg(feature = "hook")]
// pub const __on_rt_interrupt_leave_hook: fn() = rt_interrupt_leave_hook;

// 中断嵌套计数器
static mut RT_INTERRUPT_NEST: rt_uint8_t = 0;

// 钩子函数指针
#[cfg(feature = "hook")]
static mut RT_INTERRUPT_ENTER_HOOK: Option<fn()> = None;
#[cfg(feature = "hook")]
static mut RT_INTERRUPT_LEAVE_HOOK: Option<fn()> = None;

/// 设置中断进入钩子函数
#[cfg(feature = "hook")]
pub fn rt_interrupt_enter_sethook(hook: fn()) {
    unsafe {
        RT_INTERRUPT_ENTER_HOOK = Some(hook);
    }
}

/// 设置中断退出钩子函数
#[cfg(feature = "hook")]
pub fn rt_interrupt_leave_sethook(hook: fn()) {
    unsafe {
        RT_INTERRUPT_LEAVE_HOOK = Some(hook);
    }
}

/// 中断进入函数
pub fn rt_interrupt_enter() {
    {
        let _level = interrupt_disable();
        
        unsafe {
            RT_INTERRUPT_NEST += 1;
            #[cfg(feature = "hook")]
            if let Some(hook) = RT_INTERRUPT_ENTER_HOOK {
                hook();
            }
        }
    }
    
    #[cfg(feature = "debug")]
    {
        rtthread::rt_kprintf("irq has come..., irq current nest:%d\n", 
            unsafe { RT_INTERRUPT_NEST as i32 });
    }
}

/// 中断退出函数
pub fn rt_interrupt_leave() {
    #[cfg(feature = "debug")]
    {
        rtthread::rt_kprintf("irq is going to leave, irq current nest:%d\n", 
            unsafe { RT_INTERRUPT_NEST as i32 });
    }
    {
        let _level = interrupt_disable();
        
        unsafe {
            #[cfg(feature = "hook")]
            if let Some(hook) = RT_INTERRUPT_LEAVE_HOOK {
                hook();
            }
            RT_INTERRUPT_NEST -= 1;
        }

    }
}

/// 获取中断嵌套层数
pub fn rt_interrupt_get_nest() -> u8 {
    {
        let _level = interrupt_disable();
        let ret = unsafe { RT_INTERRUPT_NEST };
        ret
    }
}

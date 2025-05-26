use core::arch::asm;
use cortex_m_semihosting::hprintln;
use lazy_static::lazy_static;
use crate::kservice::RTIntrFreeCell;
use core::sync::atomic::{AtomicU8, Ordering};

// 保持原有的宏定义
// #[cfg(feature = "hook")]
// pub const __on_rt_interrupt_enter_hook: fn() = rt_interrupt_enter_hook;
// #[cfg(feature = "hook")]
// pub const __on_rt_interrupt_leave_hook: fn() = rt_interrupt_leave_hook;

// 中断嵌套计数器
lazy_static! {
    // 中断嵌套计数器
    static ref RT_INTERRUPT_NEST: AtomicU8 = AtomicU8::new(0);
    // 中断进入钩子函数
    static ref RT_INTERRUPT_ENTER_HOOK: RTIntrFreeCell<Option<fn()>> = unsafe { RTIntrFreeCell::new(None) };
    // 中断退出钩子函数
    static ref RT_INTERRUPT_LEAVE_HOOK: RTIntrFreeCell<Option<fn()>> = unsafe { RTIntrFreeCell::new(None) };
}

// 禁用中断
pub fn rt_hw_interrupt_disable() -> u32 {
    let level: u32;
    unsafe {
        asm!(
            "MRS {0}, PRIMASK",
            "CPSID I",
            out(reg) level,
            options(nostack, preserves_flags)
        );
    }
    level
}

// 启用中断
pub fn rt_hw_interrupt_enable(level: u32) {
    unsafe {
        asm!(
            "MSR PRIMASK, {0}",
            in(reg) level,
            options(nostack, preserves_flags)
        );
    }
}


/// 设置中断进入钩子函数
#[cfg(feature = "hook")]
pub fn rt_interrupt_enter_sethook(hook: fn()) {
    RT_INTERRUPT_ENTER_HOOK.exclusive_access() = Some(hook);
}

/// 设置中断退出钩子函数
#[cfg(feature = "hook")]
pub fn rt_interrupt_leave_sethook(hook: fn()) {
    RT_INTERRUPT_LEAVE_HOOK.exclusive_access() = Some(hook);
}

/// 中断进入函数
pub fn rt_interrupt_enter() {
    {
        let level = rt_hw_interrupt_disable();
        
            RT_INTERRUPT_NEST.fetch_add(1, Ordering::Relaxed);
            #[cfg(feature = "hook")]
            if let Some(hook) = RT_INTERRUPT_ENTER_HOOK.exclusive_access() {
                hook();
            }
        rt_hw_interrupt_enable(level);
    }
    
    #[cfg(feature = "debug")]
    {
        hprintln!("enter interrupt, nest:{}", 
            RT_INTERRUPT_NEST as i32);
    }
}

/// 中断退出函数
pub fn rt_interrupt_leave() {
    #[cfg(feature = "debug")]
    {
        hprintln!("leave interrupt, nest:{}", 
            RT_INTERRUPT_NEST as i32);
    }
    {
        let level = rt_hw_interrupt_disable();
        
        RT_INTERRUPT_NEST.fetch_sub(1, Ordering::Relaxed);
        #[cfg(feature = "hook")]
        if let Some(hook) = RT_INTERRUPT_LEAVE_HOOK.exclusive_access() {
            hook();
        }
        rt_hw_interrupt_enable(level);
    }
}

/// 获取中断嵌套层数
pub fn rt_interrupt_get_nest() -> u8 {
    {
        let level = rt_hw_interrupt_disable();
        let ret = RT_INTERRUPT_NEST.load(Ordering::Acquire);
        rt_hw_interrupt_enable(level);
        ret
    }
}

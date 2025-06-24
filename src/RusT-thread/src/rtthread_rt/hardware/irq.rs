//! 中断管理模块
//! 
//! 本文件提供了中断嵌套计数、钩子设置、使能/禁用中断等功能。
//! 主要函数：
//! - rt_hw_interrupt_disable/enable：保存/恢复中断状态，常用于临界区保护。
//! - rt_interrupt_enter/leave：中断进入/退出时调用，维护嵌套计数。
//! - rt_interrupt_get_nest：获取当前中断嵌套层数。
//! - rt_interrupt_enter_sethook/leave_sethook：设置中断进入/退出钩子（需启用hook特性）。

use core::arch::asm;
use cortex_m_semihosting::hprintln;
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicU8, Ordering};

use crate::rtthread_rt::kservice::RTIntrFreeCell;

// 中断嵌套计数器
lazy_static! {
    // 中断嵌套计数器
    static ref RT_INTERRUPT_NEST: AtomicU8 = AtomicU8::new(0);
    // 中断进入钩子函数
    static ref RT_INTERRUPT_ENTER_HOOK: RTIntrFreeCell<Option<fn()>> = unsafe { RTIntrFreeCell::new(None) };
    // 中断退出钩子函数
    static ref RT_INTERRUPT_LEAVE_HOOK: RTIntrFreeCell<Option<fn()>> = unsafe { RTIntrFreeCell::new(None) };
}

/// 禁用中断，并返回原中断状态。
/// 通常用于进入临界区，使用后应配合 rt_hw_interrupt_enable 恢复。
/// 返回值：原始 PRIMASK 状态。
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

/// 恢复中断状态。
/// 参数 level 应为 rt_hw_interrupt_disable 返回值。
/// 通常用于离开临界区。
pub fn rt_hw_interrupt_enable(level: u32) {
    unsafe {
        asm!(
            "MSR PRIMASK, {0}",
            in(reg) level,
            options(nostack, preserves_flags)
        );
    }
}

/// 设置中断进入钩子函数。
/// 仅在启用 feature = "hook" 时可用。
/// 调用后，每次进入中断时会自动调用 hook。
#[cfg(feature = "hook")]
pub fn rt_interrupt_enter_sethook(hook: fn()) {
    RT_INTERRUPT_ENTER_HOOK.exclusive_access() = Some(hook);
}

/// 设置中断退出钩子函数。
/// 仅在启用 feature = "hook" 时可用。
/// 调用后，每次退出中断时会自动调用 hook。
#[cfg(feature = "hook")]
pub fn rt_interrupt_leave_sethook(hook: fn()) {
    RT_INTERRUPT_LEAVE_HOOK.exclusive_access() = Some(hook);
}

/// 中断进入时调用。
/// 维护中断嵌套计数，必要时调用进入钩子。
/// 通常在中断服务函数入口处调用。
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
            RT_INTERRUPT_NEST.load(Ordering::Acquire) as i32);
    }
}

/// 中断退出时调用。
/// 维护中断嵌套计数，必要时调用退出钩子。
/// 通常在中断服务函数出口处调用。
pub fn rt_interrupt_leave() {
    #[cfg(feature = "debug")]
    {
        hprintln!("leave interrupt, nest:{}", 
            RT_INTERRUPT_NEST.load(Ordering::Acquire) as i32);
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

/// 获取当前中断嵌套层数。
/// 可用于判断当前是否处于中断上下文。
/// 返回值：嵌套层数，0 表示无中断。
pub fn rt_interrupt_get_nest() -> u8 {
    {
        let level = rt_hw_interrupt_disable();
        let ret = RT_INTERRUPT_NEST.load(Ordering::Acquire);
        rt_hw_interrupt_enable(level);
        ret
    }
}




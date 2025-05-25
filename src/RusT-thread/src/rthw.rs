use super::rtdef;

#[unsafe(link_section = ".text")]
unsafe extern "C" {
    pub fn rt_hw_interrupt_disable() -> rtdef::rt_base_t;
    pub fn rt_hw_interrupt_enable(level: rtdef::rt_base_t);
}

    // 封装为安全的Rust接口
pub struct InterruptGuard {
    level: rtdef::rt_base_t,
}

impl InterruptGuard {
    fn interrupt_disable() -> Self {
        let level = unsafe { rt_hw_interrupt_disable() };
        Self { level }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        unsafe { rt_hw_interrupt_enable(self.level) };
    }
}

// 提供便捷的全局函数
pub fn interrupt_disable() -> InterruptGuard {
    InterruptGuard::interrupt_disable()
}


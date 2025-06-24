use cortex_m_rt::{exception};
use cortex_m_semihosting::hprintln;


/// 异常处理
#[exception]
fn MemoryManagement() -> ! {
    hprintln!("MemoryManagement");
    loop {}
}

#[exception]
fn BusFault() -> ! {
    hprintln!("BusFault");
    loop {}
}

#[exception]
fn UsageFault() -> ! {
    hprintln!("UsageFault");
    loop {}
}

#[exception] 
unsafe fn DefaultHandler(irqn: i16) {
    hprintln!("DefaultHandler: irqn = {}", irqn);
    loop {}
}
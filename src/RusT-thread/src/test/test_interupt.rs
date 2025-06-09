use crate::irq;
use cortex_m_semihosting::hprintln;
use crate::kservice;

pub fn test_interupt(){
    let level =irq::rt_hw_interrupt_disable();
    hprintln!("level: {:?}", level);    
    hprintln!("interrupt_nest: {:?}", irq::rt_interrupt_get_nest());
    // do something
    interrupt_inner();
    hprintln!("interrupt_nest: {:?}", irq::rt_interrupt_get_nest());
    irq::rt_hw_interrupt_enable(level);
}

fn interrupt_inner(){
    let level =irq::rt_hw_interrupt_disable();
    hprintln!("interrupt_inner level: {:?}", level);
    hprintln!("interrupt_nest: {:?}", irq::rt_interrupt_get_nest());
    irq::rt_hw_interrupt_enable(level);
}

pub fn test_RtIntrFreeCell(){
    let mut data = unsafe { kservice::RTIntrFreeCell::<u128>::new(0) };

    let mut data = data.exclusive_access();
    *data += 1;

    hprintln!("data: {:?}", *data);
}


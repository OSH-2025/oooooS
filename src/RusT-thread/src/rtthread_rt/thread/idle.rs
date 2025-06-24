use cortex_m::asm;
use cortex_m_semihosting::hprintln;

use crate::rtthread_rt::thread::thread::*;
use crate::rtthread_rt::rtconfig;


pub extern "C" fn idle_entry(arg: usize) -> () {
    hprintln!("idle_entry...");
    loop{
        asm::nop;
    }
    hprintln!("idle_entry finished.");
}

pub fn init_idle(){
    hprintln!("Initializing idle...");
    let idle = rt_thread_create("idle", idle_entry as usize, 1024, rtconfig::RT_THREAD_PRIORITY_MAX as u8, 1000);
    hprintln!("idle created.");
    rt_thread_startup(idle);
    hprintln!("idle started.");
    hprintln!("Idle initialized.");
}
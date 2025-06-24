use cortex_m::asm;
use cortex_m_semihosting::hprintln;

use crate::rtthread_rt::thread::thread::*;
use crate::rtthread_rt::rtconfig;
use crate::rtthread_rt::thread::scheduler;


pub extern "C" fn idle_entry(arg: usize) -> () {
    hprintln!("idle_entry...");
    loop{
        asm::nop;
    }
    hprintln!("idle_entry finished.");
}

pub fn init_idle(){
    // hprintln!("Initializing idle...");
    let idle = rt_thread_create("idle", idle_entry as usize, 1024, (rtconfig::RT_THREAD_PRIORITY_MAX - 1) as u8, 1000);
    // 以下实现有误！，我们要将idle线程插入到就绪队列中，然后调用rt_thread_startup
    // rt_thread_startup(idle);
    scheduler::insert_thread(idle.clone());
    // hprintln!("Idle initialized.");
}
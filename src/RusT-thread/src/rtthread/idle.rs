use cortex_m::asm;
use crate::rtthread;
use crate::rtconfig;


pub extern "C" fn idle_entry(arg: usize) -> () {
    loop{
        asm::nop;
    }
}

pub fn init_idle(){
    let idle = rtthread::thread::rt_thread_create("idle", idle_entry as usize, 1024, rtconfig::RT_THREAD_PRIORITY_MAX as u8, 1000);
    rtthread::thread::rt_thread_startup(idle);
}
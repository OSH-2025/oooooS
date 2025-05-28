use lazy_static::lazy_static;
use crate::kservice::RTIntrFreeCell;
use crate::rtthread::thread::RtThread;
use crate::rtconfig::RT_TICK_PER_SECOND;

// 线程让出标志
pub const RT_THREAD_STAT_YIELD: u8 = 0x08;
pub const RT_WAITING_FOREVER: u32 = 0xFFFFFFFF;

// 定义全局变量：当前时钟，使用RTIntrFreeCell包裹，实现中断安全
lazy_static! {
    static ref RT_TICK: RTIntrFreeCell<u32> = unsafe { RTIntrFreeCell::new(0) };
}

// 获取当前时钟周期
pub fn rt_tick_get() -> u32 {
    *RT_TICK.exclusive_access()
}

// 设置当前时钟周期
pub fn rt_tick_set(tick: u32) {
    *RT_TICK.exclusive_access() = tick;
}

//时钟中断处理函数
pub fn rt_tick_increase() {

    *RT_TICK.exclusive_access() +=1 ;

    let thread = RtThread::current();//TODO: 获取当前线程
    thread.inner.exclusive_access().remaining_tick -= 1;

    if thread.inner.exclusive_access().remaining_tick == 0 {
        thread.inner.exclusive_access().remaining_tick = thread.inner.exclusive_access().init_tick;
        thread.inner.exclusive_access().stat |= RT_THREAD_STAT_YIELD;
        RtThread::schedule();//TODO: 重新调度
    }

    // 检查定时器
    //TODO: 检查定时器
}

// 将毫秒转换为时钟周期
pub fn rt_tick_from_millisecond(ms: i32) -> u32 {
    if ms < 0 {
        RT_WAITING_FOREVER
    } else {
        let tick = RT_TICK_PER_SECOND * (ms as u32 / 1000);
        let tick = tick + (RT_TICK_PER_SECOND * (ms as u32 % 1000) + 999) / 1000;
        tick
    }
}

// 获取自启动以来经过的毫秒数
pub fn rt_tick_get_millisecond() -> u32 {
    if 1000 % RT_TICK_PER_SECOND == 0 {
        rt_tick_get() * (1000 / RT_TICK_PER_SECOND)
    } else {
        // 错误情况，直接返回0
        0
    }
}







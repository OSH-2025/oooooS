use crate::rtdef::*;
use crate::rtthread::thread::RtThread;
use crate::kservice::RTIntrFreeCell;
extern crate alloc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;

const IDLE_HOOK_LIST_SIZE: usize = 4;
const IDLE_THREAD_STACK_SIZE: usize = 256;

lazy_static! {
    static ref IDLE_HOOK_LIST: RTIntrFreeCell<Vec<Box<dyn Fn() + Send + Sync>>> = 
        unsafe { RTIntrFreeCell::new(Vec::with_capacity(IDLE_HOOK_LIST_SIZE)) };
    static ref DEFUNCT_THREADS: RTIntrFreeCell<Vec<Box<RtThread>>> = 
        unsafe { RTIntrFreeCell::new(Vec::new()) };
}

pub struct IdleThread {
    thread: Box<RtThread>,
    stack: [u8; IDLE_THREAD_STACK_SIZE],
}

impl IdleThread {
    pub fn new() -> Self {
        let stack = [0u8; IDLE_THREAD_STACK_SIZE];
        let thread = Box::new(RtThread {
            name: [0; 8],
            object_type: RT_Object_Class_Thread as u8,
            inner: unsafe { RTIntrFreeCell::new(RtThreadInner {
                error: 0,
                stat: ThreadState::RT_THREAD_READY,
                current_priority: RT_THREAD_PRIORITY_MAX - 1,
                number_mask: 0,
                entry: Box::new(|| {}),
                init_tick: 0,
                remaining_tick: 0,
                context: RtContext {
                    ra: 0,
                    sp: 0,
                    s: [0; 12],
                },
                user_data: 0,
            })},
            cleanup: |_| {},
        });
        
        IdleThread { thread, stack }
    }
}

pub fn idle_thread_init() {
    let idle_thread = IdleThread::new();
    // Initialize idle thread and start it
    // TODO: Implement thread startup logic
}

pub fn idle_thread_set_hook<F>(hook: F) -> RtError 
where
    F: Fn() + Send + Sync + 'static
{
    let mut hooks = IDLE_HOOK_LIST.lock();
    if hooks.len() >= IDLE_HOOK_LIST_SIZE {
        return RtError::RT_EFULL;
    }
    hooks.push(Box::new(hook));
    RtError::RT_EOK
}

pub fn idle_thread_del_hook<F>(hook: &F) -> RtError 
where
    F: Fn() + Send + Sync + 'static
{
    let mut hooks = IDLE_HOOK_LIST.lock();
    if let Some(pos) = hooks.iter().position(|h| std::ptr::eq(h.as_ref(), hook)) {
        hooks.remove(pos);
        RtError::RT_EOK
    } else {
        RtError::RT_ENOSYS
    }
}

pub fn defunct_thread_enqueue(thread: Box<RtThread>) {
    let mut defunct = DEFUNCT_THREADS.lock();
    defunct.push(thread);
}

fn defunct_thread_dequeue() -> Option<Box<RtThread>> {
    let mut defunct = DEFUNCT_THREADS.lock();
    defunct.pop()
}

fn defunct_thread_execute() {
    while let Some(thread) = defunct_thread_dequeue() {
        // Execute cleanup if exists
        if let Some(cleanup) = thread.cleanup {
            cleanup(Box::into_raw(thread));
        }
    }
}

pub fn idle_thread_entry() {
    loop {
        // Execute idle hooks
        let hooks = IDLE_HOOK_LIST.lock();
        for hook in hooks.iter() {
            hook();
        }
        drop(hooks);

        // Execute defunct thread cleanup
        defunct_thread_execute();

        // TODO: Implement power management if needed
    }
}


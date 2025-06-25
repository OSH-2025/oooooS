//! 定时器模块
//! 
//! 本模块实现了RT-Thread的定时器功能
//! 包括定时器的创建、启动、停止、控制等

#![warn(unused_imports)]

pub mod timer;
pub mod clock;

pub use self::timer::*;
pub use self::clock::*;


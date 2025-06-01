# Rust-thread

## 项目结构

该项目包含以下几个模块：

- `src/mem`：内存管理模块，包含内存分配、释放、重分配等功能。
- `src/rtthread`：RT-Thread内核模块，包含线程管理、调度等功能。
- `src/rtconfig`：RT-Thread配置模块，包含RT-Thread的配置信息。
- `src/context`：上下文切换模块，包含上下文切换的功能。
- `src/irq`：中断管理模块，包含中断管理的功能。
- `src/clock`&`src/timer`：时钟管理模块，包含时钟管理的功能。
- `src/ipc`：进程间通信模块，包含进程间通信的功能。

## 项目实现

## 运行

执行 `cargo build` 构建项目。

执行 `qemu-system-arm   -cpu cortex-m4   -machine netduinoplus2   -nographic   -semihosting-config enable=on,target=native   -kernel target/thumbv7em-none-eabihf/debug/RusT-thread` 在QEMU上运行项目。

（注：需要安装QEMU，并安装arm-none-eabi-gcc）

或者执行 `cargo run` 在QEMU上运行项目。

在`.cargo/config.toml`中可以调整试用的架构，默认使用 `thumbv7em-none-eabihf` 架构（cortex-m4）。
作为验证，我们只编写了Cortex-M4的硬件支持，如果需要使用其他架构，需修改:

- `src/context.rs` (更换为其他架构的上下文切换代码)
- `src/cpuport.rs` (更换为其他架构的CPU接口)
- `memory.x` (更换为其他架构的地址空间)


## 测试

在编译时添加 `test` 特性，即可运行测试用例。
在test目录下，有测试用例。




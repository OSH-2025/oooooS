# Rust-thread

<pre>
    ____            ______  ________                        __
   / __ \__  ______/_  __/ /_  __/ /_  ________  ____ _____/ /
  / /_/ / / / / ___// /_____/ / / __ \/ ___/ _ \/ __ `/ __  / 
 / _, _/ /_/ (__  )/ /_____/ / / / / / /  /  __/ /_/ / /_/ /  
/_/ |_|\__,_/____//_/     /_/ /_/ /_/_/   \___/\__,_/\__,_/   
</pre>

## 项目结构

<pre>
oooooS/src/RusT-thread/src
├── main.rs                           // 程序入口点，初始化硬件、内存、定时器和线程系统
├── rtthread_rt                       // RT-Thread实时操作系统核心模块
│   ├── hardware                      // 硬件抽象层，提供与硬件相关的底层功能
│   │   ├── context.rs                // 线程上下文切换实现
│   │   ├── cpuport.rs                // CPU端口相关函数，栈初始化和CPU控制
│   │   ├── exception.rs              // 异常处理机制
│   │   ├── irq.rs                    // 中断请求处理
│   │   └── mod.rs                    // 硬件模块统一导出
│   ├── ipc.rs                        // 进程间通信机制
│   ├── kservice                      // 内核服务模块
│   │   ├── cell.rs                   // 内核服务单元实现
│   │   └── mod.rs                    // 内核服务模块统一导出
│   ├── mem                           // 内存管理模块
│   │   ├── allocator.rs              // 内存分配器实现
│   │   ├── mod.rs                    // 内存管理模块统一导出
│   │   ├── object.rs                 // 内核对象管理
│   │   ├── oom.rs                    // 内存不足处理
│   │   ├── safelist.rs               // 线程安全链表实现
│   │   ├── small_mem_allocator.rs    // 小内存分配器
│   │   └── small_mem_impl.rs         // 小内存分配器具体实现
│   ├── mod.rs                        // RT-Thread根模块，统一导出所有子模块
│   ├── rtconfig.rs                   // RT-Thread配置参数
│   ├── rtdef.rs                      // RT-Thread核心定义和常量
│   ├── thread                        // 线程管理模块
│   │   ├── idle.rs                   // 空闲线程实现
│   │   ├── kstack.rs                 // 内核栈管理
│   │   ├── mod.rs                    // 线程模块统一导出
│   │   ├── scheduler.rs              // 线程调度器实现
│   │   ├── thread.rs                 // 线程核心实现
│   │   └── thread_priority_table.rs  // 线程优先级表管理
│   └── timer                         // 定时器模块
│       ├── clock.rs                  // 系统时钟管理
│       ├── mod.rs                    // 定时器模块统一导出
│       └── timer.rs                  // 定时器核心实现
├── test                              // 测试模块
│   ├── mod.rs                        // 测试模块统一导出和测试运行入口
│   ├── test_excp.rs                  // 异常处理测试
│   ├── test_interupt.rs              // 中断处理测试
│   ├── test_mem.rs                   // 内存管理测试
│   ├── test_scheduler.rs             // 调度器测试
│   ├── test_small_mem.rs             // 小内存分配器测试
│   ├── test_thread.rs                // 线程功能测试
│   └── test_timer.rs                 // 定时器功能测试
└── user_main.rs                      // 用户主线程入口，用户应用程序逻辑实现
</pre>

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

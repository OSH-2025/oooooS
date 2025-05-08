# 立项依据——罗浩民

# 问题定义

**核心命题：以Rust语言重构RT-Thread Nano内核，构建兼具** **安全性、实时性和开发效率** **的嵌入式实时操作系统**。

### 重构范围与边界

**核心重构对象**：

- 任务调度器（Scheduler）
- 进程间通信（IPC）
- 时钟管理（Timer）
- 内存管理（Heap Allocator）

# 技术可行性

## **核心模块的Rust化改造**

• **Rust调用C代码**：

使用`bindgen`工具自动生成RT-Thread内核API的Rust绑定（FFI）通过`extern "C"`声明保留原有C实现的模块，作为过渡阶段的兼容层。

- **C调用Rust代码**：

对Rust重构的安全抽象层（如内存分配器、智能锁），通过`cbindgen`生成C头文件，确保原有C代码（如HAL驱动）可无缝调用。

## **开发-调试工具链**

- **编译环境**：
    
    基于PlatformIO构建多语言工程。
    
    [支持不同的语言和编译器 - Ideas - PlatformIO Community](https://community.platformio.org/t/support-for-different-languages-and-compilers/921)这篇文章中，Platform官方提出其可以自定义开发平台，而其底层是基于**SCons**，而SCons支持集成 Rust。
    
- **仿真验证**：
    
    **Wokwi**：配置`.wokwi.toml`模拟STM32硬件外设，实时可视化线程状态切换（如通过GPIO电平模拟调度器行为）。
    
- **上板验证：**
    
    PlatformIO可直接管理与真实设备的连接并方便上板调试运行以测试。
    

## **原项目**

- RTThread完全开源，且拥有丰富完整的文档以及相关生态。

# 关键点

- Rust+C编译环境的搭建
- RT-thread Nano的内核结构与具体实现
- Rust改写
- 改写成果的调试验证与优化

# 预期目标

- **内存安全**：通过所有权模型和借用检查，消除CWE-119（缓冲区溢出）、CWE-416（释放后使用）等漏洞
- **并发安全**：Rust的`Send`/`Sync` Trait静态验证数据竞争，结合`Mutex<RefCell<T>>`智能锁，降低调度器数据竞争发生率
- **性能优化：**尝试优化该系统的性能，如实时性等
- **精简代码：**利用Rust的优质特性精简代码
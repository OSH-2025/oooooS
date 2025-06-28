# Rust-Thread 库示例代码

本目录包含了展示 rust-thread 库各种功能的示例代码。

## 示例文件说明

### 1. `example.rs` - 基础示例
展示了 rust-thread 库的基本功能：
- 线程创建、启动、挂起、恢复、删除
- 线程优先级控制
- 线程睡眠
- 基本的调度演示

**主要函数：**
- `run_example()` - 运行基础示例
- `run_comprehensive_demo()` - 运行全面功能演示
- `run_mfq_demo()` - 运行多级反馈队列调度演示
- `run_all_demos()` - 运行所有演示

### 2. `comprehensive_example.rs` - 全面功能示例
展示了 rust-thread 库的所有主要功能：

#### 线程管理功能
- **基础线程演示** (`basic_thread_demo`) - 展示线程生命周期
- **优先级变化演示** (`priority_change_demo`) - 动态改变线程优先级
- **睡眠演示** (`sleep_demo`) - 线程睡眠和唤醒
- **让出CPU演示** (`yield_demo`) - 线程主动让出CPU
- **线程控制演示** (`thread_control_demo`) - 线程挂起和恢复
- **恢复辅助线程** (`resume_helper_thread`) - 恢复被挂起的线程

#### 定时器功能
- **定时器演示** (`timer_demo`) - 单次定时器和周期定时器
- 定时器回调函数
- 定时器启动、停止和控制

#### 调度策略
- **调度策略演示** (`scheduling_policy_demo`) - 显示当前调度策略
- **多级反馈队列演示** (`mfq_demo_thread`) - MFQ调度策略演示

#### 中断管理
- **中断级别演示** (`interrupt_level_demo`) - 中断禁用/启用演示

## 使用方法

### 运行基础示例
```rust
use crate::test::example;

// 运行基础示例
example::run_example();

// 运行全面功能演示
example::run_comprehensive_demo();

// 运行所有演示
example::run_all_demos();
```

### 运行全面示例
```rust
use crate::test::comprehensive_example;

// 运行全面功能演示
comprehensive_example::run_comprehensive_demo();

// 运行多级反馈队列调度演示
comprehensive_example::run_mfq_demo();
```

## 功能特性展示

### 1. 线程管理
- ✅ 线程创建和启动
- ✅ 线程挂起和恢复
- ✅ 线程删除
- ✅ 优先级设置和动态调整
- ✅ 线程睡眠
- ✅ 线程让出CPU

### 2. 定时器系统
- ✅ 单次定时器
- ✅ 周期定时器
- ✅ 定时器回调函数
- ✅ 定时器启动/停止
- ✅ 定时器控制

### 3. 调度策略
- ✅ 优先级调度
- ✅ 多级反馈队列调度
- ✅ 调度策略动态切换

### 4. 中断管理
- ✅ 中断禁用/启用
- ✅ 中断级别查询
- ✅ 中断安全的数据访问

## 注意事项

1. **中断安全**：所有示例都考虑了中断安全性，使用 `RTIntrFreeCell` 保护共享数据
2. **资源管理**：示例中正确管理线程和定时器资源，避免资源泄漏
3. **性能考虑**：示例代码考虑了性能影响，避免过度打印和计算

这些示例为 rust-thread 库提供了全面的功能展示，可以作为学习和开发的参考。 
```bash
# 基准测试需要在主机系统上运行，不是之前的嵌入式平台
rustup target add x86_64-unknown-linux-gnu

```

进入项目目录
```bash
cd src/RusT-thread/
```

### 内存管理基准测试 (`memory_bench.rs`)
- 内存分配器创建性能
- 单次和批量内存分配
- 内存分配/释放循环
- 内存碎片处理和整理算法
### 定时器管理基准测试 (`timer_bench.rs`)
- 定时器系统创建和初始化性能
- 单次定时器和周期定时器添加性能
- 定时器到期处理和调度算法效率
- 定时器删除和堆维护操作
- 定时器堆数据结构性能随规模变化

### 运行特定基准测试
```bash
# 运行内存管理基准测试
cargo bench --no-default-features --target x86_64-unknown-linux-gnu --bench memory_bench

### 运行定时器的性能测试
cargo bench --no-default-features --target x86_64-unknown-linux-gnu --bench timer_bench

```

### 运行特定的基准测试函数
```bash
### 运行内存模块的性能测试：
cargo bench --no-default-features --target x86_64-unknown-linux-gnu --bench memory_bench -- single_allocation

### 运行定时器的性能测试
cargo bench --no-default-features --target x86_64-unknown-linux-gnu --bench timer_bench -- timer_creation
cargo bench --no-default-features --target x86_64-unknown-linux-gnu --bench timer_bench -- heap_operations

```

### 终端输出
会有类似的输出结果：
```
allocator_creation/create_allocator/1024
                        time:   [40.502 ns 41.258 ns 42.805 ns]
                        change: [-7.77% -4.05% +2.24%] (p = 0.05 > 0.05)
```

解释：
- time: `[最小值 平均值 最大值]` 执行时间
- change: 与上次运行的性能变化百分比
- outliers: 异常值数量和类型

### HTML报告位置
Criterion会自动生成HTML报告：

```bash
# 查看 HTML 报告位置
ls target/x86_64-unknown-linux-gnu/release/deps/

# 报告保存在 target/criterion/ 目录中
find target/criterion -name "index.html" -type f
```
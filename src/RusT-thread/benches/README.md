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

### 运行特定基准测试
```bash
# 运行内存管理基准测试
cargo bench --no-default-features --target x86_64-unknown-linux-gnu --bench memory_bench

```

### 运行特定的基准测试函数
```bash
# 运行特定的基准测试函数
cargo bench --no-default-features --target x86_64-unknown-linux-gnu --bench memory_bench -- single_allocation

```

### 终端输出
基准测试会在终端显示实时结果：
```
allocator_creation/create_allocator/1024
                        time:   [40.502 ns 41.258 ns 42.805 ns]
                        change: [-7.77% -4.05% +2.24%] (p = 0.05 > 0.05)
```

结果解释：
- **time**: `[最小值 平均值 最大值]` 执行时间
- **change**: 与上次运行的性能变化百分比
- **outliers**: 异常值数量和类型

### HTML报告位置
Criterion会自动生成HTML报告：

```bash
# 查看 HTML 报告位置
ls target/x86_64-unknown-linux-gnu/release/deps/

# 报告保存在 target/criterion/ 目录中
find target/criterion -name "index.html" -type f
```
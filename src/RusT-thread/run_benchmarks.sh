#!/bin/bash

echo "=========================================="
echo "正在为Rust操作系统运行性能基准测试..."
echo "=========================================="

echo ""
echo "运行内存管理基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --bench memory_bench --target x86_64-unknown-linux-gnu

echo ""
echo "运行调度器基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --bench scheduler_bench --target x86_64-unknown-linux-gnu

echo ""
echo "运行上下文切换基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --bench context_switch_bench --target x86_64-unknown-linux-gnu

echo ""
echo "运行定时器基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --bench timer_bench --target x86_64-unknown-linux-gnu

echo ""
echo "基准测试完成！"
echo "HTML报告位置: target/x86_64-unknown-linux-gnu/release/deps/"
echo "详细结果已保存在 target/criterion/ 目录中"

# 检查是否有HTML报告生成
if [ -d "target/criterion" ]; then
    echo ""
    echo "📊 可用的基准测试报告:"
    find target/criterion -name "index.html" -type f 2>/dev/null | head -5
fi 
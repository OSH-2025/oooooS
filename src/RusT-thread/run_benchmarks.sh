#!/bin/bash

echo "=========================================="
echo "正在为Rust操作系统运行性能基准测试..."
echo "=========================================="

# 检测操作系统并设置正确的目标架构
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    TARGET="x86_64-unknown-linux-gnu"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    TARGET="x86_64-apple-darwin"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    TARGET="x86_64-pc-windows-msvc"
else
    echo "未识别的操作系统，使用默认 Linux 目标"
    TARGET="x86_64-unknown-linux-gnu"
fi

echo "使用目标架构: $TARGET"
echo ""

echo "运行内存管理基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --target $TARGET --bench memory_bench

echo ""
echo "运行调度器基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --target $TARGET --bench scheduler_bench

echo ""
echo "运行上下文切换基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --target $TARGET --bench context_switch_bench

echo ""
echo "运行定时器基准测试..."
echo "----------------------------------------"
cargo bench --no-default-features --target $TARGET --bench timer_bench

echo ""
echo "基准测试完成！"
echo "HTML报告位置: target/$TARGET/release/deps/"
echo "详细结果已保存在 target/criterion/ 目录中"
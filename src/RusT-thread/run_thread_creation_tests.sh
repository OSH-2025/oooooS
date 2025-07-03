#!/bin/bash

#* 使用方法：在脚本所在目录下执行 ./run_thread_creation_tests.sh 自定义测试次数
#* 确保原系统执行cargo run 命令时运行的是线程创建的测试

# 设置测试次数
TEST_COUNT=5
if [ $# -gt 0 ]; then
  TEST_COUNT=$1
fi

echo "开始进行 $TEST_COUNT 次线程创建时间测试..."

# 用于存储所有测试结果的数组
declare -a RESULTS

# 进行多次测试
for ((i=1; i<=$TEST_COUNT; i++)); do
  echo "===================="
  echo "运行测试 $i/$TEST_COUNT"
  echo "===================="
  
  # 创建临时文件保存输出
  TEMP_OUTPUT_FILE=$(mktemp)
  
  # 使用超时并在后台运行cargo，同时将输出重定向到临时文件
  timeout --foreground 5s cargo run > "$TEMP_OUTPUT_FILE" 2>&1 &
  CARGO_PID=$!
  
  # 监控输出文件，直到发现测试完成的标记
  COMPLETE=0
  while [ $COMPLETE -eq 0 ]; do
    if grep -q "线程创建时间测试完成" "$TEMP_OUTPUT_FILE" || grep -q "平均每个线程创建时间" "$TEMP_OUTPUT_FILE"; then
      echo "测试 $i 已完成"
      COMPLETE=1
      # 终止cargo进程及其子进程
      pkill -P $CARGO_PID
      kill -9 $CARGO_PID 2>/dev/null
    fi
    
    # 如果进程已经结束，也退出循环
    if ! ps -p $CARGO_PID > /dev/null; then
      COMPLETE=1
    fi
    
    # 等待一小段时间再检查
    sleep 0.5
  done
  
  # 读取输出
  OUTPUT=$(cat "$TEMP_OUTPUT_FILE")
  
  # 清理临时文件
  rm -f "$TEMP_OUTPUT_FILE"
  
  # 从输出中提取平均创建时间（微秒）
  AVG_TIME=$(echo "$OUTPUT" | grep "平均每个线程创建时间.*微秒" | awk '{print $(NF-1)}')
  
  if [ -n "$AVG_TIME" ]; then
    echo "测试 $i 完成: 平均线程创建时间 = $AVG_TIME us"
    RESULTS+=($AVG_TIME)
  else
    echo "警告: 测试 $i 未能获取结果"
    # 如果需要调试，可以取消下面两行的注释
    # echo "输出内容摘要:"
    # echo "$OUTPUT" | tail -n 20
  fi
  
  # 确保所有相关进程都已终止
  pkill -f "cargo run" 2>/dev/null
  
  # 短暂暂停，让系统恢复
  sleep 2
done

# 计算平均值
SUM=0
COUNT=0
# 初始化最小值和最大值
MIN_VALUE=""
MAX_VALUE=""

for result in "${RESULTS[@]}"; do
  SUM=$(echo "$SUM + $result" | bc -l)
  COUNT=$((COUNT + 1))
  
  # 更新最小值
  if [ -z "$MIN_VALUE" ] || [ $(echo "$result < $MIN_VALUE" | bc -l) -eq 1 ]; then
    MIN_VALUE=$result
  fi
  
  # 更新最大值
  if [ -z "$MAX_VALUE" ] || [ $(echo "$result > $MAX_VALUE" | bc -l) -eq 1 ]; then
    MAX_VALUE=$result
  fi
done

if [ $COUNT -gt 0 ]; then
  AVERAGE=$(echo "scale=2; $SUM / $COUNT" | bc -l)
  
  echo "===================="
  echo "测试结果汇总"
  echo "===================="
  echo "总测试次数: $COUNT"
  echo "所有测试的平均线程创建时间: $AVERAGE us"
  echo "最小值: $MIN_VALUE us"
  echo "最大值: $MAX_VALUE us"
  
  # 计算标准差
  if [ $COUNT -gt 1 ]; then
    VARIANCE=0
    for result in "${RESULTS[@]}"; do
      DIFF=$(echo "$result - $AVERAGE" | bc -l)
      SQUARE=$(echo "$DIFF * $DIFF" | bc -l)
      VARIANCE=$(echo "$VARIANCE + $SQUARE" | bc -l)
    done
    STD_DEV=$(echo "scale=2; sqrt($VARIANCE / ($COUNT - 1))" | bc -l)
    echo "标准差: 0$STD_DEV us"
  fi
  
  # 显示所有测试结果
  echo "===================="
  echo "各次测试结果 (us):"
  for ((i=0; i<$COUNT; i++)); do
    echo "测试 $((i+1)): ${RESULTS[$i]}"
  done
else
  echo "错误: 没有成功的测试结果"
  exit 1
fi 
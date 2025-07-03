#!/bin/bash

# Ray集群测试自动化脚本
# 自动执行多次测试，从1到16个工作节点，每个节点数测量多次

# 配置参数
NUM_MEASUREMENTS=10  # 每个工作节点数测量5次

echo "🚀 开始Ray集群测试自动化执行"
echo "=================================="
echo "📊 每个工作节点数将测量 $NUM_MEASUREMENTS 次"
echo ""

# 检查是否在集群环境中运行
if [ -n "$RAY_HEAD_ADDRESS" ]; then
    echo "🔗 检测到集群环境，头节点地址: $RAY_HEAD_ADDRESS"
    CLUSTER_MODE=true
else
    echo "🏠 本地模式运行"
    CLUSTER_MODE=false
fi

# 创建结果目录
RESULTS_DIR="cluster_test_results"
mkdir -p $RESULTS_DIR

# 清理之前的报告文件
echo "🧹 清理之前的测试报告..."
rm -f cluster_report_*.json

# 执行测试循环
for workers in {1..5}
do
    echo ""
    echo "🔧 执行测试: --num-workers $workers (测量 $NUM_MEASUREMENTS 次)"
    echo "----------------------------------------"
    
    # 为每个工作节点数执行多次测量
    for measurement in $(seq 1 $NUM_MEASUREMENTS)
    do
        echo "   测量 $measurement/$NUM_MEASUREMENTS"
        
        # 执行测试
        if [ "$CLUSTER_MODE" = true ]; then
            # 集群模式：作为工作节点连接到头节点
            python ClusterPrimeTest.py --worker --head-address="$RAY_HEAD_ADDRESS" --num-workers $workers
        else
            # 本地模式
            python ClusterPrimeTest.py --num-workers $workers
        fi
        
        # 等待一下确保文件写入完成
        sleep 2
        
        # 移动生成的报告文件到结果目录
        if ls cluster_report_*.json 1> /dev/null 2>&1; then
            for report in cluster_report_*.json; do
                # 重命名文件以包含工作节点数和测量次数
                new_name="${RESULTS_DIR}/cluster_report_workers_${workers}_measurement_${measurement}_${report#cluster_report_}"
                mv "$report" "$new_name"
                echo "      📄 保存报告: $new_name"
            done
        else
            echo "      ⚠️  未找到测试报告文件"
        fi
        
        # 测量间隔
        if [ $measurement -lt $NUM_MEASUREMENTS ]; then
            echo "      ⏳ 等待 5 秒后进行下一次测量..."
            sleep 5
        fi
    done
    
    echo "✅ 完成 $workers 个工作节点的 $NUM_MEASUREMENTS 次测量"
    echo ""
done

echo "🎉 所有测试完成！"
echo "📁 结果保存在: $RESULTS_DIR/"
echo "📊 现在可以运行数据分析程序: python anylise.py" 
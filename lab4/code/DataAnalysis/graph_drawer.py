'''
根据已有数据绘制图表

功能说明：
给定一个 cluster_size：
1. 绘制 total_workers 和 throughput 的折线图（带误差条）
2. 绘制 total_workers 和 average_efficiency 的折线图（带误差条）
3. 绘制 total_workers 和 avg_cpu_percent 的折线图（带误差条）
4. 绘制 total_workers 和 avg_memory_percent 的折线图（带误差条）
5. 绘制 total_workers 和 max_cpu_percent 的折线图（带误差条）
6. 绘制 total_workers 和 max_memory_percent 的折线图（带误差条）

给定一个 config.workers：
1. 绘制 cluster_size 和 throughput 的折线图（带误差条）
2. 绘制 cluster_size 和 average_efficiency 的折线图（带误差条）
3. 绘制 cluster_size 和 avg_cpu_percent 的折线图（带误差条）
4. 绘制 cluster_size 和 avg_memory_percent 的折线图（带误差条）
5. 绘制 cluster_size 和 max_cpu_percent 的折线图（带误差条）
6. 绘制 cluster_size 和 max_memory_percent 的折线图（带误差条）
'''

import os
import json
import matplotlib.pyplot as plt
import numpy as np
from collections import defaultdict

# 设置中文字体
import matplotlib
matplotlib.rcParams['font.sans-serif'] = ['SimHei', 'Microsoft YaHei', 'WenQuanYi Micro Hei', 'DejaVu Sans']
matplotlib.rcParams['axes.unicode_minus'] = False
matplotlib.rcParams['font.family'] = 'sans-serif'

# 读取 ../DataAnalysis/cluster_test_results 下的所有 json 文件
cluster_test_results_dir = "../DataAnalysis/cluster_test_results"
json_files = [f for f in os.listdir(cluster_test_results_dir) if f.endswith('.json')]

img_dir = "../DataAnalysis/img"
os.makedirs(img_dir, exist_ok=True)


def load_data():
    """加载所有JSON文件的数据"""
    data_list = []
    for json_file in json_files:
        with open(os.path.join(cluster_test_results_dir, json_file), 'r') as f:
            data = json.load(f)
            data_list.append(data)
    return data_list

def calculate_stats(values):
    """计算平均值和标准差"""
    if not values:
        return 0, 0
    mean_val = np.mean(values)
    std_val = np.std(values, ddof=1)  # 使用样本标准差
    return mean_val, std_val

# -------------------给定 cluster_size-------------------

def draw_throughput_graph_by_cluster_size(cluster_size):
    """绘制指定cluster_size下total_workers和throughput的关系图（带误差条）"""
    data_list = load_data()
    
    # 筛选指定cluster_size的数据
    filtered_data = [data for data in data_list if data['stats']['cluster_size'] == cluster_size]
    
    if not filtered_data:
        print(f"没有找到cluster_size为{cluster_size}的数据")
        return
    
    # 按total_workers分组数据
    workers_data = defaultdict(list)
    for data in filtered_data:
        total_workers = data['stats']['total_workers']
        throughput = data['stats']['throughput']['prime_generation']['avg_throughput']
        workers_data[total_workers].append(throughput)
    
    # 计算每个total_workers的平均值和标准差
    total_workers_list = []
    throughput_means = []
    throughput_stds = []
    
    for total_workers in sorted(workers_data.keys()):
        values = workers_data[total_workers]
        mean_val, std_val = calculate_stats(values)
        total_workers_list.append(total_workers)
        throughput_means.append(mean_val)
        throughput_stds.append(std_val)
    
    # 绘制图表
    plt.figure(figsize=(10, 6))
    plt.errorbar(total_workers_list, throughput_means, yerr=throughput_stds, 
                fmt='o-', linewidth=2, markersize=8, capsize=5, capthick=2)
    plt.xlabel('Total Workers')
    plt.ylabel('Average Throughput')
    plt.title(f'Cluster Size {cluster_size} - Total Workers vs Throughput (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    # 保存图表
    output_dir = os.path.join(img_dir, f"cluster_size_{cluster_size}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "throughput_vs_total_workers.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_average_efficiency_graph_by_cluster_size(cluster_size):
    """绘制指定cluster_size下total_workers和average_efficiency的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['stats']['cluster_size'] == cluster_size]
    
    if not filtered_data:
        print(f"没有找到cluster_size为{cluster_size}的数据")
        return
    
    # 按total_workers分组数据
    workers_data = defaultdict(list)
    for data in filtered_data:
        total_workers = data['stats']['total_workers']
        efficiency = data['stats']['summary']['average_efficiency']
        workers_data[total_workers].append(efficiency)
    
    # 计算每个total_workers的平均值和标准差
    total_workers_list = []
    efficiency_means = []
    efficiency_stds = []
    
    for total_workers in sorted(workers_data.keys()):
        values = workers_data[total_workers]
        mean_val, std_val = calculate_stats(values)
        total_workers_list.append(total_workers)
        efficiency_means.append(mean_val)
        efficiency_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(total_workers_list, efficiency_means, yerr=efficiency_stds, 
                fmt='o-', linewidth=2, markersize=8, color='green', capsize=5, capthick=2)
    plt.xlabel('Total Workers')
    plt.ylabel('Average Efficiency')
    plt.title(f'Cluster Size {cluster_size} - Total Workers vs Efficiency (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"cluster_size_{cluster_size}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "efficiency_vs_total_workers.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_avg_cpu_percent_graph_by_cluster_size(cluster_size):
    """绘制指定cluster_size下total_workers和avg_cpu_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['stats']['cluster_size'] == cluster_size]
    
    if not filtered_data:
        print(f"没有找到cluster_size为{cluster_size}的数据")
        return
    
    # 按total_workers分组数据
    workers_data = defaultdict(list)
    for data in filtered_data:
        total_workers = data['stats']['total_workers']
        cpu_percent = data['stats']['monitoring']['avg_cpu_percent']
        workers_data[total_workers].append(cpu_percent)
    
    # 计算每个total_workers的平均值和标准差
    total_workers_list = []
    cpu_means = []
    cpu_stds = []
    
    for total_workers in sorted(workers_data.keys()):
        values = workers_data[total_workers]
        mean_val, std_val = calculate_stats(values)
        total_workers_list.append(total_workers)
        cpu_means.append(mean_val)
        cpu_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(total_workers_list, cpu_means, yerr=cpu_stds, 
                fmt='o-', linewidth=2, markersize=8, color='red', capsize=5, capthick=2)
    plt.xlabel('Total Workers')
    plt.ylabel('Average CPU Usage (%)')
    plt.title(f'Cluster Size {cluster_size} - Total Workers vs CPU Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"cluster_size_{cluster_size}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "avg_cpu_vs_total_workers.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_avg_memory_percent_graph_by_cluster_size(cluster_size):
    """绘制指定cluster_size下total_workers和avg_memory_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['stats']['cluster_size'] == cluster_size]
    
    if not filtered_data:
        print(f"没有找到cluster_size为{cluster_size}的数据")
        return
    
    # 按total_workers分组数据
    workers_data = defaultdict(list)
    for data in filtered_data:
        total_workers = data['stats']['total_workers']
        memory_percent = data['stats']['monitoring']['avg_memory_percent']
        workers_data[total_workers].append(memory_percent)
    
    # 计算每个total_workers的平均值和标准差
    total_workers_list = []
    memory_means = []
    memory_stds = []
    
    for total_workers in sorted(workers_data.keys()):
        values = workers_data[total_workers]
        mean_val, std_val = calculate_stats(values)
        total_workers_list.append(total_workers)
        memory_means.append(mean_val)
        memory_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(total_workers_list, memory_means, yerr=memory_stds, 
                fmt='o-', linewidth=2, markersize=8, color='purple', capsize=5, capthick=2)
    plt.xlabel('Total Workers')
    plt.ylabel('Average Memory Usage (%)')
    plt.title(f'Cluster Size {cluster_size} - Total Workers vs Memory Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"cluster_size_{cluster_size}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "avg_memory_vs_total_workers.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_max_cpu_percent_graph_by_cluster_size(cluster_size):
    """绘制指定cluster_size下total_workers和max_cpu_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['stats']['cluster_size'] == cluster_size]
    
    if not filtered_data:
        print(f"没有找到cluster_size为{cluster_size}的数据")
        return
    
    # 按total_workers分组数据
    workers_data = defaultdict(list)
    for data in filtered_data:
        total_workers = data['stats']['total_workers']
        max_cpu_percent = data['stats']['monitoring']['max_cpu_percent']
        workers_data[total_workers].append(max_cpu_percent)
    
    # 计算每个total_workers的平均值和标准差
    total_workers_list = []
    max_cpu_means = []
    max_cpu_stds = []
    
    for total_workers in sorted(workers_data.keys()):
        values = workers_data[total_workers]
        mean_val, std_val = calculate_stats(values)
        total_workers_list.append(total_workers)
        max_cpu_means.append(mean_val)
        max_cpu_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(total_workers_list, max_cpu_means, yerr=max_cpu_stds, 
                fmt='o-', linewidth=2, markersize=8, color='darkred', capsize=5, capthick=2)
    plt.xlabel('Total Workers')
    plt.ylabel('Max CPU Usage (%)')
    plt.title(f'Cluster Size {cluster_size} - Total Workers vs Max CPU Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"cluster_size_{cluster_size}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "max_cpu_vs_total_workers.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_max_memory_percent_graph_by_cluster_size(cluster_size):
    """绘制指定cluster_size下total_workers和max_memory_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['stats']['cluster_size'] == cluster_size]
    
    if not filtered_data:
        print(f"没有找到cluster_size为{cluster_size}的数据")
        return
    
    # 按total_workers分组数据
    workers_data = defaultdict(list)
    for data in filtered_data:
        total_workers = data['stats']['total_workers']
        max_memory_percent = data['stats']['monitoring']['max_memory_percent']
        workers_data[total_workers].append(max_memory_percent)
    
    # 计算每个total_workers的平均值和标准差
    total_workers_list = []
    max_memory_means = []
    max_memory_stds = []
    
    for total_workers in sorted(workers_data.keys()):
        values = workers_data[total_workers]
        mean_val, std_val = calculate_stats(values)
        total_workers_list.append(total_workers)
        max_memory_means.append(mean_val)
        max_memory_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(total_workers_list, max_memory_means, yerr=max_memory_stds, 
                fmt='o-', linewidth=2, markersize=8, color='darkmagenta', capsize=5, capthick=2)
    plt.xlabel('Total Workers')
    plt.ylabel('Max Memory Usage (%)')
    plt.title(f'Cluster Size {cluster_size} - Total Workers vs Max Memory Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"cluster_size_{cluster_size}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "max_memory_vs_total_workers.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_all_graphs_by_cluster_size(cluster_size):
    """绘制指定cluster_size的所有图表"""
    print(f"正在为cluster_size {cluster_size}绘制图表...")
    draw_throughput_graph_by_cluster_size(cluster_size)
    draw_average_efficiency_graph_by_cluster_size(cluster_size)
    draw_avg_cpu_percent_graph_by_cluster_size(cluster_size)
    draw_avg_memory_percent_graph_by_cluster_size(cluster_size)
    draw_max_cpu_percent_graph_by_cluster_size(cluster_size)
    draw_max_memory_percent_graph_by_cluster_size(cluster_size)
    print(f"cluster_size {cluster_size}的图表绘制完成")

# -------------------给定 config.workers-------------------

def draw_throughput_graph_by_config_workers(config_workers):
    """绘制指定config.workers下cluster_size和throughput的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['config']['workers'] == config_workers]
    
    if not filtered_data:
        print(f"没有找到config.workers为{config_workers}的数据")
        return
    
    # 按cluster_size分组数据
    cluster_data = defaultdict(list)
    for data in filtered_data:
        cluster_size = data['stats']['cluster_size']
        throughput = data['stats']['throughput']['prime_generation']['avg_throughput']
        cluster_data[cluster_size].append(throughput)
    
    # 计算每个cluster_size的平均值和标准差
    cluster_size_list = []
    throughput_means = []
    throughput_stds = []
    
    for cluster_size in sorted(cluster_data.keys()):
        values = cluster_data[cluster_size]
        mean_val, std_val = calculate_stats(values)
        cluster_size_list.append(cluster_size)
        throughput_means.append(mean_val)
        throughput_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(cluster_size_list, throughput_means, yerr=throughput_stds, 
                fmt='o-', linewidth=2, markersize=8, color='blue', capsize=5, capthick=2)
    plt.xlabel('Cluster Size')
    plt.ylabel('Average Throughput')
    plt.title(f'Workers {config_workers} - Cluster Size vs Throughput (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"config_workers_{config_workers}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "throughput_vs_cluster_size.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_average_efficiency_graph_by_config_workers(config_workers):
    """绘制指定config.workers下cluster_size和average_efficiency的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['config']['workers'] == config_workers]
    
    if not filtered_data:
        print(f"没有找到config.workers为{config_workers}的数据")
        return
    
    # 按cluster_size分组数据
    cluster_data = defaultdict(list)
    for data in filtered_data:
        cluster_size = data['stats']['cluster_size']
        efficiency = data['stats']['summary']['average_efficiency']
        cluster_data[cluster_size].append(efficiency)
    
    # 计算每个cluster_size的平均值和标准差
    cluster_size_list = []
    efficiency_means = []
    efficiency_stds = []
    
    for cluster_size in sorted(cluster_data.keys()):
        values = cluster_data[cluster_size]
        mean_val, std_val = calculate_stats(values)
        cluster_size_list.append(cluster_size)
        efficiency_means.append(mean_val)
        efficiency_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(cluster_size_list, efficiency_means, yerr=efficiency_stds, 
                fmt='o-', linewidth=2, markersize=8, color='green', capsize=5, capthick=2)
    plt.xlabel('Cluster Size')
    plt.ylabel('Average Efficiency')
    plt.title(f'Workers {config_workers} - Cluster Size vs Efficiency (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"config_workers_{config_workers}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "efficiency_vs_cluster_size.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_avg_cpu_percent_graph_by_config_workers(config_workers):
    """绘制指定config.workers下cluster_size和avg_cpu_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['config']['workers'] == config_workers]
    
    if not filtered_data:
        print(f"没有找到config.workers为{config_workers}的数据")
        return
    
    # 按cluster_size分组数据
    cluster_data = defaultdict(list)
    for data in filtered_data:
        cluster_size = data['stats']['cluster_size']
        cpu_percent = data['stats']['monitoring']['avg_cpu_percent']
        cluster_data[cluster_size].append(cpu_percent)
    
    # 计算每个cluster_size的平均值和标准差
    cluster_size_list = []
    cpu_means = []
    cpu_stds = []
    
    for cluster_size in sorted(cluster_data.keys()):
        values = cluster_data[cluster_size]
        mean_val, std_val = calculate_stats(values)
        cluster_size_list.append(cluster_size)
        cpu_means.append(mean_val)
        cpu_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(cluster_size_list, cpu_means, yerr=cpu_stds, 
                fmt='o-', linewidth=2, markersize=8, color='red', capsize=5, capthick=2)
    plt.xlabel('Cluster Size')
    plt.ylabel('Average CPU Usage (%)')
    plt.title(f'Workers {config_workers} - Cluster Size vs CPU Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"config_workers_{config_workers}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "avg_cpu_vs_cluster_size.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_avg_memory_percent_graph_by_config_workers(config_workers):
    """绘制指定config.workers下cluster_size和avg_memory_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['config']['workers'] == config_workers]
    
    if not filtered_data:
        print(f"没有找到config.workers为{config_workers}的数据")
        return
    
    # 按cluster_size分组数据
    cluster_data = defaultdict(list)
    for data in filtered_data:
        cluster_size = data['stats']['cluster_size']
        memory_percent = data['stats']['monitoring']['avg_memory_percent']
        cluster_data[cluster_size].append(memory_percent)
    
    # 计算每个cluster_size的平均值和标准差
    cluster_size_list = []
    memory_means = []
    memory_stds = []
    
    for cluster_size in sorted(cluster_data.keys()):
        values = cluster_data[cluster_size]
        mean_val, std_val = calculate_stats(values)
        cluster_size_list.append(cluster_size)
        memory_means.append(mean_val)
        memory_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(cluster_size_list, memory_means, yerr=memory_stds, 
                fmt='o-', linewidth=2, markersize=8, color='purple', capsize=5, capthick=2)
    plt.xlabel('Cluster Size')
    plt.ylabel('Average Memory Usage (%)')
    plt.title(f'Workers {config_workers} - Cluster Size vs Memory Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"config_workers_{config_workers}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "avg_memory_vs_cluster_size.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_max_cpu_percent_graph_by_config_workers(config_workers):
    """绘制指定config.workers下cluster_size和max_cpu_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['config']['workers'] == config_workers]
    
    if not filtered_data:
        print(f"没有找到config.workers为{config_workers}的数据")
        return
    
    # 按cluster_size分组数据
    cluster_data = defaultdict(list)
    for data in filtered_data:
        cluster_size = data['stats']['cluster_size']
        max_cpu_percent = data['stats']['monitoring']['max_cpu_percent']
        cluster_data[cluster_size].append(max_cpu_percent)
    
    # 计算每个cluster_size的平均值和标准差
    cluster_size_list = []
    max_cpu_means = []
    max_cpu_stds = []
    
    for cluster_size in sorted(cluster_data.keys()):
        values = cluster_data[cluster_size]
        mean_val, std_val = calculate_stats(values)
        cluster_size_list.append(cluster_size)
        max_cpu_means.append(mean_val)
        max_cpu_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(cluster_size_list, max_cpu_means, yerr=max_cpu_stds, 
                fmt='o-', linewidth=2, markersize=8, color='darkred', capsize=5, capthick=2)
    plt.xlabel('Cluster Size')
    plt.ylabel('Max CPU Usage (%)')
    plt.title(f'Workers {config_workers} - Cluster Size vs Max CPU Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"config_workers_{config_workers}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "max_cpu_vs_cluster_size.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_max_memory_percent_graph_by_config_workers(config_workers):
    """绘制指定config.workers下cluster_size和max_memory_percent的关系图（带误差条）"""
    data_list = load_data()
    
    filtered_data = [data for data in data_list if data['config']['workers'] == config_workers]
    
    if not filtered_data:
        print(f"没有找到config.workers为{config_workers}的数据")
        return
    
    # 按cluster_size分组数据
    cluster_data = defaultdict(list)
    for data in filtered_data:
        cluster_size = data['stats']['cluster_size']
        max_memory_percent = data['stats']['monitoring']['max_memory_percent']
        cluster_data[cluster_size].append(max_memory_percent)
    
    # 计算每个cluster_size的平均值和标准差
    cluster_size_list = []
    max_memory_means = []
    max_memory_stds = []
    
    for cluster_size in sorted(cluster_data.keys()):
        values = cluster_data[cluster_size]
        mean_val, std_val = calculate_stats(values)
        cluster_size_list.append(cluster_size)
        max_memory_means.append(mean_val)
        max_memory_stds.append(std_val)
    
    plt.figure(figsize=(10, 6))
    plt.errorbar(cluster_size_list, max_memory_means, yerr=max_memory_stds, 
                fmt='o-', linewidth=2, markersize=8, color='darkmagenta', capsize=5, capthick=2)
    plt.xlabel('Cluster Size')
    plt.ylabel('Max Memory Usage (%)')
    plt.title(f'Workers {config_workers} - Cluster Size vs Max Memory Usage (with Error Bars)')
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_dir = os.path.join(img_dir, f"config_workers_{config_workers}")
    os.makedirs(output_dir, exist_ok=True)
    plt.savefig(os.path.join(output_dir, "max_memory_vs_cluster_size.png"), dpi=300, bbox_inches='tight')
    plt.close()

def draw_all_graphs_by_config_workers(config_workers):
    """绘制指定config.workers的所有图表"""
    print(f"正在为config.workers {config_workers}绘制图表...")
    draw_throughput_graph_by_config_workers(config_workers)
    draw_average_efficiency_graph_by_config_workers(config_workers)
    draw_avg_cpu_percent_graph_by_config_workers(config_workers)
    draw_avg_memory_percent_graph_by_config_workers(config_workers)
    draw_max_cpu_percent_graph_by_config_workers(config_workers)
    draw_max_memory_percent_graph_by_config_workers(config_workers)
    print(f"config.workers {config_workers}的图表绘制完成")

def main():
    # 获取所有可能的 cluster_size
    data_list = load_data()
    cluster_size_list = []
    for data in data_list:
        cluster_size_list.append(data['stats']['cluster_size'])
    cluster_size_list = list(set(cluster_size_list))
    cluster_size_list.sort()
    print(f"发现的cluster_size: {cluster_size_list}")
    
    for cluster_size in cluster_size_list:
        draw_all_graphs_by_cluster_size(cluster_size)

    # 获取所有可能的 config.workers
    config_workers_list = []
    for data in data_list:
        config_workers_list.append(data['config']['workers'])
    config_workers_list = list(set(config_workers_list))
    config_workers_list.sort()
    print(f"发现的config.workers: {config_workers_list}")
    
    for config_workers in config_workers_list:
        draw_all_graphs_by_config_workers(config_workers)
    
    print("所有图表绘制完成！")

if __name__ == "__main__":
    main()
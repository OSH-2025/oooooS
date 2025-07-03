#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Ray集群大素数寻找与分解测试程序
功能: 多节点集群分布式计算，吞吐量测量和性能监控

本程序专门为集群环境设计，支持：
1. 多节点Ray集群配置
2. 实时吞吐量测量
3. 集群资源监控
4. 负载均衡优化
5. 性能报告生成

集群部署说明：
1. 在头节点运行: python ClusterPrimeTest.py --head
2. 在工作节点运行: python ClusterPrimeTest.py --worker --head-address=ray://head-ip:10001
3. 或者使用Ray集群启动脚本
"""

# ==================== 导入必要的库 ====================
import ray
import time
import random
import math
import argparse
import json
import os
import psutil
import threading
from typing import List, Tuple, Optional, Union, Dict, Any
from dataclasses import dataclass, asdict
from datetime import datetime
import multiprocessing

# 导入素数验证模块
try:
    from prime_validator import PrimeValidator
    VALIDATOR_AVAILABLE = True
    print("✅ 素数验证模块可用")
except ImportError:
    VALIDATOR_AVAILABLE = False
    print("❌ 素数验证模块不可用")

# ==================== 集群配置类 ====================
@dataclass
class ClusterConfig:
    """集群配置类"""
    head_address: str = "localhost:10001"
    num_workers_per_node: int = 16
    total_nodes: int = 1
    enable_monitoring: bool = True
    log_level: str = "INFO"
    object_store_memory: int = 2000000000  # 2GB
    num_cpus: Optional[int] = None
    num_gpus: int = 0
    resources: Optional[Dict[str, float]] = None

# ==================== 性能监控类 ====================
@dataclass
class PerformanceMetrics:
    """性能指标数据类"""
    timestamp: float
    node_id: str
    cpu_percent: float
    memory_percent: float
    network_io: Dict[str, float]
    disk_io: Dict[str, float]
    ray_objects: int
    ray_tasks: int

class ClusterMonitor:
    """集群性能监控器"""
    
    def __init__(self, config: ClusterConfig):
        self.config = config
        self.metrics_history: List[PerformanceMetrics] = []
        self.monitoring = False
        self.monitor_thread = None
    
    def start_monitoring(self):
        """开始监控"""
        if not self.config.enable_monitoring:
            return
        
        self.monitoring = True
        self.monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.monitor_thread.start()
        print("🔍 集群监控已启动")
    
    def stop_monitoring(self):
        """停止监控"""
        self.monitoring = False
        if self.monitor_thread:
            self.monitor_thread.join()
        print("🔍 集群监控已停止")
    
    def _monitor_loop(self):
        """监控循环"""
        while self.monitoring:
            try:
                metrics = self._collect_metrics()
                self.metrics_history.append(metrics)
                time.sleep(1)  # 每秒收集一次
            except Exception as e:
                print(f"监控错误: {e}")
    
    def _collect_metrics(self) -> PerformanceMetrics:
        """收集性能指标"""
        cpu_percent = psutil.cpu_percent(interval=0.1)
        memory = psutil.virtual_memory()
        memory_percent = memory.percent

        net_io = psutil.net_io_counters()
        network_io = {k: float(v) for k, v in {
            'bytes_sent': net_io.bytes_sent,
            'bytes_recv': net_io.bytes_recv,
            'packets_sent': net_io.packets_sent,
            'packets_recv': net_io.packets_recv
        }.items()}

        disk_io = psutil.disk_io_counters()
        disk_metrics = {k: float(v) for k, v in {
            'read_bytes': disk_io.read_bytes if disk_io else 0,
            'write_bytes': disk_io.write_bytes if disk_io else 0,
            'read_count': disk_io.read_count if disk_io else 0,
            'write_count': disk_io.write_count if disk_io else 0
        }.items()}

        # Ray指标（不再统计，设为0）
        ray_objects = 0
        ray_tasks = 0

        return PerformanceMetrics(
            timestamp=time.time(),
            node_id=str(ray.get_runtime_context().get_node_id()),
            cpu_percent=cpu_percent,
            memory_percent=memory_percent,
            network_io=network_io,
            disk_io=disk_metrics,
            ray_objects=ray_objects,
            ray_tasks=ray_tasks
        )
    
    def get_summary(self) -> Dict[str, Any]:
        """获取监控摘要"""
        if not self.metrics_history:
            return {}
        
        # 计算平均值
        avg_cpu = sum(m.cpu_percent for m in self.metrics_history) / len(self.metrics_history)
        avg_memory = sum(m.memory_percent for m in self.metrics_history) / len(self.metrics_history)
        avg_ray_objects = sum(m.ray_objects for m in self.metrics_history) / len(self.metrics_history)
        
        # 计算峰值
        max_cpu = max(m.cpu_percent for m in self.metrics_history)
        max_memory = max(m.memory_percent for m in self.metrics_history)
        
        return {
            'monitoring_duration': self.metrics_history[-1].timestamp - self.metrics_history[0].timestamp,
            'avg_cpu_percent': avg_cpu,
            'avg_memory_percent': avg_memory,
            'max_cpu_percent': max_cpu,
            'max_memory_percent': max_memory,
            'avg_ray_objects': avg_ray_objects,
            'total_metrics_collected': len(self.metrics_history)
        }

# ==================== 吞吐量测量类 ====================
@dataclass
class ThroughputMetrics:
    """吞吐量指标"""
    operation_type: str  # "prime_generation" 或 "factorization"
    bits: int
    total_operations: int
    successful_operations: int
    total_time: float
    throughput: float  # 每秒操作数
    avg_time_per_operation: float
    cluster_size: int
    timestamp: float

class ThroughputMeasurer:
    """吞吐量测量器"""
    
    def __init__(self):
        self.metrics_history: List[ThroughputMetrics] = []
    
    def measure_prime_generation(self, bits: int, num_primes: int, 
                               total_time: float, successful_primes: int,
                               cluster_size: int) -> ThroughputMetrics:
        """测量素数生成吞吐量"""
        throughput = successful_primes / total_time if total_time > 0 else 0
        avg_time = total_time / successful_primes if successful_primes > 0 else 0
        
        metrics = ThroughputMetrics(
            operation_type="prime_generation",
            bits=bits,
            total_operations=num_primes,
            successful_operations=successful_primes,
            total_time=total_time,
            throughput=throughput,
            avg_time_per_operation=avg_time,
            cluster_size=cluster_size,
            timestamp=time.time()
        )
        
        self.metrics_history.append(metrics)
        return metrics
    
    def measure_factorization(self, numbers: List[int], total_time: float,
                            successful_factorizations: int, cluster_size: int) -> ThroughputMetrics:
        """测量大数分解吞吐量"""
        throughput = successful_factorizations / total_time if total_time > 0 else 0
        avg_time = total_time / successful_factorizations if successful_factorizations > 0 else 0
        
        # 使用最大位数作为指标
        max_bits = max(len(bin(abs(num))) - 2 for num in numbers) if numbers else 0
        
        metrics = ThroughputMetrics(
            operation_type="factorization",
            bits=max_bits,
            total_operations=len(numbers),
            successful_operations=successful_factorizations,
            total_time=total_time,
            throughput=throughput,
            avg_time_per_operation=avg_time,
            cluster_size=cluster_size,
            timestamp=time.time()
        )
        
        self.metrics_history.append(metrics)
        return metrics
    
    def get_summary(self) -> Dict[str, Any]:
        """获取吞吐量摘要"""
        if not self.metrics_history:
            return {}
        
        # 按操作类型分组
        prime_metrics = [m for m in self.metrics_history if m.operation_type == "prime_generation"]
        factor_metrics = [m for m in self.metrics_history if m.operation_type == "factorization"]
        
        summary = {
            'total_operations': len(self.metrics_history),
            'prime_generation': {
                'total_operations': len(prime_metrics),
                'avg_throughput': sum(m.throughput for m in prime_metrics) / len(prime_metrics) if prime_metrics else 0,
                'max_throughput': max(m.throughput for m in prime_metrics) if prime_metrics else 0,
                'total_primes_generated': sum(m.successful_operations for m in prime_metrics)
            },
            'factorization': {
                'total_operations': len(factor_metrics),
                'avg_throughput': sum(m.throughput for m in factor_metrics) / len(factor_metrics) if factor_metrics else 0,
                'max_throughput': max(m.throughput for m in factor_metrics) if factor_metrics else 0,
                'total_factorizations': sum(m.successful_operations for m in factor_metrics)
            }
        }
        
        return summary

# ==================== 集群素数工作节点 ====================
@ray.remote
class ClusterPrimeWorker:
    """集群素数工作节点 - 增强版"""
    
    def __init__(self, worker_id: int, node_id: Optional[str] = None):
        self.worker_id = worker_id
        self.node_id = node_id or str(ray.get_runtime_context().get_node_id())
        self.primes_found = 0
        self.factorizations_completed = 0
        self.total_work_time = 0.0
        self.start_time = time.time()
    
    def miller_rabin_test(self, n: int, k: int = 5) -> bool:
        """Miller-Rabin素数测试算法"""
        if n <= 1:
            return False
        if n <= 3:
            return True
        if n % 2 == 0:
            return False
        
        r = 0
        d = n - 1
        while d % 2 == 0:
            r += 1
            d //= 2
        
        for _ in range(k):
            a = random.randrange(2, n - 1)
            x = pow(a, d, n)
            
            if x == 1 or x == n - 1:
                continue
            
            for _ in range(r - 1):
                x = (x * x) % n
                if x == n - 1:
                    break
            else:
                return False
        
        return True
    
    def generate_random_odd_number(self, bits: int) -> int:
        """生成指定位数的随机奇数"""
        n = random.getrandbits(bits)
        n |= (1 << (bits - 1))
        n |= 1
        return n
    
    def find_prime(self, bits: int, max_attempts: int = 1000) -> Optional[int]:
        """寻找指定位数的素数"""
        task_start = time.time()
        attempts = 0
        
        while attempts < max_attempts:
            n = self.generate_random_odd_number(bits)
            if self.miller_rabin_test(n):
                self.primes_found += 1
                self.total_work_time += time.time() - task_start
                return n
            attempts += 1
        
        self.total_work_time += time.time() - task_start
        return None
    
    def pollard_rho_factorization(self, n: int, max_attempts: int = 10) -> List[int]:
        """Pollard's Rho算法进行大数分解"""
        task_start = time.time()
        
        if n <= 1:
            return []
        if self.miller_rabin_test(n):
            self.factorizations_completed += 1
            self.total_work_time += time.time() - task_start
            return [n]
        
        if max_attempts <= 0:
            self.total_work_time += time.time() - task_start
            return [n]
        
        def f(x):
            return (x * x + 1) % n
        
        def gcd(a, b):
            while b:
                a, b = b, a % b
            return a
        
        x = random.randrange(2, n)
        y = x
        d = 1
        
        while d == 1:
            x = f(x)
            y = f(f(y))
            d = gcd(abs(x - y), n)
        
        if d == n:
            result = self.pollard_rho_factorization(n, max_attempts - 1)
            self.total_work_time += time.time() - task_start
            return result
        
        factors = (self.pollard_rho_factorization(d, max_attempts) + 
                  self.pollard_rho_factorization(n // d, max_attempts))
        self.factorizations_completed += 1
        self.total_work_time += time.time() - task_start
        return factors
    
    def get_detailed_stats(self) -> dict:
        """获取详细统计信息"""
        uptime = time.time() - self.start_time
        return {
            'worker_id': self.worker_id,
            'node_id': self.node_id,
            'primes_found': self.primes_found,
            'factorizations_completed': self.factorizations_completed,
            'total_work_time': self.total_work_time,
            'uptime': uptime,
            'efficiency': self.total_work_time / uptime if uptime > 0 else 0
        }

# ==================== 集群测试套件 ====================
class ClusterPrimeTestSuite:
    """集群素数测试套件"""
    
    def __init__(self, config: ClusterConfig):
        self.config = config
        self.monitor = ClusterMonitor(config)
        self.throughput_measurer = ThroughputMeasurer()
        self.workers = []
        self.cluster_size = 0
        
        # 初始化素数验证器
        if VALIDATOR_AVAILABLE:
            self.validator = PrimeValidator()
        else:
            self.validator = None
    
    def initialize_cluster(self):
        """初始化集群"""
        print("🚀 初始化Ray集群...")
        
        # 启动监控
        self.monitor.start_monitoring()
        
        # 等待集群稳定
        time.sleep(2)
        
        # 获取集群信息
        try:
            nodes = ray.nodes()
            self.cluster_size = len(nodes)
            print(f"📊 集群信息: {self.cluster_size} 个节点")
            
            for node in nodes:
                node_id = node['NodeID']
                resources = node['Resources']
                print(f"  节点 {node_id}: CPU={resources.get('CPU', 0)}, "
                      f"内存={resources.get('memory', 0) / 1024**3:.1f}GB")
        except Exception as e:
            print(f"⚠️  无法获取集群信息: {e}")
            self.cluster_size = 1
        
        # 创建工作节点
        total_workers = self.config.num_workers_per_node * self.cluster_size
        print(f"🔧 创建 {total_workers} 个工作节点...")
        
        self.workers = []
        for i in range(total_workers):
            worker = ClusterPrimeWorker.remote(i)
            self.workers.append(worker)
        
        print("✅ 集群初始化完成")
    
    def find_primes_cluster(self, bits: int, num_primes: int) -> List[int]:
        """集群素数生成"""
        print(f"🔍 使用集群寻找 {num_primes} 个 {bits} 位素数...")
        
        start_time = time.time()
        
        # 任务分配
        tasks_per_worker = num_primes // len(self.workers)
        remaining_tasks = num_primes % len(self.workers)
        
        futures = []
        for i, worker in enumerate(self.workers):
            worker_tasks = tasks_per_worker + (1 if i < remaining_tasks else 0)
            for _ in range(worker_tasks):
                future = worker.find_prime.remote(bits)
                futures.append(future)
        
        # 收集结果
        primes = []
        for future in futures:
            prime = ray.get(future)
            if prime:
                primes.append(prime)
        
        total_time = time.time() - start_time
        
        # 测量吞吐量
        metrics = self.throughput_measurer.measure_prime_generation(
            bits, num_primes, total_time, len(primes), self.cluster_size
        )
        
        print(f"✅ 找到 {len(primes)} 个素数")
        print(f"⏱️  耗时: {total_time:.2f} 秒")
        print(f"🚀 吞吐量: {metrics.throughput:.2f} 素数/秒")
        print(f"📊 平均时间: {metrics.avg_time_per_operation:.3f} 秒/素数")
        
        return primes
    
    def factorize_numbers_cluster(self, numbers: List[int]) -> List[Tuple[int, List[int]]]:
        """集群大数分解"""
        print(f"🔍 使用集群分解 {len(numbers)} 个数...")
        
        start_time = time.time()
        
        # 轮询分配任务
        futures = []
        for i, num in enumerate(numbers):
            worker = self.workers[i % len(self.workers)]
            future = worker.pollard_rho_factorization.remote(num)
            futures.append(future)
        
        # 收集结果
        factorizations = []
        for i, future in enumerate(futures):
            factors = ray.get(future)
            factorizations.append((numbers[i], factors))
        
        total_time = time.time() - start_time
        
        # 测量吞吐量
        metrics = self.throughput_measurer.measure_factorization(
            numbers, total_time, len(factorizations), self.cluster_size
        )
        
        print(f"✅ 完成 {len(factorizations)} 个数的分解")
        print(f"⏱️  耗时: {total_time:.2f} 秒")
        print(f"🚀 吞吐量: {metrics.throughput:.2f} 分解/秒")
        print(f"📊 平均时间: {metrics.avg_time_per_operation:.3f} 秒/分解")
        
        return factorizations
    
    def validate_generated_primes(self, primes: List[int]) -> dict:
        """验证生成的素数"""
        if not self.validator or not primes:
            return {'validated': False, 'message': '验证器不可用或无素数'}
        
        print(f"\n🔍 验证 {len(primes)} 个生成的素数...")
        validation_start = time.time()
        
        validation_results = self.validator.validate_primes_batch(primes, method="sympy")
        
        valid_primes = [r for r in validation_results if r['is_prime']]
        invalid_primes = [r for r in validation_results if not r['is_prime']]
        
        validation_time = time.time() - validation_start
        
        stats = {
            'validated': True,
            'total_primes': len(primes),
            'valid_primes': len(valid_primes),
            'invalid_primes': len(invalid_primes),
            'accuracy': len(valid_primes) / len(primes) * 100 if primes else 0,
            'validation_time': validation_time,
            'results': validation_results
        }
        
        print(f"✅ 验证完成，耗时 {validation_time:.4f} 秒")
        print(f"📊 准确率: {stats['accuracy']:.2f}%")
        
        if invalid_primes:
            print(f"⚠️  发现 {len(invalid_primes)} 个错误的素数")
        
        return stats
    
    def get_cluster_stats(self) -> dict:
        """获取集群统计信息"""
        print("\n📊 收集集群统计信息...")
        
        # 获取工作节点统计
        worker_stats = []
        for worker in self.workers:
            stats = ray.get(worker.get_detailed_stats.remote())
            worker_stats.append(stats)
        
        # 计算汇总统计
        total_primes = sum(s['primes_found'] for s in worker_stats)
        total_factorizations = sum(s['factorizations_completed'] for s in worker_stats)
        total_work_time = sum(s['total_work_time'] for s in worker_stats)
        avg_efficiency = sum(s['efficiency'] for s in worker_stats) / len(worker_stats) if worker_stats else 0
        
        # 获取监控摘要
        monitor_summary = self.monitor.get_summary()
        
        # 获取吞吐量摘要
        throughput_summary = self.throughput_measurer.get_summary()
        
        cluster_stats = {
            'cluster_size': self.cluster_size,
            'total_workers': len(self.workers),
            'worker_stats': worker_stats,
            'summary': {
                'total_primes_found': total_primes,
                'total_factorizations': total_factorizations,
                'total_work_time': total_work_time,
                'average_efficiency': avg_efficiency
            },
            'monitoring': monitor_summary,
            'throughput': throughput_summary
        }
        
        return cluster_stats
    
    def run_cluster_test(self, test_config: dict):
        """运行集群测试"""
        print("=" * 80)
        print("🚀 Ray集群大素数寻找与分解测试")
        print("=" * 80)
        
        # 显示测试配置
        print(f"📋 测试配置:")
        print(f"  素数生成任务数: {test_config['num_of_generate']}")
        for i, (bits, count) in enumerate(test_config['generate_list']):
            print(f"    {i+1}. {bits}位素数 x {count}个")
        print(f"  大数分解任务数: {test_config['num_of_factorize']}")
        print(f"  验证启用: {'是' if test_config['validate_enable'] else '否'}")
        print()
        
        all_primes = []
        
        # 素数生成测试
        print("🔍 1. 素数生成测试")
        for i, (bits, count) in enumerate(test_config['generate_list']):
            print(f"\n   任务 {i+1}: 生成 {count} 个 {bits} 位素数")
            primes = self.find_primes_cluster(bits, count)
            all_primes.extend(primes)
            
            if test_config['validate_enable']:
                self.validate_generated_primes(primes)
        
        # 大数分解测试
        print(f"\n🔍 2. 大数分解测试")
        for i, num in enumerate(test_config['factorize_list']):
            print(f"\n   任务 {i+1}: 分解数 {num}")
            factorizations = self.factorize_numbers_cluster([num])
            for num_result, factors in factorizations:
                print(f"   {num_result}: {factors}")
        
        # 生成素数乘积进行分解测试
        if all_primes and test_config['num_of_factorize'] > 0:
            print(f"\n🔍 3. 素数乘积分解测试")
            for i in range(min(test_config['num_of_factorize'], len(all_primes) // 2)):
                if i * 2 + 1 < len(all_primes):
                    num = all_primes[i * 2] * all_primes[i * 2 + 1]
                    print(f"\n   任务 {i+1}: 分解素数乘积 {num}")
                    factorizations = self.factorize_numbers_cluster([num])
                    for num_result, factors in factorizations:
                        print(f"   {num_result}: {factors}")
        
        # 生成性能报告
        print("\n📊 4. 性能报告")
        cluster_stats = self.get_cluster_stats()
        self._print_performance_report(cluster_stats)
        
        # 保存报告
        self._save_report(cluster_stats, test_config)
        
        print("\n✅ 集群测试完成！")
    
    def _print_performance_report(self, stats: dict):
        """打印性能报告"""
        print("\n" + "=" * 60)
        print("📊 集群性能报告")
        print("=" * 60)
        
        # 集群信息
        print(f"🏢 集群规模: {stats['cluster_size']} 个节点")
        print(f"🔧 工作节点: {stats['total_workers']} 个")
        
        # 工作统计
        summary = stats['summary']
        print(f"\n📈 工作统计:")
        print(f"  总素数生成: {summary['total_primes_found']} 个")
        print(f"  总分解任务: {summary['total_factorizations']} 个")
        print(f"  总工作时间: {summary['total_work_time']:.2f} 秒")
        print(f"  平均效率: {summary['average_efficiency']:.2%}")
        
        # 吞吐量统计
        throughput = stats['throughput']
        if throughput:
            print(f"\n🚀 吞吐量统计:")
            if throughput['prime_generation']['total_operations'] > 0:
                prime_gen = throughput['prime_generation']
                print(f"  素数生成:")
                print(f"    平均吞吐量: {prime_gen['avg_throughput']:.2f} 素数/秒")
                print(f"    最大吞吐量: {prime_gen['max_throughput']:.2f} 素数/秒")
                print(f"    总生成数: {prime_gen['total_primes_generated']} 个")
            
            if throughput['factorization']['total_operations'] > 0:
                factor = throughput['factorization']
                print(f"  大数分解:")
                print(f"    平均吞吐量: {factor['avg_throughput']:.2f} 分解/秒")
                print(f"    最大吞吐量: {factor['max_throughput']:.2f} 分解/秒")
                print(f"    总分解数: {factor['total_factorizations']} 个")
        
        # 监控统计
        monitoring = stats['monitoring']
        if monitoring:
            print(f"\n🔍 系统监控:")
            print(f"  监控时长: {monitoring['monitoring_duration']:.1f} 秒")
            print(f"  平均CPU: {monitoring['avg_cpu_percent']:.1f}%")
            print(f"  平均内存: {monitoring['avg_memory_percent']:.1f}%")
            print(f"  峰值CPU: {monitoring['max_cpu_percent']:.1f}%")
            print(f"  峰值内存: {monitoring['max_memory_percent']:.1f}%")
            print(f"  平均Ray对象: {monitoring['avg_ray_objects']:.0f} 个")
    
    def _save_report(self, stats: dict, config: dict):
        """保存性能报告"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"cluster_report_{timestamp}.json"
        
        report = {
            'timestamp': timestamp,
            'config': config,
            'stats': stats
        }
        
        try:
            with open(filename, 'w', encoding='utf-8') as f:
                json.dump(report, f, indent=2, ensure_ascii=False, default=str)
            print(f"📄 性能报告已保存: {filename}")
        except Exception as e:
            print(f"⚠️  保存报告失败: {e}")
    
    def cleanup(self):
        """清理资源"""
        print("🧹 清理集群资源...")
        self.monitor.stop_monitoring()
        ray.shutdown()
        print("✅ 清理完成")

# ==================== 主函数 ====================
def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="Ray集群大素数测试程序")
    parser.add_argument("--head", action="store_true", help="作为头节点运行")
    parser.add_argument("--worker", action="store_true", help="作为工作节点运行")
    parser.add_argument("--head-address", type=str, default="localhost:10001", 
                       help="头节点地址")
    parser.add_argument("--num-workers", type=int, default=4, 
                       help="每个节点的工作节点数")
    parser.add_argument("--object-store-memory", type=int, default=2000000000,
                       help="对象存储内存大小(字节)")
    parser.add_argument("--config-file", type=str, help="测试配置文件")
    
    args = parser.parse_args()
    
    # 配置Ray
    config = ClusterConfig(
        head_address=args.head_address,
        num_workers_per_node=args.num_workers,
        object_store_memory=args.object_store_memory
    )
    
    # 初始化Ray
    if args.head:
        print("🚀 启动Ray头节点...")
        # 使用现代Ray启动方式
        ray.init(
            _system_config={"object_store_memory": config.object_store_memory},
            ignore_reinit_error=True
        )
        print("✅ Ray头节点已启动，等待工作节点连接...")
        print("💡 提示: 在工作节点上运行: python ClusterPrimeTest.py --worker --head-address=<this-node-ip>:10001")
        
        # 保持头节点运行
        try:
            while True:
                time.sleep(10)
                print("🔄 头节点运行中...")
        except KeyboardInterrupt:
            print("\n🛑 头节点停止")
            ray.shutdown()
            return
            
    elif args.worker:
        print(f"🔧 连接到Ray头节点: {args.head_address}")
        # 连接到已启动的Ray集群
        ray.init(
            address=f"ray://{args.head_address}",
            ignore_reinit_error=True
        )
    else:
        # print("🚀 启动本地Ray集群...")
        # # 尝试连接到现有集群，如果失败则启动新的
        # try:
        #     ray.init(
        #         ignore_reinit_error=True,
        #         _system_config={"object_store_memory": config.object_store_memory}
        #     )
        # except Exception as e:
            # print(f"⚠️  启动本地集群失败: {e}")
            print("🔧 尝试连接到默认集群...")
            ray.init(
                ignore_reinit_error=True
            )
    
    # 默认测试配置
    test_config = {
        'num_of_generate': 2,
        'generate_list': [
            (512, 50),   # 512位素数 x 50个
            (1024, 20),  # 1024位素数 x 20个
        ],
        'num_of_factorize': 0,
        'factorize_list': [
            # 1000000007 * 1000000009,
            # 123456789 * 987654321,
            # 999999937 * 999999929,
        ],
        'validate_enable': True,
        'workers': config.num_workers_per_node,
        'miller_rabin_rounds': 5,
        'max_attempts': 1000,
    }
    
    # 加载配置文件
    if args.config_file and os.path.exists(args.config_file):
        try:
            with open(args.config_file, 'r', encoding='utf-8') as f:
                test_config.update(json.load(f))
            print(f"📋 已加载配置文件: {args.config_file}")
        except Exception as e:
            print(f"⚠️  加载配置文件失败: {e}")
    
    try:
        # 创建测试套件
        test_suite = ClusterPrimeTestSuite(config)
        
        # 初始化集群
        test_suite.initialize_cluster()
        
        # 运行测试
        test_suite.run_cluster_test(test_config)
        
    except KeyboardInterrupt:
        print("\n⚠️  程序被用户中断")
    except Exception as e:
        print(f"❌ 程序运行出错: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # 清理资源
        if 'test_suite' in locals():
            test_suite.cleanup()

if __name__ == "__main__":
    main() 
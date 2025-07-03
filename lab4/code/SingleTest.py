#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
使用Ray框架的大素数寻找与分解测试程序
功能: 分布式大素数生成、测试和分解

本程序实现了以下核心功能：
1. 使用Ray分布式计算框架进行并行计算
2. Miller-Rabin素数测试算法进行素数检测
3. Pollard's Rho算法进行大数分解
4. 分布式素数生成和性能基准测试
5. 工作节点统计和任务监控

技术特点：
- 支持多机器集群计算
- 自动负载均衡
- 容错和恢复机制
- 实时性能监控
"""

# ==================== 导入必要的库 ====================
import ray  # Ray分布式计算框架
import time  # 时间测量
import random  # 随机数生成
import math  # 数学函数
from typing import List, Tuple, Optional, Union  # 类型注解
import multiprocessing  # 多进程支持
import stress_test_config as stc

# 导入素数验证模块
try:
    from prime_validator import PrimeValidator
    VALIDATOR_AVAILABLE = True
    print("✅ 素数验证模块可用")
except ImportError:
    VALIDATOR_AVAILABLE = False
    print("❌ 素数验证模块不可用")

# ==================== Ray初始化 ====================
# 初始化Ray分布式计算环境
# ignore_reinit_error=True 允许重复初始化，避免多次运行时的错误
ray.init(ignore_reinit_error=True)

# ==================== 素数工作节点类 ====================
@ray.remote  # Ray远程对象装饰器，使类可以在分布式环境中运行
class PrimeWorker:
    """
    素数工作节点类 - Ray分布式计算的核心组件
    
    这个类封装了所有素数相关的计算任务：
    - 素数测试（Miller-Rabin算法）
    - 素数生成
    - 大数分解（Pollard's Rho算法）
    - 统计信息收集
    
    每个PrimeWorker实例在Ray集群中作为一个独立的计算节点运行
    """
    
    def __init__(self, worker_id: int):
        """
        初始化工作节点
        
        Args:
            worker_id: 工作节点唯一标识符，用于区分不同的计算节点
        """
        self.worker_id = worker_id
        self.primes_found = 0  # 统计找到的素数数量
        self.factorizations_completed = 0  # 统计完成的分解任务数量
    
    def miller_rabin_test(self, n: int, k: int = 5) -> bool:
        """
        Miller-Rabin素数测试算法
        
        这是一个概率性素数测试算法，具有以下特点：
        - 时间复杂度：O(k * log³n)，其中k是测试轮数
        - 准确率：对于k=5，错误概率小于1/4^k
        - 适用于大数测试
        
        算法原理：
        1. 将n-1分解为d * 2^r的形式
        2. 随机选择a，计算a^d mod n
        3. 如果结果不为1或n-1，继续平方测试
        4. 重复k轮测试
        
        Args:
            n: 待测试的数
            k: 测试轮数，默认5轮，准确率约为99.9%
            
        Returns:
            True: n很可能是素数
            False: n一定是合数
        """
        # 处理边界情况
        if n <= 1:
            return False  # 1不是素数
        if n <= 3:
            return True   # 2和3是素数
        if n % 2 == 0:
            return False  # 偶数（除了2）不是素数
        
        # 将 n-1 写成 d * 2^r 的形式
        # 这是Miller-Rabin算法的关键步骤
        r = 0
        d = n - 1
        while d % 2 == 0:
            r += 1
            d //= 2
        
        # 进行k轮测试，提高准确率
        for _ in range(k):
            # 随机选择测试基数a，范围[2, n-1)
            a = random.randrange(2, n - 1)
            # 计算 a^d mod n
            x = pow(a, d, n)
            
            # 如果x=1或x=n-1，这轮测试通过
            if x == 1 or x == n - 1:
                continue
            
            # 否则，继续平方测试
            for _ in range(r - 1):
                x = (x * x) % n
                if x == n - 1:
                    break  # 找到n-1，这轮测试通过
            else:
                return False  # 所有平方测试都失败，n是合数
        
        return True  # 通过所有k轮测试，n很可能是素数
    
    def generate_random_odd_number(self, bits: int) -> int:
        """
        生成指定位数的随机奇数
        
        这个函数确保生成的数具有以下特性：
        - 精确的位数（最高位为1）
        - 奇数（最低位为1）
        - 随机性（中间位随机）
        
        Args:
            bits: 需要的位数
            
        Returns:
            指定位数的随机奇数
        """
        # 生成随机数
        n = random.getrandbits(bits)
        # 确保最高位为1（保证位数）
        n |= (1 << (bits - 1))
        # 确保最低位为1（奇数）
        n |= 1
        return n
    
    def find_prime(self, bits: int, max_attempts: int = 1000) -> Optional[int]:
        """
        寻找指定位数的素数
        
        算法策略：
        1. 生成随机奇数
        2. 使用Miller-Rabin测试
        3. 如果失败，继续尝试
        4. 设置最大尝试次数避免无限循环
        
        Args:
            bits: 素数位数
            max_attempts: 最大尝试次数，防止无限循环
            
        Returns:
            找到的素数，如果没找到返回None
        """
        attempts = 0
        while attempts < max_attempts:
            # 生成随机奇数
            n = self.generate_random_odd_number(bits)
            # 测试是否为素数
            if self.miller_rabin_test(n):
                self.primes_found += 1  # 更新统计
                return n
            attempts += 1
        return None  # 达到最大尝试次数仍未找到
    
    def pollard_rho_factorization(self, n: int, max_attempts: int = 10) -> List[int]:
        """
        Pollard's Rho算法进行大数分解
        
        算法原理：
        1. 使用函数f(x) = x² + 1 mod n生成序列
        2. 使用Floyd循环检测算法
        3. 当找到非平凡因子时递归分解
        
        时间复杂度：平均O(√p)，其中p是最小因子
        适用场景：分解有较小因子的合数
        
        Args:
            n: 待分解的数
            max_attempts: 最大尝试次数，防止无限递归
            
        Returns:
            因子列表（可能包含重复因子）
        """
        # 处理边界情况
        if n <= 1:
            return []
        if self.miller_rabin_test(n):
            return [n]  # n是素数
        
        # 如果达到最大尝试次数，返回原数（表示无法分解）
        if max_attempts <= 0:
            return [n]
        
        # 定义迭代函数 f(x) = x² + 1 mod n
        def f(x):
            return (x * x + 1) % n
        
        # 计算最大公约数
        def gcd(a, b):
            while b:
                a, b = b, a % b
            return a
        
        # Pollard's Rho算法核心
        x = random.randrange(2, n)  # 随机起始点
        y = x
        d = 1
        
        # Floyd循环检测
        while d == 1:
            x = f(x)      # 乌龟：每次走一步
            y = f(f(y))   # 兔子：每次走两步
            d = gcd(abs(x - y), n)  # 计算差值与原数的最大公约数
        
        # 如果d == n，说明没找到非平凡因子
        if d == n:
            # 尝试不同的起始值，减少最大尝试次数
            return self.pollard_rho_factorization(n, max_attempts - 1)
        
        # 递归分解两个因子
        factors = self.pollard_rho_factorization(d, max_attempts) + self.pollard_rho_factorization(n // d, max_attempts)
        self.factorizations_completed += 1  # 更新统计
        return factors
    
    def get_stats(self) -> dict:
        """
        获取工作节点统计信息
        
        用于监控和调试，了解每个节点的任务完成情况
        
        Returns:
            包含统计信息的字典
        """
        return {
            'worker_id': self.worker_id,
            'primes_found': self.primes_found,
            'factorizations_completed': self.factorizations_completed
        }

# ==================== 基准测试函数 ====================
@ray.remote  # 远程函数装饰器
def benchmark_prime_generation(bits: int, num_primes: int) -> dict:
    """
    基准测试：素数生成性能
    
    这个函数用于测量素数生成的性能，包括：
    - 生成时间
    - 成功率
    - 实际生成的素数数量
    
    Args:
        bits: 素数位数
        num_primes: 要生成的素数数量
        
    Returns:
        包含性能指标的字典
    """
    # 创建远程工作节点
    worker = PrimeWorker.remote(0)
    start_time = time.time()
    
    # 生成指定数量的素数
    primes = []
    for _ in range(num_primes):
        prime = ray.get(worker.find_prime.remote(bits))
        if prime:
            primes.append(prime)
    
    end_time = time.time()
    
    # 返回性能统计
    return {
        'bits': bits,
        'num_primes': len(primes),
        'time_taken': end_time - start_time,
        'primes': primes[:5]  # 只返回前5个素数作为示例
    }

@ray.remote  # 远程函数装饰器
def benchmark_factorization(numbers: List[int]) -> dict:
    """
    基准测试：大数分解性能
    
    测量Pollard's Rho算法的分解性能
    
    Args:
        numbers: 要分解的数列表
        
    Returns:
        包含分解结果的字典
    """
    # 创建远程工作节点
    worker = PrimeWorker.remote(0)
    start_time = time.time()
    
    # 分解所有数
    factorizations = []
    for num in numbers:
        factors = ray.get(worker.pollard_rho_factorization.remote(num))
        factorizations.append((num, factors))
    
    end_time = time.time()
    
    return {
        'num_numbers': len(numbers),
        'time_taken': end_time - start_time,
        'factorizations': factorizations
    }

# ==================== 测试套件类 ====================
class PrimeTestSuite:
    """
    素数测试套件 - 协调分布式计算的主要类
    
    这个类负责：
    1. 管理工作节点集群
    2. 分配计算任务
    3. 收集和汇总结果
    4. 运行综合测试
    
    设计模式：主从模式（Master-Worker）
    """
    
    def __init__(self, num_workers: Optional[int] = None):
        """
        初始化测试套件
        
        Args:
            num_workers: 工作节点数量，默认为CPU核心数
        """
        if num_workers is None:
            num_workers = multiprocessing.cpu_count()  # 自动检测CPU核心数
        self.num_workers = num_workers
        # 创建远程工作节点集群
        self.workers = [PrimeWorker.remote(i) for i in range(num_workers)]
        
        # 初始化素数验证器
        if VALIDATOR_AVAILABLE:
            self.validator = PrimeValidator()
        else:
            self.validator = None
    
    def find_primes_distributed(self, bits: int, num_primes: int) -> List[int]:
        """
        分布式寻找素数
        
        任务分配策略：
        1. 计算每个工作节点的任务数量
        2. 处理任务数量不能整除的情况
        3. 并行提交所有任务
        4. 收集和汇总结果
        
        Args:
            bits: 素数位数
            num_primes: 要生成的素数数量
            
        Returns:
            找到的素数列表
        """
        print(f"使用 {self.num_workers} 个工作节点寻找 {num_primes} 个 {bits} 位素数...")
        
        start_time = time.time()
        
        # 任务分配：确保负载均衡
        tasks_per_worker = num_primes // self.num_workers  # 每个节点的基本任务数
        remaining_tasks = num_primes % self.num_workers    # 剩余任务数
        
        futures = []  # 存储所有任务的future对象
        task_count = 0
        
        # 为每个工作节点分配任务
        for i, worker in enumerate(self.workers):
            # 前remaining_tasks个节点多分配一个任务
            worker_tasks = tasks_per_worker + (1 if i < remaining_tasks else 0)
            for _ in range(worker_tasks):
                future = worker.find_prime.remote(bits)  # 提交远程任务
                futures.append(future)
                task_count += 1
        
        # 收集所有结果
        primes = []
        for future in futures:
            prime = ray.get(future)  # 等待任务完成并获取结果
            if prime:
                primes.append(prime)
        
        end_time = time.time()
        
        print(f"找到 {len(primes)} 个素数，耗时 {end_time - start_time:.2f} 秒，每秒生成 {len(primes) / (end_time - start_time):.2f} 个素数")
        return primes
    
    def factorize_numbers_distributed(self, numbers: List[int]) -> List[Tuple[int, List[int]]]:
        """
        分布式大数分解
        
        使用轮询策略分配分解任务，确保负载均衡
        
        Args:
            numbers: 要分解的数列表
            
        Returns:
            分解结果列表，每个元素是(原数, 因子列表)的元组
        """
        print(f"使用 {self.num_workers} 个工作节点分解 {len(numbers)} 个数...")
        
        start_time = time.time()
        
        # 使用轮询策略分配任务
        futures = []
        for i, num in enumerate(numbers):
            worker = self.workers[i % self.num_workers]  # 轮询选择工作节点
            future = worker.pollard_rho_factorization.remote(num)
            futures.append(future)
        
        # 收集分解结果
        factorizations = []
        for i, future in enumerate(futures):
            factors = ray.get(future)
            factorizations.append((numbers[i], factors))
        
        end_time = time.time()
        
        print(f"完成 {len(factorizations)} 个数的分解，耗时 {end_time - start_time:.2f} 秒")
        return factorizations
    
    def validate_generated_primes(self, primes: List[int]) -> dict:
        """
        验证生成的素数
        
        Args:
            primes: 待验证的素数列表
            
        Returns:
            验证结果统计
        """
        if not self.validator or not primes:
            return {'validated': False, 'message': '验证器不可用或无素数'}
        
        print(f"\n开始验证 {len(primes)} 个生成的素数...")
        validation_start = time.time()
        
        # 使用sympy进行验证
        validation_results = self.validator.validate_primes_batch(primes, method="sympy")
        
        # 统计结果
        valid_primes = [r for r in validation_results if r['is_prime']]
        invalid_primes = [r for r in validation_results if not r['is_prime']]
        
        validation_time = time.time() - validation_start
        
        # 计算验证统计
        stats = {
            'validated': True,
            'total_primes': len(primes),
            'valid_primes': len(valid_primes),
            'invalid_primes': len(invalid_primes),
            'accuracy': len(valid_primes) / len(primes) * 100 if primes else 0,
            'validation_time': validation_time,
            'results': validation_results
        }
        
        # 显示验证结果
        print(f"验证完成，耗时 {validation_time:.4f} 秒")
        print(f"总素数: {stats['total_primes']}")
        print(f"验证为素数: {stats['valid_primes']} ✅")
        print(f"验证为合数: {stats['invalid_primes']} ❌")
        print(f"准确率: {stats['accuracy']:.2f}%")
        
        # 显示前几个验证结果
        # if validation_results:
        #     print(f"\n前5个验证结果:")
        #     for i, result in enumerate(validation_results[:5]):
        #         status = "✅ 素数" if result['is_prime'] else "❌ 合数"
        #         print(f"  {result['number']}: {status} (验证耗时: {result['time_taken']:.6f}s)")
        
        # 如果有错误的素数，显示详细信息
        if invalid_primes:
            print(f"\n⚠️  发现 {len(invalid_primes)} 个错误的素数:")
            for result in invalid_primes[:3]:  # 只显示前3个
                print(f"  {result['number']} (位数: {result['bits']})")
        
        return stats
    
    def run_single_test(self,config:dict):
        """
        运行单个测试
        
        Args:
            config: 测试配置
            {
                num_of_generate:int,
                generate_list:list[(bits:int,num_of_primes:int)],
                num_of_factorize:int,
                factorize_list:list[int],
                validate_enable:bool,
                workers:int,
                miller_rabin_rounds:int,
                max_attempts:int,
            }
        """
        num_of_generate = config['num_of_generate']
        generate_list = config['generate_list']
        num_of_factorize = config['num_of_factorize']
        factorize_list = config['factorize_list']
        validate_enable = config['validate_enable']
        
        for i in range(num_of_generate):
            bits, num_of_primes = generate_list[i]
            primes = self.find_primes_distributed(bits, num_of_primes)
            # print(f"生成的素数: {primes[:5]}...")
            
            if validate_enable:
                self.validate_generated_primes(primes)
        
        # 从generate_list产生num_of_factorize个大素数乘积，并分解
        # 注意：这里需要先收集所有生成的素数，然后计算乘积
        all_primes = []
        for i in range(num_of_generate):
            bits, num_of_primes = generate_list[i]
            primes = self.find_primes_distributed(bits, num_of_primes)
            all_primes.extend(primes)
        
        # 计算前几个素数的乘积进行分解测试
        for i in range(min(num_of_factorize, len(all_primes) // 2)):
            # 取两个素数相乘
            if i * 2 + 1 < len(all_primes):
                num = all_primes[i * 2] * all_primes[i * 2 + 1]
                factorizations = self.factorize_numbers_distributed([num])
                print(f"数 {num} 的因子: {factorizations}")

        for i in range(num_of_factorize):
            num = factorize_list[i]
            # 修复：将单个整数包装成列表
            factorizations = self.factorize_numbers_distributed([num])
            # 获取第一个（也是唯一的）分解结果
            if factorizations:
                num_result, factors = factorizations[0]
                print(f"数 {num_result} 的因子: {factors}")
            
        print("测试完成")

    def run_comprehensive_test(self):
        """
        运行综合测试
        
        这个函数执行完整的测试流程，包括：
        1. 不同位数的素数生成测试
        2. 大数分解测试
        3. 性能基准测试
        4. 工作节点统计
        
        测试设计遵循渐进式原则：从简单到复杂
        """
        print("=" * 60)
        print("Ray框架大素数寻找与分解测试程序")
        print("=" * 60)
        
        # 测试1: 小位数素数生成 (32位)
        # 32位素数适合快速测试，验证基本功能
        print("\n1. 测试小位数素数生成 (32位)")
        small_primes = self.find_primes_distributed(32, 10)
        print(f"生成的素数: {small_primes[:5]}...")
        
        # 验证生成的素数
        if VALIDATOR_AVAILABLE:
            small_validation = self.validate_generated_primes(small_primes)
        
        # 测试2: 中等位数素数生成 (64位)
        # 64位素数测试中等规模计算能力
        print("\n2. 测试中等位数素数生成 (64位)")
        medium_primes = self.find_primes_distributed(64, 5)
        print(f"生成的素数: {medium_primes[:3]}...")
        
        # 验证生成的素数
        if VALIDATOR_AVAILABLE:
            medium_validation = self.validate_generated_primes(medium_primes)
        
        # 测试3: 大数分解
        # 测试Pollard's Rho算法的有效性
        print("\n3. 测试大数分解")
        # 使用一些已知的合数进行分解测试
        composite_numbers = [
            1000000007 * 1000000009,  # 两个大素数的乘积
            123456789 * 987654321,    # 两个数的乘积
            999999937 * 999999929,    # 两个大素数的乘积
        ]
        
        factorizations = self.factorize_numbers_distributed(composite_numbers)
        for num, factors in factorizations:
            print(f"数 {num} 的因子: {factors}")
        
        # 测试4: 性能基准测试
        # 测量算法的实际性能表现
        print("\n4. 性能基准测试")
        
        # 素数生成基准测试
        prime_benchmark = ray.get(benchmark_prime_generation.remote(32, 20))
        print(f"生成 {prime_benchmark['num_primes']} 个 32 位素数耗时: {prime_benchmark['time_taken']:.2f} 秒")
        
        # 分解基准测试
        test_numbers = [1000000007 * 1000000009, 123456789 * 987654321]
        factor_benchmark = ray.get(benchmark_factorization.remote(test_numbers))
        print(f"分解 {factor_benchmark['num_numbers']} 个数耗时: {factor_benchmark['time_taken']:.2f} 秒")
        
        # 显示工作节点统计
        # 监控每个节点的任务完成情况，用于性能分析和调试
        print("\n5. 工作节点统计")
        for worker in self.workers:
            stats = ray.get(worker.get_stats.remote())
            print(f"工作节点 {stats['worker_id']}: 找到 {stats['primes_found']} 个素数, 完成 {stats['factorizations_completed']} 次分解")
        
        # 测试6: 素数验证总结
        if VALIDATOR_AVAILABLE:
            print("\n6. 素数验证总结")
            all_primes = small_primes + medium_primes
            if all_primes:
                overall_validation = self.validate_generated_primes(all_primes)
                print(f"总体准确率: {overall_validation['accuracy']:.2f}%")
                
                if overall_validation['accuracy'] >= 95:
                    print("✅ 素数生成算法表现优秀！")
                elif overall_validation['accuracy'] >= 80:
                    print("⚠️  素数生成算法表现良好，但有改进空间")
                else:
                    print("❌ 素数生成算法需要改进")

# ==================== 主函数 ====================
def main():
    """
    主函数 - 程序的入口点
    
    负责：
    1. 创建测试套件
    2. 运行综合测试
    3. 异常处理
    4. 资源清理
    """
    try:
        # 创建测试套件，使用4个工作节点
        # 可以根据系统性能调整节点数量
        test_suite = PrimeTestSuite(num_workers=4)
        
        # 运行综合测试
        test_suite.run_comprehensive_test()
        
        print("\n" + "=" * 60)
        print("测试完成！")
        print("=" * 60)
        
    except KeyboardInterrupt:
        # 处理用户中断（Ctrl+C）
        print("\n程序被用户中断")
    except Exception as e:
        # 处理其他异常
        print(f"程序运行出错: {e}")
    finally:
        # 清理Ray资源，确保程序正常退出
        ray.shutdown()

# ==================== 程序入口 ====================
if __name__ == "__main__":
    # 只有直接运行此文件时才执行main函数
    # 这允许文件被作为模块导入而不执行测试
    # main()
    # # 使用默认配置
    config = {
        'num_of_generate': 1,
        'generate_list': [(1024, 100)],
        'num_of_factorize': 0,
        'factorize_list': [1000000007 * 1000000009, 123456789 * 987654321],
        'validate_enable': True,
        'workers': 16,
        'miller_rabin_rounds': 5,
        'max_attempts': 1000,
    }
    # workers = 1
    config['workers'] = 1
    test_suite = PrimeTestSuite(num_workers=config['workers'])
    test_suite.run_single_test(config)


    # wokers = 16
    config['workers'] = 16
    test_suite = PrimeTestSuite(num_workers=config['workers'])
    test_suite.run_single_test(config)
    

#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
简化版大素数寻找与分解测试程序
功能: 单线程大素数生成、测试和分解，集成素数验证功能
"""

import time
import random
import math
from typing import List, Tuple, Optional
import multiprocessing
from concurrent.futures import ProcessPoolExecutor, as_completed

# 导入素数验证模块
try:
    from prime_validator import PrimeValidator
    VALIDATOR_AVAILABLE = True
    print("✅ 素数验证模块可用")
except ImportError:
    VALIDATOR_AVAILABLE = False
    print("❌ 素数验证模块不可用")

class PrimeWorker:
    """素数工作类"""
    
    def __init__(self, worker_id: int):
        self.worker_id = worker_id
        self.primes_found = 0
        self.factorizations_completed = 0
    
    def miller_rabin_test(self, n: int, k: int = 5) -> bool:
        """
        Miller-Rabin素数测试算法
        Args:
            n: 待测试的数
            k: 测试轮数
        Returns:
            True if n is probably prime, False otherwise
        """
        if n <= 1:
            return False
        if n <= 3:
            return True
        if n % 2 == 0:
            return False
        
        # 将 n-1 写成 d * 2^r 的形式
        r = 0
        d = n - 1
        while d % 2 == 0:
            r += 1
            d //= 2
        
        # 进行k轮测试
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
        Args:
            bits: 素数位数
            max_attempts: 最大尝试次数
        Returns:
            找到的素数，如果没找到返回None
        """
        attempts = 0
        while attempts < max_attempts:
            n = self.generate_random_odd_number(bits)
            if self.miller_rabin_test(n):
                self.primes_found += 1
                return n
            attempts += 1
        return None
    
    def pollard_rho_factorization(self, n: int) -> List[int]:
        """
        Pollard's Rho算法进行大数分解
        Args:
            n: 待分解的数
        Returns:
            因子列表
        """
        if n <= 1:
            return []
        if self.miller_rabin_test(n):
            return [n]
        
        def f(x):
            return (x * x + 1) % n
        
        def gcd(a, b):
            while b:
                a, b = b, a % b
            return a
        
        # Pollard's Rho算法
        x = random.randrange(2, n)
        y = x
        d = 1
        
        while d == 1:
            x = f(x)
            y = f(f(y))
            d = gcd(abs(x - y), n)
        
        if d == n:
            # 如果没找到非平凡因子，尝试不同的起始值
            return self.pollard_rho_factorization(n)
        
        # 递归分解因子
        factors = self.pollard_rho_factorization(d) + self.pollard_rho_factorization(n // d)
        self.factorizations_completed += 1
        return factors
    
    def get_stats(self) -> dict:
        """获取工作节点统计信息"""
        return {
            'worker_id': self.worker_id,
            'primes_found': self.primes_found,
            'factorizations_completed': self.factorizations_completed
        }

def find_prime_worker(args):
    """工作函数：寻找素数"""
    worker_id, bits, max_attempts = args
    worker = PrimeWorker(worker_id)
    return worker.find_prime(bits, max_attempts)

def factorize_worker(args):
    """工作函数：分解大数"""
    worker_id, number = args
    worker = PrimeWorker(worker_id)
    return (number, worker.pollard_rho_factorization(number))

class SimplePrimeTestSuite:
    """简化版素数测试套件"""
    
    def __init__(self, num_workers: Optional[int] = None):
        if num_workers is None:
            num_workers = multiprocessing.cpu_count()
        self.num_workers = num_workers
        self.workers = [PrimeWorker(i) for i in range(num_workers)]
        
        # 初始化素数验证器
        if VALIDATOR_AVAILABLE:
            self.validator = PrimeValidator()
        else:
            self.validator = None
    
    def find_primes_parallel(self, bits: int, num_primes: int) -> List[int]:
        """并行寻找素数"""
        print(f"使用 {self.num_workers} 个工作节点寻找 {num_primes} 个 {bits} 位素数...")
        
        start_time = time.time()
        
        # 准备任务参数
        tasks = [(i % self.num_workers, bits, 1000) for i in range(num_primes)]
        
        # 使用进程池并行执行
        primes = []
        with ProcessPoolExecutor(max_workers=self.num_workers) as executor:
            futures = [executor.submit(find_prime_worker, task) for task in tasks]
            
            for future in as_completed(futures):
                prime = future.result()
                if prime:
                    primes.append(prime)
        
        end_time = time.time()
        
        print(f"找到 {len(primes)} 个素数，耗时 {end_time - start_time:.2f} 秒")
        return primes
    
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
        if validation_results:
            print(f"\n前5个验证结果:")
            for i, result in enumerate(validation_results[:5]):
                status = "✅ 素数" if result['is_prime'] else "❌ 合数"
                print(f"  {result['number']}: {status} (验证耗时: {result['time_taken']:.6f}s)")
        
        # 如果有错误的素数，显示详细信息
        if invalid_primes:
            print(f"\n⚠️  发现 {len(invalid_primes)} 个错误的素数:")
            for result in invalid_primes[:3]:  # 只显示前3个
                print(f"  {result['number']} (位数: {result['bits']})")
        
        return stats
    
    def factorize_numbers_parallel(self, numbers: List[int]) -> List[Tuple[int, List[int]]]:
        """并行大数分解"""
        print(f"使用 {self.num_workers} 个工作节点分解 {len(numbers)} 个数...")
        
        start_time = time.time()
        
        # 准备任务参数
        tasks = [(i % self.num_workers, num) for i, num in enumerate(numbers)]
        
        # 使用进程池并行执行
        factorizations = []
        with ProcessPoolExecutor(max_workers=self.num_workers) as executor:
            futures = [executor.submit(factorize_worker, task) for task in tasks]
            
            for future in as_completed(futures):
                result = future.result()
                factorizations.append(result)
        
        end_time = time.time()
        
        print(f"完成 {len(factorizations)} 个数的分解，耗时 {end_time - start_time:.2f} 秒")
        return factorizations
    
    def benchmark_prime_generation(self, bits: int, num_primes: int) -> dict:
        """基准测试：素数生成性能"""
        worker = PrimeWorker(0)
        start_time = time.time()
        
        primes = []
        for _ in range(num_primes):
            prime = worker.find_prime(bits)
            if prime:
                primes.append(prime)
        
        end_time = time.time()
        
        return {
            'bits': bits,
            'num_primes': len(primes),
            'time_taken': end_time - start_time,
            'primes': primes[:5]  # 只返回前5个素数作为示例
        }
    
    def benchmark_factorization(self, numbers: List[int]) -> dict:
        """基准测试：大数分解性能"""
        worker = PrimeWorker(0)
        start_time = time.time()
        
        factorizations = []
        for num in numbers:
            factors = worker.pollard_rho_factorization(num)
            factorizations.append((num, factors))
        
        end_time = time.time()
        
        return {
            'num_numbers': len(numbers),
            'time_taken': end_time - start_time,
            'factorizations': factorizations
        }
    
    def run_comprehensive_test(self):
        """运行综合测试"""
        print("=" * 60)
        print("简化版大素数寻找与分解测试程序")
        print("=" * 60)
        
        # 测试1: 小位数素数生成 (32位)
        print("\n1. 测试小位数素数生成 (32位)")
        small_primes = self.find_primes_parallel(32, 10)
        print(f"生成的素数: {small_primes[:5]}...")
        
        # 验证生成的素数
        if VALIDATOR_AVAILABLE:
            small_validation = self.validate_generated_primes(small_primes)
        
        # 测试2: 中等位数素数生成 (64位)
        print("\n2. 测试中等位数素数生成 (64位)")
        medium_primes = self.find_primes_parallel(64, 5)
        print(f"生成的素数: {medium_primes[:3]}...")
        
        # 验证生成的素数
        if VALIDATOR_AVAILABLE:
            medium_validation = self.validate_generated_primes(medium_primes)
        
        # 测试3: 大数分解
        print("\n3. 测试大数分解")
        # 使用一些合数进行分解测试
        composite_numbers = [
            1000000007 * 1000000009,  # 两个大素数的乘积
            123456789 * 987654321,    # 两个数的乘积
            999999937 * 999999929,    # 两个大素数的乘积
        ]
        
        factorizations = self.factorize_numbers_parallel(composite_numbers)
        for num, factors in factorizations:
            print(f"数 {num} 的因子: {factors}")
        
        # 测试4: 性能基准测试
        print("\n4. 性能基准测试")
        
        # 素数生成基准测试
        prime_benchmark = self.benchmark_prime_generation(32, 20)
        print(f"生成 {prime_benchmark['num_primes']} 个 32 位素数耗时: {prime_benchmark['time_taken']:.2f} 秒")
        
        # 分解基准测试
        test_numbers = [1000000007 * 1000000009, 123456789 * 987654321]
        factor_benchmark = self.benchmark_factorization(test_numbers)
        print(f"分解 {factor_benchmark['num_numbers']} 个数耗时: {factor_benchmark['time_taken']:.2f} 秒")
        
        # 显示工作节点统计
        print("\n5. 工作节点统计")
        for worker in self.workers:
            stats = worker.get_stats()
            print(f"工作节点 {stats['worker_id']}: 找到 {stats['primes_found']} 个素数, 完成 {stats['factorizations_completed']} 次分解")
        
        # 测试5: 素数验证总结
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

def main():
    """主函数"""
    try:
        # 创建测试套件
        test_suite = SimplePrimeTestSuite(num_workers=4)
        
        # 运行综合测试
        test_suite.run_comprehensive_test()
        
        print("\n" + "=" * 60)
        print("测试完成！")
        print("=" * 60)
        
    except KeyboardInterrupt:
        print("\n程序被用户中断")
    except Exception as e:
        print(f"程序运行出错: {e}")

if __name__ == "__main__":
    main() 
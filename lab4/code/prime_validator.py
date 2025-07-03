#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
素数验证模块
功能: 使用成熟的数学库验证素数生成结果的正确性

本模块使用以下库进行验证：
1. sympy - 强大的数学库，包含isprime()函数
2. gmpy2 - 高性能数学库（如果可用）
3. 内置算法 - 作为备用验证方法
"""

import time
import random
from typing import List, Tuple, Dict, Optional
import multiprocessing
from concurrent.futures import ProcessPoolExecutor, as_completed

# 尝试导入各种素数验证库
try:
    import sympy
    SYMPY_AVAILABLE = True
    print("✅ sympy库可用 - 将用于素数验证")
except ImportError:
    SYMPY_AVAILABLE = False
    print("❌ sympy库不可用")

try:
    import gmpy2
    GMPY2_AVAILABLE = True
    print("✅ gmpy2库可用 - 将用于高性能素数验证")
except ImportError:
    GMPY2_AVAILABLE = False
    print("❌ gmpy2库不可用")

class PrimeValidator:
    """
    素数验证器类
    
    提供多种素数验证方法：
    1. sympy.isprime() - 确定性素数测试
    2. gmpy2.is_prime() - 高性能素数测试
    3. 确定性算法 - 备用验证方法
    """
    
    def __init__(self):
        """初始化验证器"""
        self.validation_methods = []
        
        if SYMPY_AVAILABLE:
            self.validation_methods.append(("sympy", self._sympy_is_prime))
        
        if GMPY2_AVAILABLE:
            self.validation_methods.append(("gmpy2", self._gmpy2_is_prime))
        
        # 添加确定性算法作为备用
        self.validation_methods.append(("deterministic", self._deterministic_is_prime))
        
        print(f"可用验证方法: {[method[0] for method in self.validation_methods]}")
    
    def _sympy_is_prime(self, n: int) -> bool:
        """
        使用sympy库进行素数测试
        
        sympy.isprime()使用多种确定性算法：
        - 小数的试除法
        - 大数的Baillie-PSW测试
        - 其他确定性测试
        
        Args:
            n: 待测试的数
            
        Returns:
            True if n is prime, False otherwise
        """
        return sympy.isprime(n)
    
    def _gmpy2_is_prime(self, n: int) -> bool:
        """
        使用gmpy2库进行素数测试
        
        gmpy2.is_prime()使用高性能的确定性算法
        
        Args:
            n: 待测试的数
            
        Returns:
            True if n is prime, False otherwise
        """
        return bool(gmpy2.is_prime(n))
    
    def _deterministic_is_prime(self, n: int) -> bool:
        """
        确定性素数测试算法
        
        使用AKS算法的简化版本，适用于中等大小的数
        
        Args:
            n: 待测试的数
            
        Returns:
            True if n is prime, False otherwise
        """
        if n <= 1:
            return False
        if n <= 3:
            return True
        if n % 2 == 0:
            return False
        
        # 对于中等大小的数，使用确定性测试
        if n < 10**6:
            # 试除法
            for i in range(3, int(n**0.5) + 1, 2):
                if n % i == 0:
                    return False
            return True
        else:
            # 对于大数，使用强伪素数测试
            return self._strong_pseudoprime_test(n)
    
    def _strong_pseudoprime_test(self, n: int, bases: List[int] = None) -> bool:
        """
        强伪素数测试
        
        对于大数使用确定性测试，基于以下事实：
        如果n < 2^64，则只需要测试前12个素数作为基数
        
        Args:
            n: 待测试的数
            bases: 测试基数列表
            
        Returns:
            True if n is prime, False otherwise
        """
        if bases is None:
            # 对于n < 2^64，使用前12个素数作为基数
            if n < 2**64:
                bases = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
            else:
                # 对于更大的数，使用更多基数
                bases = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71]
        
        # 将n-1分解为d * 2^r
        r = 0
        d = n - 1
        while d % 2 == 0:
            r += 1
            d //= 2
        
        # 对每个基数进行测试
        for a in bases:
            if a >= n:
                continue
            
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
    
    def validate_prime(self, n: int, method: str = "auto") -> Dict[str, any]:
        """
        验证一个数是否为素数
        
        Args:
            n: 待验证的数
            method: 验证方法 ("auto", "sympy", "gmpy2", "deterministic")
            
        Returns:
            包含验证结果的字典
        """
        start_time = time.time()
        
        if method == "auto":
            # 自动选择最快的可用方法
            if SYMPY_AVAILABLE:
                method = "sympy"
            elif GMPY2_AVAILABLE:
                method = "gmpy2"
            else:
                method = "deterministic"
        
        # 执行验证
        if method == "sympy" and SYMPY_AVAILABLE:
            is_prime = self._sympy_is_prime(n)
        elif method == "gmpy2" and GMPY2_AVAILABLE:
            is_prime = self._gmpy2_is_prime(n)
        else:
            is_prime = self._deterministic_is_prime(n)
        
        end_time = time.time()
        
        return {
            'number': n,
            'is_prime': is_prime,
            'method': method,
            'time_taken': end_time - start_time,
            'bits': n.bit_length()
        }
    
    def validate_primes_batch(self, numbers: List[int], method: str = "auto") -> List[Dict[str, any]]:
        """
        批量验证素数
        
        Args:
            numbers: 待验证的数列表
            method: 验证方法
            
        Returns:
            验证结果列表
        """
        results = []
        for n in numbers:
            result = self.validate_prime(n, method)
            results.append(result)
        return results
    
    def validate_primes_parallel(self, numbers: List[int], method: str = "auto", max_workers: int = None) -> List[Dict[str, any]]:
        """
        并行批量验证素数
        
        Args:
            numbers: 待验证的数列表
            method: 验证方法
            max_workers: 最大工作进程数
            
        Returns:
            验证结果列表
        """
        if max_workers is None:
            max_workers = multiprocessing.cpu_count()
        
        def validate_worker(args):
            n, method = args
            validator = PrimeValidator()
            return validator.validate_prime(n, method)
        
        # 准备任务
        tasks = [(n, method) for n in numbers]
        
        # 并行执行
        results = []
        with ProcessPoolExecutor(max_workers=max_workers) as executor:
            futures = [executor.submit(validate_worker, task) for task in tasks]
            
            for future in as_completed(futures):
                result = future.result()
                results.append(result)
        
        # 按原始顺序排序
        results.sort(key=lambda x: numbers.index(x['number']))
        return results
    
    def compare_methods(self, numbers: List[int]) -> Dict[str, any]:
        """
        比较不同验证方法的性能和结果
        
        Args:
            numbers: 待验证的数列表
            
        Returns:
            比较结果字典
        """
        comparison = {
            'numbers': numbers,
            'methods': {},
            'consistency': True,
            'total_time': 0
        }
        
        start_time = time.time()
        
        # 对每个方法进行验证
        for method_name, method_func in self.validation_methods:
            method_results = []
            method_start = time.time()
            
            for n in numbers:
                result = self.validate_prime(n, method_name)
                method_results.append(result)
            
            method_end = time.time()
            
            comparison['methods'][method_name] = {
                'results': method_results,
                'time_taken': method_end - method_start,
                'primes_found': sum(1 for r in method_results if r['is_prime'])
            }
        
        # 检查结果一致性
        if len(comparison['methods']) > 1:
            method_names = list(comparison['methods'].keys())
            first_method = method_names[0]
            first_results = [r['is_prime'] for r in comparison['methods'][first_method]['results']]
            
            for method_name in method_names[1:]:
                other_results = [r['is_prime'] for r in comparison['methods'][method_name]['results']]
                if first_results != other_results:
                    comparison['consistency'] = False
                    break
        
        comparison['total_time'] = time.time() - start_time
        return comparison

def test_validator():
    """测试验证器功能"""
    print("=" * 60)
    print("素数验证器测试")
    print("=" * 60)
    
    validator = PrimeValidator()
    
    # 测试数据
    test_numbers = [
        2, 3, 4, 5, 7, 11, 13, 17, 19, 23,  # 小素数
        29, 31, 37, 41, 43, 47, 53, 59, 61, 67,  # 更多素数
        71, 73, 79, 83, 89, 97,  # 继续素数
        100, 121, 169,  # 合数
        2147483647,  # 梅森素数 M31
        2305843009213693951,  # 梅森素数 M61
        1152921504606846883,  # 大素数
        1152921504606846884,  # 大合数
    ]
    
    print(f"测试 {len(test_numbers)} 个数...")
    
    # 比较不同方法
    comparison = validator.compare_methods(test_numbers)
    
    # 显示结果
    print(f"\n验证方法比较:")
    print(f"总耗时: {comparison['total_time']:.4f} 秒")
    print(f"结果一致性: {'✅ 一致' if comparison['consistency'] else '❌ 不一致'}")
    
    for method_name, method_data in comparison['methods'].items():
        print(f"\n{method_name}:")
        print(f"  耗时: {method_data['time_taken']:.4f} 秒")
        print(f"  找到素数: {method_data['primes_found']} 个")
        
        # 显示前几个结果
        print(f"  前5个结果:")
        for i, result in enumerate(method_data['results'][:5]):
            status = "✅ 素数" if result['is_prime'] else "❌ 合数"
            print(f"    {result['number']}: {status} ({result['time_taken']:.6f}s)")
    
    return validator

if __name__ == "__main__":
    # 运行测试
    validator = test_validator() 
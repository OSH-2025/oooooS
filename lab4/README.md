# lab4

> 实验报告在[Lab4/report.md](/lab4/report.md)中！
> 以下仅为实验代码和配置文件的简要说明。

## Ray集群大素数寻找与分解测试程序

源代码在lab4/code目录下，包含以下文件：'

- `ClusterPrimeTest.py`: 集群分布式计算程序，支持多节点协同工作。
- `SimplePrimeTest.py`: 简化版单线程大素数生成与分解测试程序，集成素数验证功能。
- `SingleTest.py`: 单机版大素数寻找与分解测试程序，使用Ray框架实现分布式计算。
- `prime_validator.py`: 素数验证模块，使用成熟的数学库验证素
- `DataAnalysis/graph_drawer.py`: 数据分析模块，处理测试结果并生成图表。

- `run_cluster_tests.sh`: 集群运行脚本，自动启动Ray集群并执行测试任务。


## Docker集群配置和运行指南

进入lab4/docker目录后，执行以下命令：

构建方法： 

``` bash
docker build -t raytest .
```

运行方法： 

运行工作节点：

``` bash
docker run -d --name ray-worker raytest ray start --address="ray-head:8000" --redis-password="1234"
```

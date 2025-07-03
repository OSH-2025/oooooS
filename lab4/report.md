# Ray分布式计算框架实验报告

- [Ray分布式计算框架实验报告](#ray分布式计算框架实验报告)
  - [一、实验背景与目的](#一实验背景与目的)
  - [二、可选性能指标](#二可选性能指标)
    - [（一）核心性能指标](#一核心性能指标)
    - [（二）资源效率指标](#二资源效率指标)
    - [（三）系统级稳定性指标](#三系统级稳定性指标)
  - [三、选定测试任务及测试指标](#三选定测试任务及测试指标)
    - [（一）选定测试任务](#一选定测试任务)
    - [（二）测试指标](#二测试指标)
  - [四、环境配置](#四环境配置)
    - [1. 更新系统包](#1-更新系统包)
    - [2. 安装依赖工具](#2-安装依赖工具)
    - [3. 添加Python版本源并安装](#3-添加python版本源并安装)
    - [4. 设置默认Python版本](#4-设置默认python版本)
    - [5. 安装Python虚拟环境工具](#5-安装python虚拟环境工具)
    - [6. 创建项目目录并初始化虚拟环境](#6-创建项目目录并初始化虚拟环境)
    - [7. 升级pip并安装Ray](#7-升级pip并安装ray)
    - [8. 退出虚拟环境](#8-退出虚拟环境)
  - [五、单机版部署与性能测试](#五单机版部署与性能测试)
    - [（一）部署过程](#一部署过程)
    - [（二）性能测试](#二性能测试)
    - [（三）测试结果](#三测试结果)
  - [六、分布式部署与性能测试](#六分布式部署与性能测试)
    - [（一）部署方式](#一部署方式)
    - [（二）性能测试方法](#二性能测试方法)
    - [（三）测试结果](#三测试结果-1)
  - [七、基于Docker的分布式部署与性能测试](#七基于docker的分布式部署与性能测试)
    - [（一）Docker部署的优势](#一docker部署的优势)
    - [（二）Docker部署过程](#二docker部署过程)
    - [（三）测试结果](#三测试结果-2)
  - [八、实验报告发布](#八实验报告发布)


## 一、实验背景与目的

在大数据与人工智能快速发展的当下，分布式计算框架成为高效处理海量数据与复杂计算任务的关键技术。Ray作为新兴的分布式计算框架，凭借其出色的灵活性与扩展性，在多个领域大放异彩。本次实验聚焦于Ray框架，旨在深入探究其性能表现，掌握部署技巧，并通过优化提升性能。

大素数寻找与分解任务对计算资源要求极高，涉及大量复杂运算。选择该任务作为测试对象，可以充分利用Ray框架的分布式计算优势，实现任务的高效并行处理，从而显著提升计算效率。

## 二、可选性能指标

### （一）核心性能指标
1. **任务延迟（Task Latency）**
    - **定义**：单个Ray任务从提交到完成的时间（毫秒级）。
    - **合理性**：直接反映Ray任务调度的效率，尤其对实时推理场景（如LLM服务）至关重要。高延迟可能暴露任务分解或负载均衡问题。
2. **吞吐量（Throughput）**
    - **定义**：单位时间内成功处理的任务数量（任务/秒）。
    - **合理性**：衡量Ray分布式计算的并行能力，适用于批处理场景（如大规模方差计算）。低吞吐量可能提示任务粒度不合理或资源不足。

### （二）资源效率指标
1. **CPU/GPU利用率**
    - **定义**：计算节点中CPU/GPU核心的忙闲比例（%）。
    - **合理性**：低利用率可能因任务粒度过小或线程竞争（如NumPy/PyTorch多线程冲突），需结合任务调度策略优化。
2. **内存使用率（Memory Utilization）**
    - **定义**：节点内存及Ray对象存储的占用情况（GB）。
    - **合理性**：大模型推理易出现内存泄漏或显存溢出，监控可预防OOM错误，并优化批处理大小。

### （三）系统级稳定性指标
1. **错误率（Error Rate）**
    - **定义**：任务失败或异常的比例（%）。
    - **合理性**：高错误率可能因节点通信故障或资源竞争，高错误率可能因节点通信故障或资源竞争，影响系统稳定性。
2. **队列长度（Queue Length）**
    - **定义**：待处理任务的积压数量。
    - **合理性**：长队列暴露任务分配不均或资源瓶颈，需动态调整Actor数量或启用自动扩缩容。

## 三、选定测试任务及测试指标

### （一）选定测试任务
选定的大素数寻找与分解任务，具有高度的计算密集型特点，涉及复杂的数学运算与大量的数据处理。在该任务中，需要寻找大素数并对其进行分解，这不仅考验单个节点的计算能力，更对分布式计算框架的任务分配与并行处理机制提出了严峻挑战。

### （二）测试指标
1. 吞吐量：每秒处理的素数数量，直接体现Ray框架在单位时间内完成任务的效率，反映其并行计算能力。
2. 资源（CPU/Memory）利用率： CPU利用率衡量计算节点中CPU核心的忙碌程度，Memory利用率反映节点内存及Ray对象存储的占用状况。合理利用资源可以提升计算效率，过低或过高的利用率都可能影响任务执行效果。
3. 节点利用效率：“实际工作时间”除以“节点存活总时间”，衡量节点在任务执行中的实际贡献。高利用效率表明节点资源被充分利用，低利用效率可能提示任务分配不均或资源浪费。

## 四、环境配置

### 1. 更新系统包
```bash
sudo apt update
sudo apt upgrade
```

### 2. 安装依赖工具
```bash
sudo apt install software-properties-common
```

### 3. 添加Python版本源并安装
```bash
sudo add-apt-repository ppa:deadsnakes/ppa
sudo apt install python3.12
```

### 4. 设置默认Python版本
```bash
sudo update-alternatives --install /usr/bin/python python /usr/bin/python3.12 2
python --version  # 验证Python版本
```

### 5. 安装Python虚拟环境工具
```bash
sudo apt install python3.12-venv
```

### 6. 创建项目目录并初始化虚拟环境
```bash
mkdir raytest
cd raytest/
python3 -m venv rayenv
source rayenv/bin/activate  # 激活虚拟环境
```

### 7. 升级pip并安装Ray
```bash
pip install --upgrade pip
pip install "ray[default]"  # 安装Ray框架
pip install gmpy2  # 安装数学计算库
```

### 8. 退出虚拟环境
```bash
deactivate
```



## 五、单机版部署与性能测试

### （一）部署过程

见上述的环境配置部分，单机版部署与配置过程与分布式部署类似，只需在单台机器上启动Ray服务即可。

### （二）性能测试
1. 吞吐量测试：设计测试用例，模拟大素数寻找与分解任务。记录单位时间内完成的任务数量，计算吞吐量。在测试过程中，观察不同任务规模下吞吐量的变化情况，分析Ray框架的任务调度机制对吞吐量的影响。

    ```python
    @ray.remote
    def find_prime(bits):
        # 素数生成逻辑
        pass

    primes = ray.get([find_prime.remote(1024) for _ in range(100)])
    ```

2. 资源利用率测试：使用系统监控工具或Ray自带的资源监控功能，实时监测CPU和内存的使用情况。在任务执行过程中，记录CPU利用率和内存利用率的变化曲线，分析资源利用的高峰期与低谷期。

    ```python
    import psutil

    cpu_percent = psutil.cpu_percent(interval=1)
    memory_percent = psutil.virtual_memory().percent
    ```

3. 节点利用效率测试：计算每个节点的实际工作时间与存活总时间，评估节点在任务执行中的实际贡献。

    ```python
    node_uptime = ray.nodes()[0]['uptime']  # 获取节点存活时间
    node_work_time = ray.get_runtime_context().get_node_work_time()  # 获取实际工作时间
    efficiency = node_work_time / node_uptime if node_uptime > 0 else 0
    ```

### （三）测试结果

在单机版测试中，我们修改单节点的Worker数量,测试得到以下数据。具体测试结果如下：

CPU利用率随着Worker数量的增加而逐渐上升，但在Worker数量达到一定程度后，CPU利用率趋于平稳。
![img](/lab4/img/cluster_size_1/avg_cpu_vs_total_workers.png)

内存使用量随着Worker数量的增加而逐渐上升。
![img](/lab4/img/cluster_size_1/avg_memory_vs_total_workers.png)

吞吐量随着Worker数量的增加而显著提升，表明Ray框架能够有效利用多核CPU进行并行计算。

然而，由于我们使用的虚拟机CPU仅为2核，吞吐量在Worker数量达到2时就达到了峰值，之后由于资源竞争与调度开销的增加，吞吐量略有下降。
![img](/lab4/img/cluster_size_1/throughput_vs_total_workers.png)

显然，与Worker = 1相比，worker数量为2时，吞吐量提升了约一倍，达到了实验所需的优化要求。

节点利用效率随着Worker数量的增加而逐渐下降，可能是由于任务调度与资源分配的开销增加所致。
![img](/lab4/img/cluster_size_1/efficiency_vs_total_workers.png)

## 六、分布式部署与性能测试

### （一）部署方式
分布式部署采用多台服务器构建Ray集群。

目前使用的节点均为USTC vlab平台提供的虚拟机，配置如下：

- 每台服务器配置2核CPU，4GB内存，操作系统为Ubuntu 24.04。
- 每台服务器安装Ray框架，确保版本一致性。
- 多台服务器通过局域网连接，形成Ray集群，实现分布式计算。

在每台服务器上安装Ray，并配置集群参数以实现节点之间的通信与协作。将一台服务器设置为主节点，负责任务调度与资源管理，其他服务器作为工作节点，承担实际的计算任务。通过合理分配任务，充分发挥各节点的计算能力。

主节点上，运行以下命令启动Ray服务，并指定主节点IP地址：

```bash
ray start --head --node-ip-address=主节点IP --port=10001
```

工作节点上，运行以下命令连接到主节点：

```bash
ray start --address='ray://主节点IP:10001'
```

值得一提的是，在实验初期，我们也曾尝试过使用实体机（我们几个组员的个人电脑）进行分布式部署，但由于网络环境不稳定，加上互联的方式较为复杂，导致部署过程频繁出错，最终决定使用USTC vlab平台提供的虚拟机进行分布式部署。

### （二）性能测试方法

性能测试延续单机版测试内容，包括任务延迟、吞吐量、资源利用率等指标。此外，重点关注分布式环境下任务在各节点之间的分配情况与数据传输效率。


### （三）测试结果

在每台机器上启动同样数目的工作节点，并分配相同数量的任务。通过监控各节点的CPU与内存使用情况，记录平均CPU利用率与内存占用。

首先，平均CPU利用率随着集群规模的扩大而波动，但总体规律为多台机器协同工作时，CPU利用率较单机低，可能的原因是任务调度、资源分配与节点间通信的开销。
![img](/lab4/img/config_workers_4/avg_cpu_vs_cluster_size.png)

其次，平均内存使用量随着集群规模的扩大而增加，但增长幅度相对平稳，表明Ray在分布式环境下能够较好地管理内存资源，避免过度消耗。
(而图片中出现这样的趋势是因为在测集群规模2，3，5前重启了机器，导致内存使用量较低。 应该把1，4，6看成一条线，2，3，5看成另一条线。)
![img](/lab4/img/config_workers_4/avg_memory_vs_cluster_size.png)

吞吐量上，随着集群规模的扩大，Ray每秒处理的素数数量显著增加，表明分布式部署能够有效提升计算效率。测试结果显示，在集群规模为5时，吞吐量达到了最高峰。但在集群规模为6时，吞吐量略有下降，可能是由于节点间通信开销增加或任务调度不均衡所致。
![img](/lab4/img/config_workers_2/throughput_vs_cluster_size.png)

效率方面，随着集群规模的扩大，节点利用效率整体呈现明显的下降趋势。并且每个节点的worker数量增加时，下降趋势更加明显。这表明在分布式环境下，Ray的任务调度与资源分配机制可能存在一定的瓶颈，导致节点间的协同效率降低。需要进一步优化任务分配策略，以提升整体计算效率。
![img](/lab4/img/config_workers_1/efficiency_vs_cluster_size.png)

![img](/lab4/img/config_workers_3/efficiency_vs_cluster_size.png)

![img](/lab4/img/config_workers_6/efficiency_vs_cluster_size.png)

## 七、基于Docker的分布式部署与性能测试

### （一）Docker部署的优势
Docker容器化技术为分布式部署提供了诸多便利。它能够将Ray应用及其依赖环境打包成一个独立的容器镜像，确保在不同服务器上的一致性。容器之间相互隔离，避免环境冲突与依赖问题，便于扩展与维护。在分布式部署中，利用Docker可以快速创建与销毁容器，灵活调整计算资源规模。

### （二）Docker部署过程
1. 构建Docker镜像：编写Dockerfile，指定基础镜像、安装Ray及相关依赖库，并配置运行环境。将应用代码与配置文件添加到镜像中，定义容器启动时执行的命令。

具体的Dockerfile在[`lab4/docker/Dockerfile`](/lab4/docker/Dockerfile)。

```dockerfile
# 使用Ubuntu 24.04作为基础镜像
FROM ubuntu:24.04

# 构建参数 - 可以在构建时选择镜像源
ARG APT_MIRROR=tuna
ARG PIP_MIRROR=tuna

# 设置环境变量避免交互式提示
ENV DEBIAN_FRONTEND=noninteractive

# 首先更新包列表并安装证书相关包
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    apt-transport-https \
    software-properties-common \
    gpg-agent \
    curl \
    wget && \
    apt-get clean

# 配置国内APT镜像源 (使用HTTP避免证书问题)
RUN case "$APT_MIRROR" in \
    "tuna") \
        echo "" && \
        sed -i 's|http://archive.ubuntu.com/ubuntu/|http://mirrors.tuna.tsinghua.edu.cn/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources && \
        sed -i 's|http://security.ubuntu.com/ubuntu/|http://mirrors.tuna.tsinghua.edu.cn/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources ;; \
    "aliyun") \
        echo "" && \
        sed -i 's|http://archive.ubuntu.com/ubuntu/|http://mirrors.aliyun.com/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources && \
        sed -i 's|http://security.ubuntu.com/ubuntu/|http://mirrors.aliyun.com/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources ;; \
    "ustc") \
        echo "" && \
        sed -i 's|http://archive.ubuntu.com/ubuntu/|http://mirrors.ustc.edu.cn/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources && \
        sed -i 's|http://security.ubuntu.com/ubuntu/|http://mirrors.ustc.edu.cn/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources ;; \
    "163") \
        echo "" && \
        sed -i 's|http://archive.ubuntu.com/ubuntu/|http://mirrors.163.com/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources && \
        sed -i 's|http://security.ubuntu.com/ubuntu/|http://mirrors.163.com/ubuntu/|g' /etc/apt/sources.list.d/ubuntu.sources ;; \
    *) \
        echo "" ;; \
    esac

# 更新软件源
RUN apt-get update

# 添加deadsnakes PPA以获取Python3.9
RUN add-apt-repository ppa:deadsnakes/ppa -y && \
    apt-get update

# 安装核心依赖和指定版本的Python
RUN apt-get install -y --no-install-recommends \
    python3.9 \
    python3.9-dev \
    python3.9-venv \
    python3.9-distutils \
    python3-pip \
    net-tools \
    redis-tools && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# 设置Python版本
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3.9 1 && \
    update-alternatives --set python /usr/bin/python3.9

# 确保pip指向Python3.9
# RUN python3.9 -m pip install --upgrade pip --break-system-packages

# 创建非root用户并设置Ray运行环境
RUN useradd -m -s /bin/bash rayuser && \
    mkdir -p /home/rayuser/.pip && \
    chown -R rayuser:rayuser /home/rayuser

# 切换到非root用户
USER rayuser
WORKDIR /home/rayuser/raytest

# 复制pip配置到用户目录
RUN case "$PIP_MIRROR" in \
    "tuna") \
        echo "[global]" > /home/rayuser/.pip/pip.conf && \
        echo "index-url = https://pypi.tuna.tsinghua.edu.cn/simple" >> /home/rayuser/.pip/pip.conf && \
        echo "trusted-host = pypi.tuna.tsinghua.edu.cn" >> /home/rayuser/.pip/pip.conf ;; \
    "aliyun") \
        echo "[global]" > /home/rayuser/.pip/pip.conf && \
        echo "index-url = https://mirrors.aliyun.com/pypi/simple/" >> /home/rayuser/.pip/pip.conf && \
        echo "trusted-host = mirrors.aliyun.com" >> /home/rayuser/.pip/pip.conf ;; \
    "douban") \
        echo "[global]" > /home/rayuser/.pip/pip.conf && \
        echo "index-url = https://pypi.doubanio.com/simple/" >> /home/rayuser/.pip/pip.conf && \
        echo "trusted-host = pypi.doubanio.com" >> /home/rayuser/.pip/pip.conf ;; \
    "ustc") \
        echo "[global]" > /home/rayuser/.pip/pip.conf && \
        echo "index-url = https://pypi.mirrors.ustc.edu.cn/simple/" >> /home/rayuser/.pip/pip.conf && \
        echo "trusted-host = pypi.mirrors.ustc.edu.cn" >> /home/rayuser/.pip/pip.conf ;; \
    *) \
        echo "使用默认pip源..." ;; \
    esac

# 创建虚拟环境
RUN python3.9 -m venv rayenv

# 安装Python依赖
RUN . rayenv/bin/activate && \
    pip install --no-cache-dir "ray[default]" gmpy2

# 暴露Ray和Redis端口
EXPOSE 8000 6279

# 添加环境变量用于配置Redis密码
ENV REDIS_PASSWORD=changeme123

# 添加健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD ray status || exit 1

# 设置入口点
ENTRYPOINT ["/bin/bash", "-c", "source rayenv/bin/activate && exec \"$@\"", "--"]

# 默认启动Ray head节点
CMD ["ray", "start", "--head", "--port=8000", "--redis-password=$REDIS_PASSWORD"]
```

启动Docker容器：在每台服务器上，使用Docker命令启动容器。

构建方法： 

``` bash
docker build -t raytest .
```

运行方法： 

运行头节点：

``` bash
docker run -d --name ray-head -p 8000:8000 raytest ray start --head --port=8000
```

运行工作节点：

``` bash
docker run -d --name ray-worker raytest ray start --address="xxx.xxx.xx.xx:8000" 
```

### （三）测试结果
在Docker容器中运行分布式Ray集群，性能测试结果与物理机部署相似，这里就不重复展示了。总体而言，Docker部署的分布式Ray集群在吞吐量、资源利用率与节点效率等较直接部署并无显著的性能损失，反而在部署与管理上提供了更高的灵活性与便利性。

## 八、实验报告发布

本次实验报告已发布在[罗浩民的个人博客上](https://luohaomin.github.io/Luo-Haomin/2025/07/03/Ray分布式计算框架测试报告/)，报告详细记录了实验过程、性能测试结果与分析总结，供读者参考与交流。

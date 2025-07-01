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
EXPOSE 8000 6379

# 添加环境变量用于配置Redis密码
ENV REDIS_PASSWORD=changeme123

# 添加健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD ray status || exit 1

# 设置入口点
ENTRYPOINT ["/bin/bash", "-c", "source rayenv/bin/activate && exec \"$@\"", "--"]

# 默认启动Ray head节点
CMD ["ray", "start", "--head", "--port=8000", "--redis-password=$REDIS_PASSWORD"]

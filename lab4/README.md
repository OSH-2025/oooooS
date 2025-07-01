# lab4 README

进入lab4/img目录后，执行以下命令：

# Dockerfile

构建方法： 

``` bash
docker build -t raytest .
```

运行方法： 

运行工作节点：

``` bash
docker run -d --name ray-worker raytest ray start --address="ray-head:8000" --redis-password="1234"
```

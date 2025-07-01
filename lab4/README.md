# lab4 README

进入lab4/img目录后，执行以下命令：

# Dockerfile

构建方法： 

``` bash
docker build -t raytest .
```

运行方法： 

``` bash
docker run -d --name ray-head  -p 8000:8000  -p 6379:6379   raytest
```

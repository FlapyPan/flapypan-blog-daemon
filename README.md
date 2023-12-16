# flapypan-blog-daemon

博客状态采集客户端

## 编译

```powershell
cargo build
```

## 启动

### 使用 cmd

```cmd
# 设置环境变量
set DAEMON_SERVER='http://localhost:3000/api/status/author'
set DAEMON_TOKEN='<后台获取token>'
cargo run
```

### 使用 powershell

```powershell
# 设置环境变量
$Env:DAEMON_SERVER = 'http://localhost:3000/api/status/author'
$Env:DAEMON_TOKEN = '<后台获取token>'
cargo run
```

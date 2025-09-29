# 安装指南

本指南详细说明如何安装和配置 `distributed` 分布式系统库。

## 📋 系统要求

### 最低要求

- **Rust**: 1.70.0 或更高版本
- **操作系统**: Linux, macOS, Windows
- **内存**: 至少 2GB RAM
- **存储**: 至少 1GB 可用空间

### 推荐配置

- **Rust**: 1.75.0 或更高版本
- **内存**: 8GB RAM 或更多
- **存储**: SSD，至少 10GB 可用空间
- **网络**: 稳定的网络连接（用于下载依赖）

## 🔧 安装步骤

### 1. 安装 Rust

如果尚未安装 Rust，请先安装：

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 2. 添加依赖到项目

在您的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
distributed = "0.5.0"

# 必需的依赖
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# 可选的依赖（根据需要添加）
# 用于配置管理
config = "0.13"
toml = "0.8"

# 用于加密
ring = "0.17"
rustls = "0.21"

# 用于网络通信
tonic = "0.10"
prost = "0.12"

# 用于数据序列化
bincode = "1.3"
rmp-serde = "1.1"
```

### 3. 基本安装

```bash
# 创建新项目
cargo new my-distributed-app
cd my-distributed-app

# 添加依赖（编辑 Cargo.toml 后）
cargo build
```

### 4. 验证安装

创建简单的测试文件：

```rust
// src/main.rs
use distributed::consistency::ConsistencyLevel;
use distributed::replication::LocalReplicator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("distributed 安装成功！");
    
    // 测试基本功能
    let replicator = LocalReplicator::new(3, 2, 2);
    replicator.replicate("test", "value", ConsistencyLevel::Quorum).await?;
    
    let value = replicator.read("test", ConsistencyLevel::Quorum).await?;
    println!("读取结果: {:?}", value);
    
    Ok(())
}
```

运行测试：

```bash
cargo run
```

## 🐳 Docker 安装

### 使用预构建镜像

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-distributed-app /usr/local/bin/

CMD ["my-distributed-app"]
```

构建和运行：

```bash
# 构建镜像
docker build -t my-distributed-app .

# 运行容器
docker run -p 8080:8080 my-distributed-app
```

### 使用 Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
    volumes:
      - ./config:/app/config
      - ./data:/app/data
    
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
```

启动服务：

```bash
docker-compose up -d
```

## ☸️ Kubernetes 部署

### 创建 ConfigMap

```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: distributed-app-config
data:
  config.toml: |
    [consistency]
    level = "quorum"
    
    [performance]
    connection_pool_size = 100
    batch_size = 1000
    
    [monitoring]
    enable_metrics = true
    enable_tracing = true
```

### 创建 Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: distributed-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: distributed-app
  template:
    metadata:
      labels:
        app: distributed-app
    spec:
      containers:
      - name: distributed-app
        image: my-distributed-app:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /app/config
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
      volumes:
      - name: config
        configMap:
          name: distributed-app-config
```

### 创建 Service

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: distributed-app-service
spec:
  selector:
    app: distributed-app
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP
```

部署到 Kubernetes：

```bash
# 应用配置
kubectl apply -f k8s/

# 检查部署状态
kubectl get pods
kubectl get services
```

## 🔧 配置选项

### 基本配置

```toml
# config.toml
[consistency]
level = "quorum"  # strong, quorum, eventual
quorum_reads = true
quorum_writes = true

[performance]
connection_pool_size = 100
batch_size = 1000
batch_timeout = "10ms"
max_concurrent_requests = 1000

[timeouts]
read_timeout = "100ms"
write_timeout = "200ms"
connection_timeout = "5s"

[retry]
max_retries = 3
retry_backoff = "100ms"
retry_jitter = true

[monitoring]
enable_metrics = true
enable_tracing = true
metrics_port = 9090
tracing_endpoint = "http://jaeger:14268/api/traces"

[logging]
level = "info"  # trace, debug, info, warn, error
format = "json"  # json, pretty
output = "stdout"  # stdout, file
```

### 高级配置

```toml
# config.toml
[consensus.raft]
election_timeout_min = "150ms"
election_timeout_max = "300ms"
heartbeat_interval = "50ms"
snapshot_interval = "10m"
max_log_entries = 10000

[replication]
replication_factor = 3
read_repair = true
hinted_handoff = true
hinted_handoff_timeout = "1h"

[security]
enable_tls = true
cert_file = "/certs/server.crt"
key_file = "/certs/server.key"
ca_file = "/certs/ca.crt"

[storage]
data_dir = "/data"
wal_dir = "/data/wal"
snapshot_dir = "/data/snapshots"
max_file_size = "1GB"
```

## 🧪 测试安装

### 运行测试套件

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_replication

# 运行集成测试
cargo test --test integration

# 运行性能测试
cargo test --release --test performance
```

### 基准测试

```bash
# 安装基准测试工具
cargo install criterion

# 运行基准测试
cargo bench

# 运行特定基准
cargo bench replication
```

### 代码质量检查

```bash
# 运行 clippy
cargo clippy -- -D warnings

# 运行 fmt
cargo fmt

# 运行测试覆盖率
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 🚨 故障排查

### 常见问题

#### 1. 编译错误

```bash
# 错误：找不到依赖
error: failed to select a version for `tokio`

# 解决方案：更新 Cargo.toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
```

#### 2. 运行时错误

```bash
# 错误：连接被拒绝
Error: Connection refused

# 解决方案：检查配置
[network]
bind_address = "0.0.0.0"
port = 8080
```

#### 3. 性能问题

```bash
# 错误：内存使用过高
Error: Out of memory

# 解决方案：调整配置
[performance]
cache_size = 1000  # 减少缓存大小
connection_pool_size = 50  # 减少连接池大小
```

### 调试技巧

#### 启用详细日志

```bash
# 设置环境变量
export RUST_LOG=debug
export RUST_BACKTRACE=1

# 运行应用
cargo run
```

#### 使用调试器

```bash
# 安装调试工具
cargo install cargo-debug

# 运行调试
cargo debug
```

#### 性能分析

```bash
# 安装性能分析工具
cargo install flamegraph

# 生成火焰图
cargo flamegraph
```

## 📚 下一步

安装完成后，您可以：

1. **开始开发** → [快速开始指南](./QUICKSTART.md)
2. **学习理论** → [一致性模型详解](./consistency/README.md)
3. **查看示例** → [示例代码](./examples/README.md)
4. **了解最佳实践** → [系统设计最佳实践](./design/BEST_PRACTICES.md)

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/distributed/issues)
- **Discussions**: [讨论交流](https://github.com/your-org/distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/c20-distributed)

---

**安装完成！** 🎉 现在您可以开始构建分布式应用了。如有任何问题，请随时联系我们。

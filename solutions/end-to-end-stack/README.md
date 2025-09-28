# 端到端分布式计算参考栈

## 概述

本参考栈提供了一个完整的分布式计算解决方案，集成了 Foundations + DataFusion 查询服务、Vector 分布式日志拓扑、多语言客户端和完整的监控体系。

## 架构组件

### 核心服务

- **DataFusion 服务**: 基于 Foundations 的分布式 SQL 查询引擎
- **Vector Edge**: 边缘节点日志收集器
- **Vector Aggregator**: 日志聚合和降采样节点
- **NATS**: 高性能消息队列
- **ClickHouse**: 列式数据库，用于日志存储和分析
- **Prometheus**: 指标收集和存储
- **Grafana**: 数据可视化和仪表板

### 客户端支持

- **Python**: Arrow Flight 客户端
- **Go**: gRPC 客户端
- **Java**: Maven + gRPC 客户端
- **Rust**: 原生客户端

## 快速开始

### 1. 环境要求

- Docker 和 Docker Compose
- Python 3.8+ (用于客户端测试)
- Go 1.21+ (可选)
- Java 17+ (可选)
- Rust 1.90+ (可选)

### 2. 一键部署

```bash
# 克隆项目
git clone <repository-url>
cd solutions/end-to-end-stack

# 运行部署脚本
./scripts/deploy-all.sh dev
```

### 3. 验证部署

```bash
# 运行端到端测试
python examples/complete-demo.py

# 运行性能测试
python benchmarks/performance_test.py

# 运行单元测试
python -m pytest tests/
```

## 详细使用指南

### DataFusion 服务

#### 启动服务

```bash
cd solutions/foundations-datafusion
cargo run
```

#### 客户端连接

```python
import pyarrow.flight as fl

# 连接到服务
client = fl.connect("grpc://localhost:50051")

# 执行查询
ticket = fl.Ticket(b"SELECT * FROM users LIMIT 10")
reader = client.do_get(ticket)
table = reader.read_all()
print(table.to_pandas())
```

### Vector 分布式拓扑

#### 启动边缘节点

```bash
cd solutions/vector-topology
./scripts/start-edge.sh
```

#### 启动聚合节点

```bash
./scripts/start-aggregator.sh
```

#### 配置说明

- `config/edge.toml`: 边缘节点配置，负责日志收集
- `config/aggregator.toml`: 聚合节点配置，负责日志聚合和存储

### 监控和可观测性

#### 访问监控界面

- **Grafana**: <http://localhost:3000> (admin/admin)
- **Prometheus**: <http://localhost:9090>
- **Vector Metrics**: <http://localhost:9598> (Edge), <http://localhost:9599> (Aggregator)

#### 关键指标

- 查询 QPS 和延迟
- 日志处理速率
- 服务可用性
- 资源使用率

## 部署策略

### 开发环境

使用 Docker Compose 进行单机部署，适合开发和测试：

```bash
cd solutions/deployment-strategies/docker
docker-compose up -d
```

### 生产环境

使用 Kubernetes 进行多节点高可用部署：

```bash
cd solutions/deployment-strategies/kubernetes
kubectl apply -f datafusion-deployment.yaml
```

### 云基础设施

使用 Terraform 在 AWS 上部署完整的基础设施：

```bash
cd solutions/deployment-strategies/terraform
terraform init
terraform plan
terraform apply
```

## 性能优化

### DataFusion 优化

- 调整连接池大小
- 优化查询计划
- 启用查询缓存
- 使用列式存储格式

### Vector 优化

- 调整批处理大小
- 优化内存缓冲区
- 启用压缩
- 配置适当的重试策略

### 系统优化

- 调整 JVM 参数
- 优化网络配置
- 使用 SSD 存储
- 配置适当的资源限制

## 故障排除

### 常见问题

#### 1. DataFusion 服务无法启动

```bash
# 检查端口占用
netstat -tlnp | grep 50051

# 检查日志
docker-compose logs datafusion
```

#### 2. Vector 日志收集异常

```bash
# 检查 Vector 配置
vector validate config/edge.toml

# 检查指标
curl http://localhost:9598/metrics
```

#### 3. 监控数据缺失

```bash
# 检查 Prometheus 配置
curl http://localhost:9090/api/v1/targets

# 检查 Grafana 数据源
curl http://localhost:3000/api/datasources
```

### 日志分析

```bash
# 查看服务日志
docker-compose logs -f datafusion
docker-compose logs -f vector-edge
docker-compose logs -f vector-aggregator

# 查看 ClickHouse 数据
curl "http://localhost:8123/?query=SELECT%20*%20FROM%20logs.vector_logs_distributed%20LIMIT%2010"
```

## 扩展和定制

### 添加新的数据源

1. 在 Vector 配置中添加新的 source
2. 配置相应的 transform 和 sink
3. 更新监控指标

### 添加新的查询功能

1. 扩展 DataFusion 服务
2. 添加新的 UDF/UDAF
3. 更新客户端代码

### 集成新的存储后端

1. 配置 Vector sink
2. 更新 ClickHouse 表结构
3. 调整 Grafana 仪表板

## 最佳实践

### 安全

- 启用 TLS 加密
- 使用强密码
- 配置防火墙规则
- 定期更新依赖

### 监控

- 设置关键指标告警
- 配置日志轮转
- 定期备份配置
- 监控资源使用

### 运维

- 使用配置管理
- 自动化部署
- 定期健康检查
- 制定灾难恢复计划

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

## 许可证

MIT License - 详见根目录 LICENSE 文件

## 支持

- 文档: [项目文档](docs/)
- 问题: [GitHub Issues](issues/)
- 讨论: [GitHub Discussions](discussions/)

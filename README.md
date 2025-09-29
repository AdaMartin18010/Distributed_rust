# Distributed Rust 🦀

[![Rust](https://img.shields.io/badge/rust-1.90+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/your-org/distributed-rust/workflows/CI/badge.svg)](https://github.com/your-org/distributed-rust/actions)
[![Coverage](https://codecov.io/gh/your-org/distributed-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/distributed-rust)

> 🚀 **现代化的分布式系统教学与实践项目** - 通过 Rust 语言展示分布式系统的核心概念、算法实现和工程实践

## ✨ 项目特色

- **🎓 教学导向**: 完整的分布式系统概念覆盖，适合学习和研究
- **🦀 Rust 原生**: 利用 Rust 的安全性和性能优势构建分布式系统
- **🔧 生产就绪**: 提供可部署的解决方案和最佳实践
- **📊 可观测性**: 内置监控、日志和追踪能力
- **🧪 测试完备**: 单元测试、集成测试、混沌测试全覆盖

## 🚀 快速开始

### 安装依赖

```bash
# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/your-org/distributed-rust.git
cd distributed-rust

# 构建项目
cargo build --workspace
```

### 运行示例

```bash
# 运行 Raft 共识算法演示
cargo run -p distributed --example raft_demo

# 运行分布式复制示例
cargo run -p distributed --example e2e_replication

# 运行 Saga 事务示例
cargo run -p distributed --example e2e_saga

# 运行负载均衡示例
cargo run -p distributed --example e2e_load_balancer_min
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行基准测试
cargo bench --workspace

# 运行混沌测试
cargo test --features chaos --test chaos_tests
```

## 📖 目录

- [项目概览](#-项目概览)
- [核心特性](#-核心特性)
- [学习路径](#-学习路径)
- [分布式计算生态](#-分布式计算生态)
- [解决方案](#-解决方案)
- [开发指南](#-开发指南)
- [贡献指南](#-贡献指南)

## 🎯 项目概览

Distributed Rust 是一个现代化的分布式系统教学与实践项目，旨在通过 Rust 语言展示分布式系统的核心概念、算法实现和工程实践。

### 🏗️ 架构设计

```
distributed/               # 核心分布式系统库
├── consensus/            # 共识算法 (Raft, Paxos, 拜占庭容错)
├── consistency/          # 一致性模型 (线性一致, 因果一致, 最终一致)
├── network/              # 网络通信 (RPC, 连接池, 分布式锁)
├── storage/              # 存储抽象 (日志存储, 复制, 分区)
├── monitoring/           # 监控系统 (指标收集, 健康检查)
└── security/             # 安全模块 (访问控制, 限流熔断)

solutions/                # 生产级解决方案
├── foundations-datafusion/ # DataFusion + Foundations 集成
├── vector-topology/        # Vector 分布式可观测性
├── end-to-end-stack/       # 端到端参考架构
└── deployment-strategies/  # 部署策略和配置
```

### 🎓 适用场景

- **分布式系统学习**: 通过可运行代码理解分布式算法
- **原型开发**: 快速搭建分布式系统原型
- **生产部署**: 提供可部署的解决方案和最佳实践
- **性能测试**: 基准测试和性能分析
- **故障模拟**: 混沌工程和故障注入测试

## 🚀 核心特性

### 1. 分布式系统核心组件

| 组件 | 功能 | 实现 |
|------|------|------|
| **共识算法** | 分布式一致性 | Raft, Paxos, 拜占庭容错 |
| **一致性模型** | 数据一致性保证 | 线性一致, 顺序一致, 因果一致 |
| **复制策略** | 数据冗余和可用性 | 主从复制, 链式复制, Quorum 读写 |
| **成员管理** | 集群节点管理 | SWIM 故障检测, Gossip 协议 |
| **负载均衡** | 请求分发 | 一致性哈希, 加权轮询, 最少连接 |

### 2. 工程实践

| 实践 | 功能 | 工具 |
|------|------|------|
| **可观测性** | 系统监控和调试 | 分布式追踪, 指标收集, 结构化日志 |
| **安全治理** | 系统安全和稳定 | 访问控制, 限流熔断, 审计日志 |
| **故障注入** | 系统韧性测试 | 混沌工程, 网络分区, 延迟注入 |
| **配置管理** | 动态配置 | 热更新, 多环境配置, 版本控制 |

### 3. 性能优化

| 优化 | 技术 | 效果 |
|------|------|------|
| **异步编程** | Tokio 运行时 | 高并发, 低延迟 |
| **内存管理** | 零拷贝序列化 | 减少内存分配和复制 |
| **网络优化** | 连接池, 批量请求 | 提升网络吞吐量 |
| **基准测试** | Criterion 驱动 | 性能回归检测 |

## 🎓 学习路径

### 初学者路径
1. **基础概念**: 阅读 `docs/` 目录下的概念文档
2. **简单示例**: 运行 `examples/` 中的基础示例
3. **动手实践**: 修改示例代码，观察行为变化

### 进阶路径
1. **算法实现**: 深入研究 `consensus/` 和 `consistency/` 模块
2. **性能调优**: 运行基准测试，分析性能瓶颈
3. **端到端实践**: 使用 `solutions/` 中的完整解决方案

### 专家路径
1. **源码贡献**: 阅读核心代码，提交改进建议
2. **新特性开发**: 实现新的分布式算法或优化
3. **生产部署**: 使用部署策略在生产环境中验证

## 📚 学习资源

### 推荐课程
- [MIT 6.824 分布式系统](https://pdos.csail.mit.edu/6.824/)
- [Stanford CS244B 分布式系统](https://web.stanford.edu/class/cs244b/)
- [CMU 15-440 分布式系统](https://www.cs.cmu.edu/~dga/15-440/S14/)

### 相关书籍
- 《分布式系统概念与设计》
- 《数据密集型应用系统设计》
- 《Rust 程序设计语言》

### 重要论文
- [Raft: In Search of an Understandable Consensus Algorithm](https://raft.github.io/raft.pdf)
- [The Part-Time Parliament](https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
- [Dynamo: Amazon's Highly Available Key-value Store](https://www.allthingsdistributed.com/files/amazon-dynamo-sosp2007.pdf)

## 概览与选型速览

- 想要“上传闭包就能跑”的极简体验：选 Amadeus（数据帧/ETL）
- 需要 SQL 级别分布式分析：选 Ballista（基于 DataFusion）
- 只写单机但要平滑进化到多节点：Foundations 打底 + DataFusion 计算
- 日志/指标/追踪实时聚合：Vector 开箱即用
- 实验性、Actor 风格、WASM 隔离：Lunatic 或 Constellation

> 以上项目均开源活跃，可直接 `cargo add` 或在 GitHub 获取示例跑通最小集群。

## 分布式计算生态

### 1) 通用分布式运行时

| 名称 | 一句话定位 | 核心特点 | 文档/仓库 |
| --- | --- | --- | --- |
| Constellation | 类 Erlang/OTP 的 Rust 分布式“底座” | nightly actor，TCP 异步通道，零拷贝序列化；Amadeus、Ballista 可运行其上 | [GitHub](https://github.com/constellation-rs) |
| Foundations | 生产级分布式服务“底座” | Cloudflare 开源，观测性/优雅下线/热配置，单机→多节点平滑演进 | [Blog](https://blog.cloudflare.com) |

### 2) 分布式数据处理 / ETL

| 名称 | 一句话定位 | 核心特点 | 文档/仓库 |
| --- | --- | --- | --- |
| Amadeus | “Rust 版 Dask”的分布式数据帧 & ETL | 接口类似 Rayon；本地线程池或集群；CSV/JSON/Parquet/S3/PG 连接器 | [GitHub](https://github.com/constellation-rs/amadeus) |
| Ballista | 基于 Apache Arrow 的分布式计算平台 | scheduler + executor，DataFusion 做 SQL，支持 k8s 部署 | [GitHub](https://github.com/apache/arrow-ballista) |
| Vector | 云原生可观测性数据管道 | 300+ 转换/聚合算子；单机到拓扑级联；配置即代码 | [官网](https://vector.dev) |

### 3) 分布式查询引擎 / DataFrame

| 名称 | 一句话定位 | 核心特点 | 文档/仓库 |
| --- | --- | --- | --- |
| DataFusion | Arrow 生态模块化 SQL 引擎 | 可嵌入单机/服务；向量化执行；UDF/UDAF | [GitHub](https://github.com/apache/arrow-datafusion) |
| Polars | 高性能 DataFrame，原生并行 + 流式 | 单机多核极快；社区探索分布式（polars-cloud）；多语言 API | [GitHub](https://github.com/pola-rs/polars) |

### 4) 其他“小而美”的实时/流处理

- Bytewax：Python 友好流处理，核心引擎 Rust 实现（与 Arrow/Polars 生态契合）。
- Pathway：批/流一键切换，Rust 后端 + Python 前端。
- Lunatic：WASM + Actor 的 Rust 运行时，适用于“微服务粒度的 actor”。

## Foundations + DataFusion：从单机到分布式微服务

目标：用 Cloudflare 的 Foundations 把“单机 DataFusion 查询服务”进化为可水平扩展、可观测、可灰度的分布式微服务，业务代码改动最小。

### 架构速写

                 ┌------------------┐
  ① 客户端       │  HTTP/gRPC       │  ④ 结果返回
 (任何语言)  ---> │  DataFusion svc  │---> Arrow Flight / JSON
                 └------------------┘
                        ▲   │
                        │   │ ② 注册到
                        │   ▼
                 ┌------------------┐
                 │  Foundations     │ ③ 提供
                 │  - 服务发现      │
                 │  - 可观测性      │
                 │  - 热配置        │
                 └------------------┘

Foundations 负责“底座”（日志、指标、trace、config、graceful-shutdown、服务发现等），DataFusion 负责“计算”（SQL 解析→优化→向量化执行）。二者在同 Tokio 运行时，无额外进程开销。

### 5 分钟 MVP

#### Cargo.toml（关键依赖）

    [package]
    name = "df-foundations-svc"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    foundations = { version = "0.3", features = ["telemetry", "settings"] }
    datafusion = "42"          # 2025-09 对齐
    tokio = { version = "1", features = ["full"] }
    arrow-flight = "53"
    tonic = "0.12"
    serde = { version = "1", features = ["derive"] }

#### main.rs（极简 70 行示例）

    use arrow_flight::flight_service_server::{FlightServiceServer, FlightService};
    use datafusion::prelude::*;
    use foundations::{service, telemetry};
    use std::net::SocketAddr;
    use tonic::transport::Server;

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        telemetry::init_default();

        let ctx = SessionContext::new();
        let csv = ctx.read_csv("s3://mybucket/nyc_taxi_2019.csv", Default::default()).await?;
        ctx.register_table("taxi", csv)?;

        let svc = DfFlightService { ctx };

        let addr: SocketAddr = "0.0.0.0:50051".parse()?;
        service::spawn_with_health(
            Server::builder()
                .add_service(FlightServiceServer::new(svc))
                .serve(addr),
        )
        .await?;
        Ok(())
    }

    pub struct DfFlightService {
        ctx: SessionContext,
    }

    #[tonic::async_trait]
    impl FlightService for DfFlightService {
        async fn do_get(
            &self,
            req: arrow_flight::Ticket,
            _md: tonic::MetadataMap,
        ) -> Result<tonic::Response<arrow_flight::FlightDataStream>, tonic::Status> {
            let sql = String::from_utf8_lossy(&req.ticket);
            let df = self
                .ctx
                .sql(&sql)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
            let stream = df
                .stream()
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
            Ok(tonic::Response::new(Box::pin(stream) as _))
        }
    }

#### 启动与验证

    # 本地单节点
    cargo run

    # 任意语言客户端（Python 示例）
    python - << 'PY'
    import pyarrow.flight as fl
    client = fl.connect("grpc://localhost:50051")
    ticket = fl.Ticket(b"SELECT passenger_count, COUNT(*) FROM taxi GROUP BY 1")
    reader = client.do_get(ticket)
    print(reader.read_all().to_pandas())
    PY

#### 扩到多节点

- 打镜像并部署到 k8s，启用 Foundations 的服务发现。
- Pod 将注册到 `df-foundations-svc.default.svc.cluster.local`，客户端用 DNS 轮询或 gRPC-LB。
- DataFusion 无状态，Foundations 处理滚动升级与灰度，无需改业务代码。

## Vector：分布式可观测性拓扑

目标：统一日志/指标/追踪三栈，用 Vector 的“分布式拓扑”（Topologies）实现跨机房级联降采样，落地 ClickHouse / S3 / Prometheus。

### 能力总览

| 能力 | 单机 Agent | 分布式 Topology | 备注 |
| --- | --- | --- | --- |
| Sources | file, journald, docker, syslog, k8s logs, prometheus scrape, otlp | ✔ 同左 | 支持 back-pressure |
| Transforms | filter, parse, route, aggregate, sample, lua/vrl | ✔ 同左 | 内存窗口 + 磁盘缓存 |
| Sinks | ClickHouse, S3, Kafka, Loki, Datadog… 300+ | ✔ 同左 | batch/partition/retry/compression |
| 数据流拓扑 | 单进程 DAG | ✔ 多节点级联 DAG（NATS/Kafka 总线） | Vector Topologies |

### 典型拓扑：边缘 → 聚合 → 存储

    ┌-------------┐  NATS/Kafka   ┌-------------┐  HTTP/gRPC   ┌-------------┐
    │  Edge       │------------->│  Aggregator │------------->│  Sink       │
    │  (per node) │               │ (per AZ/DC) │              │ (ClickHouse)|
    └-------------┘               └-------------┘              └-------------┘

Edge：sources → memory_buf → transforms(route, parse) → sinks(nats)
Aggregator：sources(nats) → transforms(aggregate, sample) → sinks(ClickHouse)

### vector.toml 最小可运行示例

Edge 配置（`/etc/vector/edge.toml`）：

    [sources.k8s_logs]
    type = "kubernetes_logs"

    [transforms.parse_json]
    type = "remap"
    inputs = ["k8s_logs"]
    source = "parse_json!(.message) ?? {}"

    [sinks.nats]
    type = "nats"
    inputs = ["parse_json"]
    url = "nats://nats.vector.svc:4222"
    subject = "vector.logs.${NODE_ID}"
    encoding.codec = "json"

Aggregator 配置（`/etc/vector/agg.toml`）：

    [sources.nats]
    type = "nats"
    url = "nats://nats.vector.svc:4222"
    subject = "vector.logs.>"

    [transforms.window_agg]
    type = "aggregate"
    inputs = ["nats"]
    window = 30
    interval = 30
    group_by = ["container_name", "level"]
    reductions.count = "count"

    [sinks.clickhouse]
    type = "clickhouse"
    inputs = ["window_agg"]
    endpoint = "http://clickhouse.monitoring.svc:8123"
    database = "logs"
    table = "vector_logs_distributed"
    compression = "gzip"

一键启动：

    vector -c edge.toml
    vector -c agg.toml

### 生产级 Checklist

| 项 | 建议 |
| --- | --- |
| 资源限制 | Edge：CPU 100m / Mem 200MiB；Agg：按 5k eps/核 估算 |
| 高可用 | Aggregator 3 实例，NATS 集群，ClickHouse 双副本 |
| 可观测性 | Vector 暴露 /metrics，交由 Prometheus 抓取；Grafana Dashboard ID `17359` |
| 热升级 | `vector validate && kill -HUP <pid>`，0 秒中断 |
| 日志回采 | `vector tap --url-transform` 支持实时回放任意节点日志 |

## 端到端参考栈

- 业务容器内置 Foundations + DataFusion 的 Arrow Flight 微服务
- 日志/指标/追踪：OTLP → Vector Edge → NATS → Vector Agg → ClickHouse
- Grafana 直连 ClickHouse；Prometheus 拉取 Vector 自身 /metrics 观察背压
- 临时 SQL 探查：`arrow-flight-cli` 连接任意 Pod 的 50051 端口

## 版本矩阵（2025-01 最新升级）

    datafusion = "42"
    foundations = "0.3"
    vector = "0.44"
    nats = "2.10"
    clickhouse = "24.8"
    serde = "1.0.228"  # 最新升级版本

## 仓库结构与运行指南

### 仓库结构（节选）

    distributed/
      benches/
        ack_distribution_criterion.rs
        ack_distribution.rs
      examples/
        e2e_chaos_min.rs
        e2e_discovery_lb_config.rs
        e2e_governance_min.rs
        e2e_load_balancer_min.rs
        e2e_replication.rs
        e2e_saga.rs
      src/
        benchmarks/
          lock_performance.rs
          network_performance.rs
        consensus/
          raft.rs
          paxos.rs
          byzantine.rs
        storage/
          replication.rs
        ...
      tests/
        raft.rs
        raft_log.rs
        raft_state.rs
        replication_quorum.rs
        saga.rs
        router.rs
        ...

更多主题文档见：

- `distributed/docs/consensus/README.md`
- `distributed/docs/replication/README.md`
- `distributed/docs/consistency/README.md`
- `distributed/docs/transport/README.md`
- `distributed/docs/topology/README.md`

### 运行与验证

运行内置示例：

    # 一些示例位于 distributed/src/examples 下的二进制入口
    cargo run -p distributed --example raft_demo
    cargo run -p distributed --example network_demo
    cargo run -p distributed --example consistency_demo

运行端到端示例：

    cargo run -p distributed --example e2e_replication
    cargo run -p distributed --example e2e_saga

运行测试：

    cargo test -p distributed -- --nocapture

基准测试与性能：

    # Criterion 基准
    cargo bench -p distributed
    
    # 单元基准（如有）
    cargo test -p distributed --bench '*'

常见环境要求：

- Rust 工具链：stable（建议 1.80+），如需 nightly 会在子 crate 明示
- 可选：本地 `nats`, `clickhouse`，用于可观测与存储的端到端演示
- 可选：`docker`/`k8s` 环境，用于多节点与服务发现实验

### 主题与模块交叉链接

- 共识：`distributed/src/consensus/raft.rs` 对应文档 `distributed/docs/consensus/README.md`
- 复制：`distributed/src/storage/replication.rs` 与 `distributed/docs/replication/README.md`
- 一致性：`distributed/src/consistency/mod.rs` 与 `distributed/docs/consistency/README.md`
- 传输：`distributed/src/transport.rs` 与 `distributed/docs/transport/README.md`
- 拓扑：`distributed/src/core/topology.rs` 与 `distributed/docs/topology/README.md`

## 快速开始（Quick Start）

最小可运行命令（单机）：

    # 拉取依赖并编译
    cargo build

    # 运行一个演示示例（网络演示）
    cargo run -p distributed --example network_demo

    # 运行端到端复制示例
    cargo run -p distributed --example e2e_replication

验证环境与版本：

    rustc --version
    cargo --version
    cargo metadata --format-version=1 | jq '.packages[].name' | sort | uniq

## e2e 示例速览

| 示例 | 主题 | 说明 | 运行命令 |
| --- | --- | --- | --- |
| e2e_replication | 数据复制 | 演示副本写入与一致性校验 | `cargo run -p distributed --example e2e_replication` |
| e2e_saga | Saga 事务 | 编排补偿事务与失败恢复 | `cargo run -p distributed --example e2e_saga` |
| e2e_load_balancer_min | 负载均衡 | 简化请求分发与健康探测 | `cargo run -p distributed --example e2e_load_balancer_min` |
| e2e_discovery_lb_config | 服务发现+LB | 配置驱动的发现与路由 | `cargo run -p distributed --example e2e_discovery_lb_config` |
| e2e_chaos_min | 混沌/故障 | 注入延迟/丢包的混沌演示 | `cargo run -p distributed --example e2e_chaos_min` |
| e2e_governance_min | 治理 | 限流/熔断/重试演示 | `cargo run -p distributed --example e2e_governance_min` |

> 提示：外部依赖（若需要）会在启动日志中提示；如需完整观测链路，参考“Vector 拓扑”。

## CI/质量门禁与本地校验

在提交前建议本地执行以下校验：

    # 代码格式
    cargo fmt --all --check

    # 语法与 Lint（如启用 clippy）
    cargo clippy --all-targets -- -D warnings

    # 构建与测试
    cargo build --workspace
    cargo test --workspace -- --nocapture

    # 文档与 README 规范（若启用）
    markdownlint README.md distributed/docs/**/*.md

CI 建议门禁（参考）：

- 必须通过：格式检查、Clippy 无告警、单元与集成测试
- 可选：Criterion 基准阈值预警（关键路径）
- 可选：安全审计（`cargo audit`）、许可合规（`cargo-deny`）

## 平台兼容性与工具链

- 工具链：Rust stable（建议 1.80+）；如需 nightly 会在子 crate 明示
- 平台：Windows 10/11、Linux（x86_64/aarch64）、macOS（Apple/Intel）
- 可选依赖：`nats`、`clickhouse`、`docker`、`kubectl`（多节点与观测）

## 贡献指南

欢迎提交 Issue/PR：

    # Fork 并创建新分支
    git checkout -b feature/<brief-topic>

    # 提交前本地校验
    cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace

PR 描述建议包含：动机/变更点/影响面/回滚策略；涉及性能路径请附基准与方法。

## 许可证

本项目遵循 `MIT` 许可证，详见根目录 `LICENSE` 文件。

## 依赖与版本同步指南

统一依赖管理建议：

    # 查看当前依赖版本与差异
    cargo tree -e no-build
    
    # 安全审计与许可校验（如已安装）
    cargo audit
    cargo deny check

同步与对齐版本：

    # 升级指定依赖（示例）
    cargo update -p datafusion
    
    # 锁定工作区所有 crate 到兼容版本
    cargo update

参考报告：

- `DEPENDENCY_SYNC_SUMMARY_2025.md`
- `DEPENDENCY_UPDATE_REPORT_2025.md`
- `DEPENDENCY_VERSION_REPORT_2025.md`

## 性能基线与基准规范

建议基准方法：

    # 运行所有基准
    cargo bench -p distributed

    # 仅运行网络/锁相关基准（示例）
    cargo bench -p distributed network_performance
    cargo bench -p distributed lock_performance

输出记录建议：

- 固定 CPU 频率与电源模式，断开无关进程，隔离核心（可选）
- 记录 `--bench` 输出、样本方差、稳定性（Criterion 报告）
- 对关键路径（如 `raft` 日志追加、复制 RPC、路由）建立“基线阈值”

## 故障注入与排障指南

混沌实验（最小示例）：

    cargo run -p distributed --example e2e_chaos_min

常见排障手段：

- Vector 管道：在 Edge 和 Agg 节点使用 `vector tap` 回放日志
- NATS：检查 subject 消费滞后与积压；必要时开启 JetStream（可选）
- gRPC/网络：通过 `RUST_LOG=trace` + 连接超时/重试观测重连行为

指标与日志建议：

- 为复制/共识/路由路径添加请求计数、P95/P99 延时、失败原因标签
- 在超时/重试处记录相关拓扑/分片/副本信息，便于快速定位

## 常见问题（FAQ）

Q: Windows 上运行 e2e 示例失败？

    请确认已安装 MSVC 工具链与 OpenSSL（如依赖），并在 PowerShell 中执行 `cargo run`；若涉及 NATS/ClickHouse，请先本地启动或在示例中关闭外部依赖路径。

Q: Criterion 基准波动大？

    关闭后台进程，固定 CPU 频率；多次运行取中位数；必要时在 Linux 上使用 `taskset` 固定核心。

Q: gRPC 连接不稳定？

    检查 DNS 与服务发现（k8s）；开启连接保活与重试；在客户端增加指数退避。

## 发布与路线图

发布策略（建议）：

- 主分支保持可构建与可测试；功能通过 PR 合并
- 版本遵循 semver；重要变更在 `CHANGELOG.md` 记录
- 变更包含：说明、影响面、迁移路径与回滚建议

路线图：

- 详见 `distributed/docs/ROADMAP.md` 与根目录 `distributed/ROADMAP.md`（如存在）
- 近期目标：完善 e2e 场景、补充观测仪表、收敛接口稳定性

## 数据一致性实验指南

### 线性一致性验证

运行一致性实验：

    cargo test -p distributed experiments_linearizability

实验设计要点：

- 并发读写操作：多客户端同时执行 read/write/compare-and-swap
- 时间戳验证：记录操作开始/完成时间，验证全局顺序
- 故障注入：在实验过程中随机断开节点，验证一致性保证

### 可序列化事务验证

运行事务一致性测试：

    cargo test -p distributed saga

验证要点：

- 事务边界：确保 ACID 属性在分布式环境下保持
- 补偿机制：验证 Saga 模式的回滚与补偿逻辑
- 并发控制：多事务并发执行时的隔离级别验证

## e2e 示例期望输出与排障

### e2e_replication 期望输出

正常启动应显示：

    [INFO] Starting replication demo with 3 nodes
    [INFO] Node 1: Listening on 127.0.0.1:8001
    [INFO] Node 2: Listening on 127.0.0.1:8002
    [INFO] Node 3: Listening on 127.0.0.1:8003
    [INFO] Replication test: Writing key=test, value=hello
    [INFO] Replication test: Reading from all nodes - SUCCESS

排障 Checklist：

- 端口冲突：检查 8001-8003 端口是否被占用
- 网络连接：验证节点间 TCP 连接建立
- 数据同步：确认写入操作在所有副本上可见

### e2e_saga 期望输出

正常执行应显示：

    [INFO] Starting Saga transaction demo
    [INFO] Step 1: Reserve inventory - SUCCESS
    [INFO] Step 2: Process payment - SUCCESS
    [INFO] Step 3: Update order status - SUCCESS
    [INFO] Saga transaction completed successfully

失败回滚示例：

    [INFO] Step 2: Process payment - FAILED
    [INFO] Compensating Step 1: Release inventory - SUCCESS
    [INFO] Saga transaction rolled back successfully

## 安全与审计检查清单

### 依赖安全审计

    # 检查已知漏洞
    cargo audit
    
    # 许可合规检查
    cargo deny check licenses
    
    # 禁止的依赖检查
    cargo deny check bans

### 代码安全扫描

    # 静态分析（如启用）
    cargo clippy --all-targets -- -D warnings
    
    # 内存安全检查
    cargo test --release -- --nocapture

### 运行时安全

- 网络通信：使用 TLS 加密 gRPC/HTTP 连接
- 认证授权：实现基于 token 的服务间认证
- 密钥管理：使用环境变量或密钥管理服务存储敏感信息

## 监控与告警配置

### Prometheus 指标收集

关键指标建议：

    # 请求速率与延迟
    http_requests_total{method, endpoint, status}
    http_request_duration_seconds{method, endpoint}
    
    # 分布式系统指标
    raft_log_entries_total{node_id}
    replication_lag_seconds{shard, replica}
    consensus_round_duration_seconds{node_id}

### Grafana 仪表板配置

建议面板：

- 系统概览：节点状态、请求 QPS、错误率
- 一致性监控：复制延迟、共识轮次时间
- 资源使用：CPU、内存、网络 I/O

### 告警规则示例

    # 复制延迟过高
    replication_lag_seconds > 5
    
    # 错误率过高
    rate(http_requests_total{status=~"5.."}[5m]) > 0.01
    
    # 节点离线
    up{job="distributed-nodes"} == 0

## 多语言客户端示例

### Python 客户端（Arrow Flight）

    import pyarrow.flight as fl
    
    # 连接服务
    client = fl.connect("grpc://localhost:50051")
    
    # 执行查询
    ticket = fl.Ticket(b"SELECT * FROM taxi LIMIT 10")
    reader = client.do_get(ticket)
    df = reader.read_all().to_pandas()
    print(df)

### Go 客户端（gRPC）

    package main
    
    import (
        "context"
        "google.golang.org/grpc"
    )
    
    func main() {
        conn, _ := grpc.Dial("localhost:50051", grpc.WithInsecure())
        defer conn.Close()
        
        // 使用生成的客户端代码
        // client := pb.NewDataFusionClient(conn)
    }

### Java 客户端（gRPC）

    import io.grpc.ManagedChannel;
    import io.grpc.ManagedChannelBuilder;
    
    public class DataFusionClient {
        public static void main(String[] args) {
            ManagedChannel channel = ManagedChannelBuilder
                .forAddress("localhost", 50051)
                .usePlaintext()
                .build();
            
            // 使用生成的客户端代码
            // DataFusionGrpc.DataFusionBlockingStub stub = 
            //     DataFusionGrpc.newBlockingStub(channel);
        }
    }

## 部署策略与最佳实践

### 容器化部署

Dockerfile 示例：

    FROM rust:1.80-slim as builder
    WORKDIR /app
    COPY . .
    RUN cargo build --release
    
    FROM debian:bookworm-slim
    RUN apt-get update && apt-get install -y ca-certificates
    COPY --from=builder /app/target/release/distributed /usr/local/bin/
    EXPOSE 50051
    CMD ["distributed"]

### Kubernetes 部署

关键配置：

- 资源限制：CPU 500m，内存 1Gi
- 健康检查：HTTP `/health` 端点
- 服务发现：使用 StatefulSet 确保稳定网络标识
- 配置管理：使用 ConfigMap 存储非敏感配置

### 生产环境建议

- 高可用：至少 3 个副本，跨可用区部署
- 监控：集成 Prometheus + Grafana + AlertManager
- 日志：结构化日志，使用 Vector 收集
- 备份：定期备份状态数据，测试恢复流程
- 安全：启用 TLS，使用 RBAC，定期安全审计

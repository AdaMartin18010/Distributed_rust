# 分布式系统总纲（c20_distributed）

本目录系统性梳理分布式系统核心主题，兼顾工程接口与理论依据，对齐主流课程与社区知识体系。

## 目录与定位

- topology：数据分片与路由（一致性哈希、重平衡、热点治理）
- replication：复制与放置策略（主从、多主、链式、读写分离）
- consensus：共识与状态机复制（Raft、Paxos、EPaxos、VSR）
- consistency：一致性模型（线性一致、顺序、因果、最终一致；CAP/PACELC）
- storage：日志/WAL、快照与恢复（状态机持久化）
- transport：RPC/超时/重试/幂等/背压
- transactions：分布式事务（SAGA、TCC、幂等等价类）
- failure：故障模型与容错（Fail-Stop、网络分区、拜占庭、FLP）
- time：时间与时钟（NTP/PTP、Lamport/Vector、TrueTime/Spanner）
- scheduling：限流、调度、优先级与负载治理
- testing：分布式测试与混沌工程（Jepsen、故障注入、可重复实验）

## 能力地图（对标）

1) 理论：CAP/PACELC、FLP 不可能性、时钟模型、共识安全性/活性
2) 模型：一致性级别、复制语义、容错与隔离级别
3) 工程：路由/放置、日志/快照、RPC/重试/幂等、监控与回滚
4) 验证：单元/属性测试、仿真、Jepsen、故障注入与回归

## 学习路线（参考课程）

- MIT 6.824 分布式系统
- Stanford CS244B Distributed Systems
- CMU 15-440/15-749、Berkeley CS262A
- EPFL Distributed Systems、UWash CSE452

## 维基与进一步阅读

- Wikipedia：CAP、Consensus、Paxos、Raft、Causal consistency、Vector clock
- Papers：Raft, Paxos, EPaxos, Spanner/TrueTime, Dynamo, Cassandra, FaRM

各专题文档末尾提供具体参考与实现接口对照表。

## 🔗 快速导航

- 模型理论：`../../formal_rust/language/18_model/01_model_theory.md`
- AI系统：`../c19_ai/docs/FAQ.md`
- WebAssembly：`../../formal_rust/language/16_webassembly/FAQ.md`
- IoT系统：`../../formal_rust/language/17_iot/FAQ.md`
- 区块链：`../../formal_rust/language/15_blockchain/FAQ.md`

## 分布式系统（Rust 1.89 对齐）

- 课程参考：MIT 6.824/6.5840、CMU 15-440/15-418、Stanford CS244B、Berkeley CS262A
- 主题导航：一致性与分区、共识、成员管理、复制、事务、调度、容错、监控

## 子目录

- [consensus](./consensus/) — Raft/Paxos/EPaxos，日志复制、选举、快照
- [consistency](./consistency/) — CAP/PACELC、线性/顺序/因果/最终一致
- [replication](./replication/) — 主从/多主、链式复制、放置与 Quorum
- [storage](./storage/) — 状态机存储、WAL、快照与恢复
- [transport](./transport/) — RPC/超时/重试/幂等/背压
- [scheduling](./scheduling/) — 限流、背压、优先级与截止时间传递
- [topology](./topology/) — Sharding、一致性哈希、重平衡与热点治理
- [transactions](./transactions/) — SAGA/TCC/2PC、幂等与隔离级别
- [failure](./failure/) — 故障模型（Fail-Stop/BFT）、FLP 与容错界
- [time](./time/) — 物理/逻辑时钟、TrueTime 与外部一致性
- [testing](./testing/) — Jepsen、仿真、故障注入与线性化检查
- [membership](./membership/) — SWIM/Gossip、成员视图与故障探测
- [observability](./observability/) — Tracing/Metrics/Logging 与 SLO
- [experiments](./experiments/) — 实验与测试指引，配套 tests 的检查清单
  - 清单：见 [experiments/CHECKLIST.md](./experiments/CHECKLIST.md)

## 路由与分片

- 在 `partitioning` 中提供 `HashRingRouter`，基于 `topology::ConsistentHashRing` 进行键到节点映射

## Feature 矩阵

- `runtime-tokio`：启用基于 Tokio 的定时器/异步能力（`scheduling::TokioTimer`）。
- `consensus-raft`：启用最小 Raft 接口与示例（`consensus_raft::*`）。
- `consensus-paxos`：预留；未来可扩展 Multi-Paxos/EPaxos。

## Rust 1.89 对齐

- 使用 `edition = 2024`、`rust-version = 1.89`
- 可选特性：`runtime-tokio`、`consensus-raft`、`consensus-paxos`
- 建议配套 crates：`tracing`、`ahash`

## 最小示例

```rust
use c20_distributed::{InMemoryRpcServer, InMemoryRpcClient};

let mut srv = InMemoryRpcServer::new();
srv.register("echo", Box::new(|b| b.to_vec()));
let cli = InMemoryRpcClient::new(srv.clone());
let rsp = cli.call("echo", b"hi").unwrap();
assert_eq!(rsp, b"hi");
```

启用 tokio 定时器（需要 feature `runtime-tokio`）：

```rust
use c20_distributed::scheduling::{TimerService, TokioTimer};

let timer = TokioTimer::default();
timer.after_ms(10, || println!("tick"));
```

## 流水线（路由→放置→复制→幂等）

```rust
use c20_distributed::topology::ConsistentHashRing;
use c20_distributed::replication::LocalReplicator;
use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::storage::InMemoryIdempotency;

let mut ring = ConsistentHashRing::new(16);
let nodes = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
for n in &nodes { ring.add_node(n); }
let mut repl: LocalReplicator<String> = LocalReplicator::new(ring, nodes.clone())
    .with_idempotency(Box::new(InMemoryIdempotency::<String>::default()));
let id = "op-1".to_string();
repl.replicate_idempotent(&id, &nodes, b"cmd".to_vec(), ConsistencyLevel::Quorum).unwrap();
```

## 快速开始

- 运行单元测试：`cargo test -p c20_distributed`
- 运行示例：`cargo run -p c20_distributed --example e2e_saga`
- 查看实验说明：`docs/EXPERIMENT_GUIDE.md`
- 查看路线图：`docs/ROADMAP.md`

## 如何选择一致性级别

- Strong/Quorum：读写延迟更高，但可避免陈旧读；适合强一致 KV/元数据。
- Eventual：吞吐更高、延迟更低，允许短暂陈旧；适合日志/时间序列等。
- 建议：先以 Quorum 写 + 读多数派作为基线，再按 SLA 调整。

## 常见陷阱

- 见 `docs/PITFALLS.md`：多数派边界、Eventual 读旧值、一致性哈希倾斜、Saga 幂等、SWIM 参数等。

## 实验入口

- 复制与一致性、Saga、哈希环、SWIM、基准：详见 `docs/EXPERIMENT_GUIDE.md`。

## 测试导航

- 共识/复制：`tests/raft*.rs`, `tests/replication*`
- 传输/重试：`tests/retry*.rs`, `tests/pipeline.rs`
- SWIM/成员视图：`tests/swim_*.rs`, `tests/router.rs`

## 练习与思考

1. 设计一条“路由→放置→复制→一致性→幂等→补偿”的最小流水线，基于内存后端实现端到端一致写入，并使用属性测试验证线性化。
2. 实现一个最小 Raft 原型：领导者选举、日志复制、快照；在 `tokio::time::timeout` 下注入网络分区和超时，评估活性与恢复时间。
3. 基于 Merkle 树实现反熵同步，比较 Range Diff 与 Merkle Diff 的带宽与延迟差异，给出实验数据与图表。
4. 设计 Saga 事务编排，给出可重试与幂等策略，注入失败场景验证补偿正确性与可观测性指标。

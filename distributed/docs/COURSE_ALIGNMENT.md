# 分布式系统课程对标（国际一流课程与本仓库模块映射）

本文件对标国际名校课程（示例：MIT 6.824/6.5840、CMU 15-440/15-745、Stanford CS244B、Berkeley CS262A、Cornell CS7410）与 Wikipedia 经典主题，将其核心单元映射到本 crate 的模块与示例代码，支持“概念-代码-实验”一体化学习路径。

- 本 crate 模块：`membership`、`topology`、`consensus`、`partitioning`、`consistency`、`transport`、`storage`、`scheduling`、`codec`、`swim`、`replication`、`transactions`（Saga）。

## 1) 节点与成员管理（Membership）

- 课程对标：MIT 6.824 Lab: Fault Tolerance 前置；CMU 15-440 Gossip & Membership；Cornell CS7410 Group Membership。
- Wikipedia：Failure detector、Gossip protocol、Membership service。
- 代码映射：`src/membership.rs`，`src/swim.rs`（SWIM 事件/状态、传播）。
- 学习要点：故障检测强弱完备性、视图一致性、反熵与传播周期参数。
- 推荐阅读：SWIM 论文、Epidemic protocols 综述。
- 本仓库实验：参见实验 4（SWIM 探测）。
- 关联测试：`tests/swim_*.rs`。
- 评估方式：给定参数下的误报率与收敛时间测量。

## 2) 集群拓扑与分片（Topology/Partitioning）

- 课程对标：MIT 6.824 Sharding/Partitioning；Berkeley CS262A Scalable storage；CMU 15-440 Consistent Hashing。
- Wikipedia：Consistent hashing、Shard、Virtual node。
- 代码映射：`src/topology.rs`（一致性哈希环）、`src/partitioning.rs`。
- 学习要点：虚拟节点、再均衡代价、热点与倾斜控制。
- 推荐阅读：Dynamo、Cassandra 分片设计。
- 本仓库实验：实验 3（一致性哈希与再均衡）。
- 关联测试：`tests/hashring_properties.rs`。
- 评估方式：扩容一次的键迁移比例接近 1/N。

## 3) 共识与复制（Consensus & Replication）

- 课程对标：MIT 6.824 Raft Labs；Stanford Raft/Paxos lectures；Cornell Paxos/VRR。
- Wikipedia：Paxos、Raft、Viewstamped Replication、Quorum。
- 代码映射：`src/consensus.rs` 抽象；可选 `consensus-raft`；`src/replication.rs` 中 MajorityQuorum、Replicator。
- 学习要点：日志复制、一致性语义、过半法定人数、领导者选举与任期。
- 推荐阅读：Raft 论文、Paxos Made Simple。
- 本仓库实验：实验 1（复制与一致性），M4（最小 Raft）。
- 关联测试：`tests/replication_local.rs`、`tests/raft_minimal.rs`（规划）。
- 评估方式：提交索引单调性与读屏障正确性。

## 4) 一致性级别（Consistency Levels）

- 课程对标：Berkeley CS262A CAP/PACELC；MIT 6.824 Linearizability vs Eventual；CMU 15-440 Consistency models。
- Wikipedia：Linearizability、Sequential consistency、Eventual consistency、Quorum reads/writes。
- 代码映射：`src/consistency.rs`（Level 枚举）与 `replication.rs` 的 `required_acks` 计算。
- 学习要点：线性一致/顺序一致/最终一致的可观测差异与测试方法（Jepsen 思路）。
- 推荐阅读：Herlihy & Wing（Linearizability）。
- 本仓库实验：实验 1、5（基准）。
- 关联测试：`tests/consistency_matrix.rs`（规划）。
- 评估方式：会话保证子集与线化小规模历史通过率。

## 5) 存储与状态机（Storage & State Machine）

- 课程对标：MIT 6.824 KV/SMR；Berkeley 事务存储；CMU 日志结构存储。
- Wikipedia：State machine replication、Write-ahead logging、Idempotency。
- 代码映射：`src/storage.rs`（`StateMachineStorage`、`LogStorage`、`IdempotencyStore`）。
- 学习要点：确定性状态机、去重幂等、快照与日志截断。
- 推荐阅读：SMR 教程、Log-structured storage。
- 本仓库实验：实验 2（与 Saga 结合）。
- 关联测试：`tests/storage_snapshot.rs`（规划）。
- 评估方式：恢复后终态一致与重复应用无副作用。

## 6) 事务与补偿（Transactions & Saga）

- 课程对标：CMU 分布式事务；Berkeley Sagas/2PC/3PC；Cornell Fault-tolerant transactions。
- Wikipedia：Two-phase commit、Three-phase commit、Saga pattern。
- 代码映射：`src/transactions.rs`（`Saga`/`SagaStep`）。
- 学习要点：严格事务 vs Saga 补偿；幂等补偿步骤；跨分区写入策略。
- 推荐阅读：Sagas（Hector & Garcia-Molina）。
- 本仓库实验：实验 2；M5 端到端样例。
- 关联测试：`tests/saga_compensation.rs`（规划）。
- 评估方式：不变式保持与补偿幂等性覆盖。

## 7) 传输与调度（Transport & Scheduling）

- 课程对标：CMU RPC/Time；Berkeley Async I/O；MIT 心跳与时钟。
- Wikipedia：RPC、Logical clock、Failure detectors、Timeouts。
- 代码映射：`src/transport.rs`、`src/scheduling.rs`（`LogicalClock`、`TimerService`）。
- 学习要点：超时与重试、逻辑时钟对顺序与重放的影响。
- 推荐阅读：RPC 语义、异步 I/O 模型。
- 本仓库实验：实验 4（心跳/探测）、5（基准中的重试开销）。
- 关联测试：`tests/retry*.rs`、`tests/pipeline.rs`（规划）。
- 评估方式：重试下的幂等保证与尾延迟控制。

## 8) 故障与健康（SWIM 与反熵）

- 课程对标：CMU Gossip；Berkeley Epidemic algorithms；Cornell Probabilistic failure detectors。
- Wikipedia：SWIM、Gossip-style membership、Anti-entropy。
- 代码映射：`src/swim.rs`（`SwimMemberState`、`SwimEvent`、`SwimTransport`）。
- 学习要点：传播因子、可达性、谣言终止时间上界。
- 推荐阅读：SWIM 原始论文；Lifeguard。
- 本仓库实验：实验 4。
- 关联测试：`tests/swim_*.rs`。
- 评估方式：误报率-超时曲线与收敛时间分布。

## 学习路径建议（与本仓库实验结合）

1. 阅读 `membership`/`swim` 并实现基本心跳与传播测试。
2. 使用 `topology` 构建一致性哈希环，模拟扩缩容再均衡。
3. 在 `replication` 上实现不同 `ConsistencyLevel` 写入策略与 ACK 统计。
4. 打通 `storage` 幂等存储，结合 `transactions::Saga` 编写两步业务补偿 Demo。
5. 若开启 `consensus-raft` 特性，完成简化日志复制与 Leader 切换实验。

## 关联导航（范式与基准）

- 范式索引：Actor 模型 [`../../../rust-formal-engineering-system/02_programming_paradigms/09_actor_model/00_index.md`](../../../rust-formal-engineering-system/02_programming_paradigms/09_actor_model/00_index.md)
- 范式索引：事件驱动 [`../../../rust-formal-engineering-system/02_programming_paradigms/08_event_driven/00_index.md`](../../../rust-formal-engineering-system/02_programming_paradigms/08_event_driven/00_index.md)
- 最小基准指南：[`../../../rust-formal-engineering-system/02_programming_paradigms/11_benchmark_minimal_guide.md`](../../../rust-formal-engineering-system/02_programming_paradigms/11_benchmark_minimal_guide.md)

## 联动与命令

- 文档：`WIKI_MAPPING.md`、`CONCEPT_MODEL.md`、`EXPERIMENT_GUIDE.md`、`ROADMAP.md`
- 命令：
  - 测试：`cargo test -p c20_distributed -- --nocapture`
  - 示例：`cargo run -p c20_distributed --example e2e_saga`
  - 基准：`cargo bench -p c20_distributed`

---

## 课程-模块-代码快速对照表

```text
MIT 6.824 / 6.5840
  - Raft Labs                -> consensus（`consensus-raft` 特性），tests/raft_minimal.rs（规划）
  - KV/SMR                   -> storage（状态机/快照），replication（提交推进）
  - Sharding/ShardKV         -> topology/partitioning（一致性哈希）

CMU 15-440
  - Gossip/Membership        -> swim、membership（故障探测与成员传播）
  - Consistency/Quorum       -> consistency、replication（R/W 法定人数）

Berkeley CS262A
  - CAP/PACELC               -> consistency（模型与工程取舍）
  - Scalable storage         -> topology（再均衡与倾斜控制）

Cornell CS7410
  - Paxos/VRR                -> consensus（对比部分），replication（多数派）
```

## 代码入口与示例清单

- 示例：
  - `examples/e2e_saga.rs`：两步事务与补偿、幂等验证
  - `examples/e2e_replication.rs`：法定人数与提交推进最小演示
  - `examples/e2e_load_balancer_min.rs`：基础负载均衡与拓扑联动
  - `examples/e2e_discovery_lb_config.rs`：服务发现 + LB 配置热更新
  - `examples/e2e_chaos_min.rs`：注入失败/超时的健壮性观测

- 测试（部分/规划）：
  - `tests/replication_local.rs`：本地复制/一致性级别
  - `tests/hashring_properties.rs`：一致性哈希性质
  - `tests/swim_*.rs`：SWIM 探测
  - `tests/raft_minimal.rs`、`tests/consistency_matrix.rs`：规划草案

## 学习与项目路径（建议）

1. 从 `swim`/`membership` 入手掌握故障探测与成员协议
2. 构建 `topology` 的一致性哈希环并做扩缩容实验
3. 在 `replication` 实现 `ConsistencyLevel` 写入策略与 ACK 表
4. 接入 `storage` 幂等存储，结合 `transactions::Saga` 完成端到端补偿样例
5. 可选：启用 `consensus-raft`，实现最小 Raft 选举与日志复制

## 检索与参考

- 文档：`EXPERIMENT_GUIDE.md`、`CONCEPT_MODEL.md`、`WIKI_MAPPING.md`
- 资料：MIT/CMU/Berkeley/Cornell/Stanford 课程主页与 Wikipedia 主题词条

## 参考资料（建议检索）

- MIT 6.824/6.5840: Distributed Systems
- CMU 15-440, 15-745: Distributed Systems
- Berkeley CS262A: Advanced Topics in Computer Systems
- Stanford CS244B: Distributed Systems
- Cornell CS7410: Distributed Computing Principles
- Wikipedia：Raft, Paxos, Consistent hashing, SWIM, Saga, Linearizability 等词条

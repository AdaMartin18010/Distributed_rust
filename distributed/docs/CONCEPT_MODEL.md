# 概念-关系-属性（CRA）模型：分布式系统核心

目标：以课程/工业标准对标的方式，统一“概念-关系-属性-约束-可证性”结构，支撑后续形式化与验证。

核心概念（节选）：

- 节点（Node）：属性：`id`、`role`、`addr`、`health`；关系：参与 `Cluster`、拥有 `Replica`。
- 集群（Cluster）：属性：`members`、`topology`；关系：包含 `Shard`、`Partition`、`ConsensusGroup`。
- 视图（View）：属性：`epoch/term`、`membership-set`；关系：约束复制与仲裁规则。
- 分片（Shard/Partition）：属性：`shard_id`、`replication_factor`；关系：映射到 `nodes`（一致性哈希）。
- 法定人数（Quorum）：属性：`size`、`policy`；关系：绑定 `ConsistencyLevel`。
- 一致性（ConsistencyLevel）：取值：Strong/Quorum/Eventual；约束：`required_acks(total)`。
- 共识组（ConsensusGroup）：属性：`leader`、`term`、`log`；关系：复制到 `followers`。
- 事务（Transaction/Saga）：属性：`steps`、`compensation`；关系：跨 `Shard` 的写集合。
- 存储（StateMachine/Log）：属性：`deterministic`、`idempotency`；关系：被 `replication` 驱动。

约束与不变量（示意）：

- Quorum 安全性：`ack_count >= required_acks(N, level)` ⇒ 提交可见。
- 线性一致需要：单领导者、日志前缀匹配、单调任期；读需防陈旧（lease/读屏障）。
- Saga 可补偿性：每一步存在幂等 `compensate`；部分失败不破坏存储不变量。

代码落点：

- `topology::ConsistentHashRing`（分片映射）；`replication::{QuorumPolicy, MajorityQuorum}`（仲裁）；
- `transactions::{Saga, SagaStep}`（补偿编排）；`storage::{StateMachineStorage, IdempotencyStore}`（确定性/幂等）。

后续将为每个概念补充：形式化定义（数学/逻辑）、可验证性质、测试/实验模版。

## 扩展形式化定义（补全）

- 领导者选举安全性：同一任期内最多一个领导者。
  - 约束：\(\forall t, \; |\{ n \mid leader(n, t) \}| \le 1\)。
  - 可证性：由多数投票与单调任期保证。

- 日志前缀匹配：若条目 \(i\) 在某节点提交，则所有提交日志在 \(i\) 之前的前缀一致。
  - 约束：\(\forall i < j, \; committed(i) \Rightarrow \text{prefix}(log, i) = \text{prefix}(log', i)\)。

- 读一致性（读屏障）：读取视图不早于最近一次写提交视图。
  - 约束：\(read(view) \ge commit\_view(last\_write)\)。

### 检查点扩展

- 领导者唯一性：对同一任期收集所有 `LeaderElected(term, node)` 事件，断言不超过 1。
- 前缀匹配：模拟不同复制延迟，验证 `append` 后 `commit_index` 单调且前缀一致。
- 读屏障：在弱一致读与强一致读下对比可见历史差异，输出违反样本。

## 形式化定义（示例）

- Quorum：令节点集合 \(V\)，法定人数族 \(\mathcal{Q} = \{Q \subseteq V : |Q| > |V|/2\}\)。性质：\(\forall Q_i, Q_j \in \mathcal{Q}, Q_i \cap Q_j \neq \emptyset\)。
- 线性一致：存在映射 \(h\) 将操作历史嵌入全序，满足实时先后与顺序语义。
- 一致性哈希：令哈希环区间分布近似均匀，虚拟节点数 \(R\) 提升负载平衡性，E[迁移比例]≈\(1/|V|\)。

## 可验证检查点

- 多数派：构造不同 N、Level 的 ACK 表，验证写后读的可见性与单调性。
- 哈希环：扩容/缩容实验记录键迁移比例与倾斜度（P95/P99）。
- Saga：注入中间失败与重复补偿，验证不变式保持与幂等性。

## 示例导航

- 复制与一致性：`cargo test -p c20_distributed --test replication_local`
- 哈希与再均衡：`cargo test -p c20_distributed --test hashring_properties`
- Saga 补偿：`cargo test -p c20_distributed --test saga` 或 `cargo run -p c20_distributed --example e2e_saga`

## 🔗 快速导航

- 模型理论：`../../formal_rust/language/18_model/01_model_theory.md`
- AI系统：`../c19_ai/docs/FAQ.md`
- WebAssembly：`../../formal_rust/language/16_webassembly/FAQ.md`
- IoT系统：`../../formal_rust/language/17_iot/FAQ.md`
- 区块链：`../../formal_rust/language/15_blockchain/FAQ.md`

## 关联导航

- 范式索引：Actor 模型（消息驱动/监督）[`../../../rust-formal-engineering-system/02_programming_paradigms/09_actor_model/00_index.md`](../../../rust-formal-engineering-system/02_programming_paradigms/09_actor_model/00_index.md)
- 范式索引：事件驱动（事件循环/总线/溯源）[`../../../rust-formal-engineering-system/02_programming_paradigms/08_event_driven/00_index.md`](../../../rust-formal-engineering-system/02_programming_paradigms/08_event_driven/00_index.md)

## 联动与命令

- 文档：`WIKI_MAPPING.md`、`COURSE_ALIGNMENT.md`、`EXPERIMENT_GUIDE.md`、`ROADMAP.md`
- 命令：
  - 测试：`cargo test -p c20_distributed -- --nocapture`
  - 示例：`cargo run -p c20_distributed --example e2e_saga`
  - 基准：`cargo bench -p c20_distributed`

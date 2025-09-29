# 文档索引（Index）

快速导航各专题与关键 API 锚点。

## 专题导航

- 共识：`./consensus/README.md`
- 一致性：`./consistency/README.md`
- 复制与放置：`./replication/README.md`、`./topology/README.md`
- 成员与故障探测：`./membership/README.md`、`./failure/README.md`
- 存储：`./storage/README.md`
- 传输：`./transport/README.md`
- 事务：`./transactions/README.md`
- 时间与时钟：`./time/README.md`
- 可观测性：`./observability/README.md`
- 性能优化：`./performance/OPTIMIZATION.md`
- 测试与实验：`./testing/README.md`、`./experiments/README.md`

## 关键不变量摘要

- 共识：领导者唯一、任期单调、日志前缀匹配、提交单调。
- 复制：R+W>N 线性读、W>⌊N/2⌋ 唯一提交、读屏障与租约降级。
- 存储：WAL 追加性、快照恢复等价、原子落盘、`last_applied ≤ commit_index`。
- 时间：租约安全 `now ≤ last_hb + L − ε`、TrueTime 外部一致性。
- 成员：incarnation/version 单调合并、gossip 期望 O(log n) 收敛。
- 拓扑：一致性哈希副本唯一、路由稳定、扩缩容迁移比例 ~1/N。
- 传输：共享 deadline、仅幂等操作自动重试、背压/拒绝保护尾延迟。
- 事务：SAGA 补偿闭包与幂等、2PC 原子提交与无分歧。
- 一致性：Linearizable ⊇ Sequential ⊇ Causal ⊇ Session/Monotonic ⊇ Eventual（非严格全序）。

## 关键 API 锚点（源码）

- 共识（Raft 最小实现）：`src/consensus/raft.rs`
- RPC 传输与重试：`src/transport.rs`、`src/network/mod.rs`
- 复制与仲裁：`src/replication.rs`、`src/storage/replication.rs`
- 拓扑/一致性哈希：`src/topology.rs`
- 存储抽象：`src/storage/mod.rs`
- 调度/定时：`src/scheduling.rs`
- 一致性工具：`src/consistency/mod.rs`

## 测试索引（不变量导向）

- Raft 读路径与屏障：`tests/raft_read.rs`
- Raft 日志前缀与匹配：`tests/raft_log.rs`
- Raft 任期/角色单调：`tests/raft_state.rs`
- Raft 快照与截断：`tests/raft_snapshot.rs`
- Raft 提交推进：`tests/raft_commit.rs`
- 复制与幂等：`tests/pipeline.rs`
- 重试与退避：`tests/retry_backoff.rs`

## 贡献与风格

- 贡献指南：`../../CONTRIBUTING.md`
- 风格规范：`./STYLE_GUIDE.md`

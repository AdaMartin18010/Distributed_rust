# 存储（Storage）

- 状态机复制、日志与快照、WAL 与恢复
- 接口：`StateMachineStorage`, `LogStorage`

## 快照（Snapshot）

- 接口：`SnapshotStorage`，示例实现：`InMemorySnapshot<T>`
- Raft 中的快照可用于截断日志、加速恢复

## 日志/WAL 不变式

- 追加性：日志仅在末尾追加；快照后允许丢弃被包含的前缀。
- 单调性：`last_applied ≤ commit_index ≤ last_log_index` 且单调不减。
- 匹配性：相同 `(index, term)` 的条目内容必须一致。

## 截断与恢复流程

1) 创建快照：冻结 `last_applied` 前的状态机快照，记录元信息（最后索引与任期）。
2) 截断日志：安全地删除已包含在快照中的前缀日志。
3) 恢复：先加载最新快照，然后重放快照之后的日志至 `commit_index`。

## 接口示例

```rust
use c20_distributed::storage::{StateMachineStorage, LogStorage};

fn apply_entry<S: StateMachineStorage>(sm: &mut S, entry: &[u8]) {
    sm.apply(entry).expect("apply");
}
```

## 与共识/复制的关系

- `consensus` 推进 `commit_index`；`storage` 负责持久化与回放，确保崩溃恢复后一致。
- `replication` 在副本间传输日志/快照，落盘策略需保证提交前后语义一致。

## 持久化策略与故障恢复

- 写前日志（WAL）：`fsync`/`fdatasync` 时机决定崩溃后一致性保障与尾延迟。
- 日志分段与校验：以段为单位滚动与重放；CRC/校验和确保损坏可检测。
- 快照压缩：定期或基于阈值触发；考虑快照期间并发写入的一致性（copy-on-write）。

## 进一步阅读

- Wiki：`Write-ahead logging`, `Log-structured storage`, `Copy-on-write`
- 课程：MIT 6.824（Lab：Raft snapshots）、CMU 15-445（日志与恢复）
- 论文/实践：Raft 论文附录（快照）、LSM-Tree、RocksDB/Peacock/Bitcask 设计文档

## 练习与思考

1. 实现一个完整的WAL系统，支持日志分段、校验和验证以及崩溃恢复。
2. 设计一个快照管理系统，能够自动创建快照、压缩存储并支持增量快照。
3. 构建一个存储引擎抽象层，支持多种底层存储后端（RocksDB、LSM-Tree等）。
4. 开发一个存储性能监控工具，能够分析I/O模式、检测热点数据并提供优化建议。

## 快速导航

- 分布式系统总纲：`../README.md`
- 共识机制：`../consensus/README.md`
- 复制机制：`../replication/README.md`
- 故障处理：`../failure/README.md`

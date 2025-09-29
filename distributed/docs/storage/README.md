# 存储（Storage）

> 关键不变量：WAL 追加性、快照恢复等价、原子落盘、`last_applied ≤ commit_index`。

- 状态机复制、日志与快照、WAL 与恢复
- 接口：`StateMachineStorage`, `LogStorage`

## 快照（Snapshot）

- 接口：`SnapshotStorage`，示例实现：`InMemorySnapshot<T>`
- Raft 中的快照可用于截断日志、加速恢复

## 日志/WAL 不变式

- 追加性：日志仅在末尾追加；快照后允许丢弃被包含的前缀。
- 单调性：`last_applied ≤ commit_index ≤ last_log_index` 且单调不减。
- 匹配性：相同 `(index, term)` 的条目内容必须一致。

形式化要点：

- 崩溃后恢复等价：执行“加载最新快照→回放日志至 commit_index”的序列，与崩溃前对外可见最终状态等价。
- 原子落盘：快照与段切换采用“写入临时文件→校验→原子替换”，避免部分写导致的撕裂状态。

### 状态机确定性约束

- 纯函数化 `apply(entry)`: 对同一输入与前置状态，结果必须唯一确定；禁止读取外部非确定源（墙钟/随机）。
- 顺序一致：按提交顺序应用；禁止跨条目的副作用依赖未提交数据。
- 幂等语义：在崩溃恢复或重放时重复 `apply` 不得改变最终状态（需结合上层幂等键或日志去重）。

## 截断与恢复流程

1) 创建快照：冻结 `last_applied` 前的状态机快照，记录元信息（最后索引与任期）。
2) 截断日志：安全地删除已包含在快照中的前缀日志。
3) 恢复：先加载最新快照，然后重放快照之后的日志至 `commit_index`。

读屏障关联：

- 在恢复完成、`commit_index` 与 `last_applied` 对齐之前，读路径需阻塞或降级，避免返回中间态。

### 崩溃一致性 checklist

- 启动顺序：先恢复快照→校验日志段校验和→自 `snapshot.last_index+1` 开始回放至 `commit_index`。
- 原子性：快照写入采用临时文件+原子替换；日志段 rollover 写入尾部校验并 `fsync`。
- 进度点：在 `commit_index` 增长时落盘持久化，以防恢复时回退提交。
- 校验：每段包含起止索引、任期与 CRC；不匹配时回滚到最后一致位置并触发日志修复。

## 接口示例

```rust
use distributed::storage::{StateMachineStorage, LogStorage};

fn apply_entry<S: StateMachineStorage>(sm: &mut S, entry: &[u8]) {
    sm.apply(entry).expect("apply");
}
```

## 幂等存储接口最佳实践

- 键设计：`(namespace, key, idempotency_key)` 复合键，区分业务空间与请求幂等键。
- 读写路径：写入前先查询幂等记录；命中则返回上次结果，未命中则占位执行，成功后写回结果。
- TTL 策略：为幂等记录设置合适 TTL（与最大重试窗口一致），避免无限增长。
- 一致性：在 `Quorum` 写下，幂等记录与业务状态应在同等或更强持久性级别存储。

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

# 共识（Consensus）

- 覆盖：Raft、Paxos、Multi-Paxos、EPaxos、Viewstamped Replication
- 课程参考：MIT 6.824 Lab2/3、CMU 15-440、Berkeley CS262A
- 接口：`ConsensusApi`、`ConsensusRole`
- 实践要点：日志复制、领导者选举、提交/应用、快照

## 安全性（Safety）与活性（Liveness）

- 安全性：不产生相互矛盾的提交。Raft 依靠多数派日志匹配与领导者限制（leader completeness）保证。
- 活性：在网络稳定的情况下最终能前进。通过随机化选举超时与心跳机制降低分歧与冲突选举。
- 不变式：
  - 提交前条件：条目必须存储于多数派上且位于领导者任期内。
  - 应用前条件：应用层按 `commit_index` 单调推进；应用是确定性的状态机。

## 选举超时与心跳

- 心跳周期 T；跟随者选举超时 E 从区间 [a, b] 均匀抽样，一般 b≥3T。
- E 的随机化避免同时竞选；失败领导者退避，减少日志冲突。
- 与 `scheduling::TimerService` 协同：心跳使用周期定时，选举使用一次性随机超时。

## Raft（feature: consensus-raft）

提供最小 Raft 接口：

- `RaftState`、`Term`、`LogIndex`
- `AppendEntriesReq/Resp`、`RequestVoteReq/Resp`
- `RaftNode` trait

建议实验路线（对齐 MIT 6.824）：

1. 领导者选举与心跳
2. 日志复制与提交
3. 快照与恢复

### 日志匹配与提交

- 领导者在 `AppendEntries` 中携带 `(prev_log_index, prev_log_term)` 以进行匹配；冲突时回退并重试。
- 提交规则：领导者任期内的条目在多数派复制后即可推进 `commit_index`；应用顺序与日志顺序一致。

### 快照与截断

- 当日志过长时，创建快照并允许截断已有日志以降低恢复成本。
- 发送安装快照 RPC（若实现提供）给进度落后的跟随者。

最小示例（特性启用）：

```toml
[dev-dependencies]
c20_distributed = { version = "0.1", features = ["consensus-raft"] }
```

```rust
use c20_distributed::consensus_raft::{MinimalRaft, RaftNode, AppendEntriesReq, Term, LogIndex};
let mut r: MinimalRaft<Vec<u8>> = MinimalRaft::new();
let req = AppendEntriesReq { term: Term(1), leader_id: "n1".into(), prev_log_index: LogIndex(0), prev_log_term: Term(0), entries: vec![b"a".to_vec()], leader_commit: LogIndex(0) };
let _resp = r.handle_append_entries(req).unwrap();
```

## Paxos / Multi-Paxos / EPaxos 对比

- Paxos：两阶段（Prepare/Accept），保证单值共识；工程中常用 Multi-Paxos 复用领导者降低开销。
- Multi-Paxos：稳定领导者后只需 Accept 阶段；与 Raft 在工程可读性与领导者限制上有所取舍。
- EPaxos：无固定领导者，基于依赖关系和提交图并行提交，降低延迟但实现更复杂。

工程取舍：

- Raft 强调易实现与日志一致性；Paxos 家族理论简洁但工程细节多；EPaxos 适合跨地域低延迟但复杂度与冲突处理成本高。

## 实验指引（对齐课程）

1) 选举与心跳：验证领导者唯一性与故障切换时间；注入网络分区与时钟抖动。
2) 日志复制：构造冲突场景，验证回退策略与提交前置条件。
3) 快照与恢复：制造长日志，测量恢复时间；验证快照一致性与截断正确性。
4) 线性化读：实现 `read_index`/`lease read` 并在分区与时钟偏差下验证安全。

### 线性化读实现指引（read_index / lease read）

- read_index：领导者向多数派确认自身仍为领导者并获取当前 `commit_index`，随后在不附加日志的情况下提供只读；开销是一次多数派往返。
- lease read：基于时间租约，领导者在租约期内直接提供只读；需要安全时钟界（见 `time/`），防止时钟回拨导致过期租约误用。
- 推荐做法：在网络稳定期使用 lease 以降低延迟；检测到时钟异常或分区风险时降级为 read_index。

## 进一步阅读

- Wiki：`Paxos`, `Raft`, `EPaxos`, `Viewstamped Replication`
- 课程：MIT 6.824（Labs 2/3）、Berkeley CS262A 共识模块
- 论文：Paxos Made Simple、Raft：In Search of an Understandable Consensus、EPaxos、VR Revisited

## 练习与思考

1. 实现一个完整的Raft共识算法，包括领导者选举、日志复制和快照功能。
2. 设计一个支持网络分区的共识协议，在分区恢复后能够自动合并状态。
3. 构建一个多主复制系统，使用EPaxos算法实现低延迟的并行提交。
4. 开发一个共识性能测试框架，测量不同网络条件下的延迟和吞吐量。

## 快速导航

- 分布式系统总纲：`../README.md`
- 一致性模型：`../consistency/README.md`
- 复制机制：`../replication/README.md`
- 故障处理：`../failure/README.md`

---

## API 映射与代码定位

- 核心抽象：`src/consensus.rs`（`ConsensusApi`、`ConsensusRole`、`LogEntry`、`CommitIndex`）
- Raft 最小实现（条件：feature `consensus-raft`）：`src/consensus_raft/*.rs`
- 相关支撑：
  - 定时/时钟：`src/scheduling.rs`（`TimerService`、`LogicalClock`）
  - 存储：`src/storage.rs`（`LogStorage`、`StateMachineStorage`、快照）
  - 传输：`src/transport.rs`（RPC/消息抽象）

快速导航到常用符号：

```text
RaftNode::handle_append_entries        -> src/consensus_raft/append.rs
RaftNode::handle_request_vote          -> src/consensus_raft/vote.rs
Leader 复制与提交推进（commit_index）  -> src/consensus_raft/leader.rs
快照/截断（如启用）                   -> src/storage.rs / src/consensus_raft/snapshot.rs
```

## 如何运行与测试

- 运行最小示例：

```powershell
cargo run -p c20_distributed --example e2e_replication
# 如开启 Raft：
cargo test -p c20_distributed --features consensus-raft --test raft_minimal -- --nocapture
```

- 推荐日志与随机种子：

```powershell
$env:RUST_LOG="info,c20_distributed=debug"; $env:RUST_BACKTRACE=1
cargo test -p c20_distributed --features consensus-raft -- --nocapture
```

- 性能/功能基线：
  - 领导者选举收敛时间（稳定网络）：< 2× 选举超时上界
  - 日志冲突回退后重同步成功率：100%（在无丢包/超时场景）
  - 快照恢复时间：随快照大小线性增长；恢复后 `commit_index` 与应用状态一致

## 常见问题与排错（FAQ/Troubleshooting）

- 现象：频繁双领导者（split brain）
  - 排查：选举超时区间是否过窄；心跳周期是否过大导致误判；时钟抖动/线程阻塞
  - 建议：`E in [300ms, 900ms]` 且 `T≈100ms` 起步，确保不同节点 E 不同步

- 现象：日志复制卡住，`commit_index` 不前进
  - 排查：多数派是否可达；`prev_log_index/term` 回退逻辑是否正确；是否错误地跨任期提交
  - 建议：开启 `trace` 日志，记录每次冲突回退的目标 index/term

- 现象：读取到了旧值
  - 排查：是否实现了 `read_index` 或安全的租约读；领导者身份是否在读时仍然有效
  - 建议：在不安全时钟环境下降级为 `read_index`

## 配置与特性开关

- `features = ["consensus-raft"]`：启用最小 Raft 接口与实现
- 关键参数：
  - `heartbeat_interval_ms`（心跳周期）
  - `election_timeout_range_ms = [min, max]`（随机选举超时区间）
  - `install_snapshot_threshold`（触发快照/截断阈值，若实现）

## 示例：线性化读（Read Index）骨架

```rust
use c20_distributed::consensus_raft::{MinimalRaft, RaftNode};

fn linearizable_read<R: RaftNode>(raft: &mut R) -> anyhow::Result<Vec<u8>> {
    // 1) 多数派心跳确认领导者仍然有效并获取 commit_index
    let read_barrier = raft.read_index()?; // 假设接口存在于最小实现中
    // 2) 等待本地应用层追至 read_barrier
    raft.wait_applied(read_barrier)?;
    // 3) 安全读取
    Ok(raft.read_state()?)
}
```

## 进一步实验建议

- 分区注入：将集群划分为两半，验证仅多数派侧可前进；合并后检查日志冲突解决与单调性
- 时钟异常：模拟时钟回拨/停顿，验证租约读的降级路径
- 快照鲁棒性：在截断点附近反复崩溃恢复，确保状态机终态一致

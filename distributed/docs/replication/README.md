# 复制与放置（Replication & Placement）

> 关键不变量：R+W>N 线性读、W>⌊N/2⌋ 唯一提交、读屏障/租约安全（不满足降级）。

- 主从、多主（leaderless/last-writer-wins）、链式复制、只读副本
- 放置：一致性哈希/分片、复制因子、跨机架/跨机房拓扑感知

## 目录

- [复制与放置（Replication \& Placement）](#复制与放置replication--placement)
  - [目录](#目录)
  - [策略对比](#策略对比)
  - [Quorum R/W](#quorum-rw)
    - [required\_acks(N, Level)](#required_acksn-level)
  - [放置与路由](#放置与路由)
  - [读路径优化](#读路径优化)
  - [进一步阅读](#进一步阅读)
  - [复制（Replication）](#复制replication)
  - [Quorum 策略](#quorum-策略)
    - [Quorum 公式](#quorum-公式)
  - [副本放置与幂等](#副本放置与幂等)
    - [放置与去重](#放置与去重)
    - [幂等执行](#幂等执行)
  - [写入路径伪代码与失败重试](#写入路径伪代码与失败重试)
  - [提交推进与读屏障（时序示意）](#提交推进与读屏障时序示意)
  - [常见问题（FAQ）](#常见问题faq)
  - [练习与思考](#练习与思考)
  - [快速导航](#快速导航)

## 策略对比

- 主从：强一致读（读主）与较低写放大；故障切换需要选主与日志追赶。
- 多主：写入可在任意副本受理；需要冲突解决（LWW/CRDT/应用级合并）。
- 链式复制：写沿链路传播，读取链尾；可简化一致性证明但牺牲部分可用性。
- 只读副本：异步复制，服务只读流量；读修复与过期读的权衡。

## Quorum R/W

- N 副本，配置读写集大小 R/W：
- 若 `R + W > N`，线性一致读；若 `W > N/2`，可保证提交的唯一性。
- 动态调参：在高延迟/分区时降低 R 或 W 以提升可用性（与业务 SLA 协调）。

形式化要点（草图）：

- 交叠性：设多数派大小 `Q=⌊N/2⌋+1`，任意两多数派集合必有交集，因此一旦某写在多数派上被确认，任何线性化读（R≥Q 且 R+W>N）都能交叠到该写。
- 提交唯一性：若 W> N/2，则不存在两个不同写同时达成提交条件且彼此不可见（因其各自写集合交叠）。

### required_acks(N, Level)

| Level     | 写入 required_acks (W)        | 读取 required_reads (R)        | 备注 |
|-----------|-------------------------------|--------------------------------|------|
| Strong    | ⌈N/2⌉+1                       | ⌈N/2⌉+1                        | 满足 R+W>N 且 W> N/2 |
| Quorum    | ⌈N/2⌉+1                       | 1 或 ⌈N/2⌉+1                   | 写入采用多数；读取可按 SLA 调整 |
| Eventual  | 1                             | 1                              | 最终一致，读修复/反熵收敛 |

计算函数示例：`required_acks(N, Strong|Quorum|Eventual)`。

## 放置与路由

- 使用 `topology::ConsistentHashRing` 选择 N 个节点；跨机架/机房做 `failure domain` 分散。
- 热点键的副本扩散：为热点键提高复制因子或启用读缓存。

## 读路径优化

- 主从：`read_index`/`lease read`；只读副本配合版本戳与读修复。
- 多主：版本向量/因果 token 保证不回退；读修复在后台反熵。

读屏障与不变量：

- read-index（或等价屏障）保证读取的提交点不早于上一次确认的写入提交点。
- 租约读需满足 `time/README.md` 的 ε 界条件；不满足时降级为多数派读。

## 进一步阅读

- Wiki：`Data replication`、`Quorum (distributed computing)`、`Chain replication`
- 课程：MIT 6.824（Replication）、UW CSE452（Replication & Consistency）
- 论文：Dynamo、Cassandra、Chain Replication、Spanner、Raft（状态机复制）

## 复制（Replication）

- 主从/多主、日志复制、复制因子与放置策略
- 课程参考：MIT 6.824、Berkeley CS262A

## Quorum 策略

- 接口：`Replicator`、`QuorumPolicy`
- 内置：`MajorityQuorum`，对 Strong/Quorum 取多数 ack，对 Eventual 仅需 1 ack

### Quorum 公式

- 写入成功需要 `W` 个副本 ack；读取需要 `R` 个副本。
- 线性一致性必要条件：`R + W > N` 且 `W > N/2`（常见设置 R=1/W=N，或 R=Quorum/W=Quorum）。
- 可调一致性：`Eventual` 采用 `W=1`，读通常 `R=1`；更强可用性但牺牲读写冲突下的强一致。

## 副本放置与幂等

- 拓扑：使用 `ConsistentHashRing::nodes_for(key, N)` 选择 N 个副本节点（去重）
- 幂等：`IdempotencyStore` 与 `InMemoryIdempotency` 防止重复执行

### 放置与去重

- 在环上对 key 进行哈希，沿顺时针选择前 N 个不同节点。
- 网络分区/节点故障时，允许备用节点接管（虚节点数量越多，重平衡越平滑）。

### 幂等执行

- 在请求头或上下文中传入幂等键 `id`，由 `IdempotencyStore` 记录已执行结果或执行中状态。
- 重试与乱序到达时可避免重复副作用；与 `transport::RetryPolicy` 协同使用。

## 写入路径伪代码与失败重试

```rust
// 写入路径（简化）：放置→去重→并行写→收敛与确认
fn write_with_quorum(key: &Key, value: Bytes, level: ConsistencyLevel, idempotency_key: Option<Id>) -> Result<()> {
    let replicas = topology.nodes_for(key, N);
    if let Some(id) = idempotency_key {
        if let Some(prev) = idempotency_store.lookup(&id) { return Ok(prev); }
        idempotency_store.begin(&id);
    }
    let w = required_acks(N, level);
    let mut successes = 0;
    let mut attempts = 0;
    for r in replicas_parallel(replicas) {
        let result = transport.retry_with_jitter_deadline(|| r.append(value.clone()));
        if result.is_ok() { successes += 1; }
        attempts += 1;
        if successes >= w { break; }
    }
    if successes >= w {
        if let Some(id) = idempotency_key { idempotency_store.commit(&id); }
        return Ok(())
    }
    if let Some(id) = idempotency_key { idempotency_store.abort(&id); }
    Err(Error::NotEnoughAcks { got: successes, need: w, attempts })
}
```

语义要点：

- 幂等键贯穿同一请求的所有重试；服务端去重缓存返回上次成功结果。
- 退避采用带抖动的指数/等距策略；重试共享同一截止时间（deadline）。
- 只对被视为暂时性错误（超时/`Unavailable`）进行重试；幂等性不足的写需谨慎开启重试。

## 提交推进与读屏障（时序示意）

```text
Client         Leader/Replica A        Replica B        Replica C
  |  write(k,v,id)    |                   |                |
  |------------------>| append log        |                |
  |                   |----replicate----->| append ok      |
  |                   |----replicate---------------------->| append ok
  |                   | commit index→t                   |
  |                   | apply state @t                   |
  |<--ack (>=W)-------|                                   |
  | read(k)           |                                   |
  |------------------>| read-index/屏障至 commit>=t       |
  |<------------------| value@t                           |
```

读屏障选项：

- read-index/lease-read：确保读取不回退到 `t` 之前；租约失效或不确定时退回多数派读。
- 会话保证：同客户端会话内读在提交后不可回退。

## 常见问题（FAQ）

- ACK 抖动：副本延迟分布长尾导致 `W` 达标时间波动。建议：采用少量超额并发（hedged）或动态副本挑选，避免固定慢副本卡尾。
- 局部可用导致写放大：少数派分区内不断重试使无效流量放大。建议：基于拓扑与视图过滤不可达副本；对重试设置总预算与幂等去重。
- 尾延迟治理：使用带抖动退避、限制单次请求的最大重试次数与并发度；必要时降级到 Eventual 读取以保护整体 SLA。

## 练习与思考

1. 实现一个支持多种复制策略的分布式存储系统，包括主从、多主和链式复制。
2. 设计一个动态调整Quorum参数的机制，根据网络延迟和分区情况自动优化读写性能。
3. 构建一个副本放置优化器，考虑节点负载、网络拓扑和故障域分布。
4. 开发一个复制一致性验证工具，能够检测和修复副本间的数据不一致。

## 快速导航

- 分布式系统总纲：`../README.md`
- 一致性模型：`../consistency/README.md`
- 共识机制：`../consensus/README.md`
- 故障处理：`../failure/README.md`

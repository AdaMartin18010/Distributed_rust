# 复制与放置（Replication & Placement）

- 主从、多主（leaderless/last-writer-wins）、链式复制、只读副本
- 放置：一致性哈希/分片、复制因子、跨机架/跨机房拓扑感知

## 策略对比

- 主从：强一致读（读主）与较低写放大；故障切换需要选主与日志追赶。
- 多主：写入可在任意副本受理；需要冲突解决（LWW/CRDT/应用级合并）。
- 链式复制：写沿链路传播，读取链尾；可简化一致性证明但牺牲部分可用性。
- 只读副本：异步复制，服务只读流量；读修复与过期读的权衡。

## Quorum R/W

- N 副本，配置读写集大小 R/W：
- 若 `R + W > N`，线性一致读；若 `W > N/2`，可保证提交的唯一性。
- 动态调参：在高延迟/分区时降低 R 或 W 以提升可用性（与业务 SLA 协调）。

## 放置与路由

- 使用 `topology::ConsistentHashRing` 选择 N 个节点；跨机架/机房做 `failure domain` 分散。
- 热点键的副本扩散：为热点键提高复制因子或启用读缓存。

## 读路径优化

- 主从：`read_index`/`lease read`；只读副本配合版本戳与读修复。
- 多主：版本向量/因果 token 保证不回退；读修复在后台反熵。

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

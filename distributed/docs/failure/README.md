# 故障模型与容错（Failure Model & Fault Tolerance）

- Fail-Stop、Crash-Recovery、Network Partition、Byzantine
- FLP 不可能性、超时和随机化、Quorum 容错界

## 故障模型

- Fail-Stop：节点一旦故障即停止，不产生任意行为。
- Crash-Recovery：节点可能宕机并恢复，需 WAL/快照保证持久性与幂等恢复。
- 网络分区：消息可能丢失/乱序/重复/延迟；以超时近似失败检测。
- 拜占庭（Byzantine）：节点可能任意/恶意行为；需 BFT 协议（PBFT/Tendermint/HotStuff）。

## 不可能性与下界

- FLP：在完全异步系统中，不存在确定性的、在任意故障模式下保证终止的共识算法。
- 现实系统采用部分同步模型（随机化超时、心跳）以获得实践可用的活性。

## Quorum 与容错能力

- N 副本、最多 f 故障时：
  - 崩溃容错（CFT）：`N ≥ 2f + 1`（多数派）
  - 拜占庭容错（BFT）：`N ≥ 3f + 2`（HotStuff/Tendermint 常见界）

## 故障注入与恢复策略

- 注入：消息丢弃、重排、延迟、重复；节点崩溃/恢复；磁盘写后回滚；时钟跃迁。
- 恢复：先加载最新快照，回放日志至 `commit_index`；网络恢复后触发反熵同步。

## 与代码接口的对应

- `transport`：重试、超时、幂等键与去重缓存。
- `storage`：WAL、快照、恢复不变式。
- `consensus`：选举超时、心跳、日志匹配；BFT 需单独特性。

## 进一步阅读

- Wiki：`FLP impossibility`、`Byzantine fault`、`Failure detector`
- 课程：MIT 6.824（Fault Tolerance）、UWash CSE452（Failure & Time）
- 论文：PBFT、Tendermint、HotStuff、Viewstamped Replication、Raft

## 练习与思考

1. 实现一个故障检测器，能够区分网络分区、节点崩溃和网络延迟，并提供相应的恢复策略。
2. 设计一个拜占庭容错系统，支持动态节点加入和离开，并保证系统在恶意节点存在时的安全性。
3. 构建一个故障注入测试框架，能够模拟各种故障场景并验证系统的容错能力。
4. 开发一个故障恢复协调器，在故障发生后能够自动选择最优的恢复策略并执行。

## 快速导航

- 分布式系统总纲：`../README.md`
- 共识机制：`../consensus/README.md`
- 复制机制：`../replication/README.md`
- 存储系统：`../storage/README.md`

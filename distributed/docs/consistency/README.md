# 一致性（Consistency）

- CAP 与 PACELC、线性一致性、顺序一致性、因果一致性、最终一致性
- 接口：`ConsistencyLevel`
- 课程参考：MIT 6.824、Stanford CS244B、UWash CSE452、CMU 15-440

## CAP/PACELC 简述

- CAP：网络分区下，强一致性与可用性难以同时满足
- PACELC：无分区时在延迟与一致性之间取舍（EL vs EC）

### Wiki/课程对齐

- Wikipedia CAP/PACELC：定义、常见误解与工程解读（如可用性定义差异）。
- 课程作业通常通过引入分区与超时模拟 CAP 的权衡；在无分区实验中验证 PACELC 的延迟-一致性取舍。

## 与接口的映射

- `Strong`：线性一致/多数派提交
- `Quorum`：读写多数派（可配置读写比）
- `Eventual`：最终一致/反熵同步

接口到语义：

- 线性一致读：若 `R + W > N` 且写在领导者任期提交，读触达新值。
- 因果读：维护会话向量或因果 token；读需满足 `deps ⊆ store`。
- 最终一致：反熵收敛；冲突通过 LWW、CRDT 或应用级决策解决。

## 线性一致性与会话保证

- 线性一致性（Linearizability）：每个操作看似在某一时间点原子生效；需要全序点与多数派交集（参见 `replication` 的 R/W 条件）。
- 会话保证：读己之写、单调读、单调写、写后读等；可通过会话向量或会话 token 实现。

实现提示：

- 读路径可使用 `read_index`（Raft）或 `lease read`（带时钟偏差安全界）降低延迟。
- 会话 token 可编码最近写入的 `(term,index)` 或向量时间戳。

## 因果一致性与顺序一致性

- 因果：若 A→B（A 发生在 B 之前并影响 B），所有节点应观察到 A 在 B 之前；无需全局全序。
- 顺序一致性：所有进程看到相同的操作顺序，但该顺序未必与真实时间一致。

工程映射：

- 因果一致可用 `vector clock`/`DSV` 实现；跨分片场景可通过 `sticky session` 或 `causal broadcast`。
- 顺序一致常见于单一副本内的多线程内存模型，对分布式需借助全序广播或单主序列化。

## 实践映射

- `ConsistencyLevel::Strong`：启用多数派写入与读；在超时紧张时可回退到 `Quorum`。
- `ConsistencyLevel::Quorum`：读写分别独立配置 R/W；满足 `R+W>N` 时具线性一致读。
- `ConsistencyLevel::Eventual`：后台反熵（gossip/repair）；读写延迟最佳但允许暂时不一致。

## 进一步阅读

- Wiki：`CAP theorem`、`Consistency models`、`Linearizability`、`Causal consistency`
- 课程：MIT 6.824 Lectures on Replication/Consistency；UWash CSE452 Consistency
- 论文：Spanner（TrueTime 与外部一致性）、Dynamo/Cassandra（最终一致与反熵）、Bayou（会话与因果）

## 练习与思考

1. 实现一个支持多种一致性级别的分布式存储系统，包括线性一致性、因果一致性和最终一致性。
2. 设计一个向量时钟系统，用于跟踪分布式系统中的因果依赖关系。
3. 构建一个会话一致性保证机制，确保客户端能够读取到自己写入的数据。
4. 开发一个一致性模型验证工具，能够检测分布式系统中的一致性违规。

## 快速导航

- 分布式系统总纲：`../README.md`
- 共识机制：`../consensus/README.md`
- 复制机制：`../replication/README.md`
- 故障处理：`../failure/README.md`

---

## R/W 法定人数与语义矩阵

设副本数 \(N\)，读写分别取 \(R, W\)：

- 线性一致读的充分条件（基于法定人数）：\(R + W > N\) 且写入由单主在任期内提交
- 写写冲突避免：\(W > N/2\)
- 读延迟-一致性权衡：较小 \(R\) 减少延迟但更易读到旧值

常见配置：

```text
Strong  -> R = Majority, W = Majority
Quorum  -> R, W 可独立配置但满足 R + W > N
Eventual-> R = 1, W = 1（或任意），后台反熵收敛
```

## 读写路径示例（与 Raft 结合）

```rust
use c20_distributed::{consistency::ConsistencyLevel, replication::Replicator};

fn write_quorum(rep: &Replicator, key: &str, val: Vec<u8>) -> anyhow::Result<()> {
    rep.write(key, val, ConsistencyLevel::Quorum)?; // 内部根据 N 计算 required_acks
    Ok(())
}

fn read_strong(rep: &Replicator, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
    // 若结合 Raft，读路径建议使用 read_index 或租约读
    rep.read(key, ConsistencyLevel::Strong)
}
```

## 常见问题与排错

- 现象：Quorum 配置下仍读到旧值
  - 排查：是否满足 `R + W > N`；写入是否在任期内提交；读路径是否绕过了读屏障（read_index/lease）
  - 建议：将只读操作接入 `read_index` 或在租约无效时回退

- 现象：扩容后热点倾斜导致某些副本过载
  - 排查：一致性哈希虚拟节点数是否过小；是否存在热点键
  - 建议：增大虚拟节点数并结合读写分离/缓存

- 现象：Eventual 下长时间不收敛
  - 排查：gossip/repair 周期与反熵带宽限制；反熵对象是否过大
  - 建议：对大对象使用校验/分块修复；提升反熵并发度与频率

## 与测试/基准联动

- 测试：`tests/replication_local.rs`、`tests/consistency_matrix.rs`（规划）
- 基准：观察不同 N 与 Level 下 `required_acks` 导致的尾延迟差异

## 进一步实验建议

- Jepsen 思路：在分区/重连/时钟抖动下，验证 Strong/Quorum 的线性化读通过率
- 会话语义：注入跨会话因果依赖，验证会话向量/Token 的过滤正确性

# 成员管理与故障探测（Membership & Failure Detection）

> 关键不变量：incarnation/version 单调合并；gossip 期望 O(log n) 收敛；Suspect 超时转 Faulty。

- Gossip/SWIM、视图一致性、故障标记与传播
- 接口：`MembershipView`, `FailureDetector`

## SWIM 概述

- 周期性随机探测：选取目标节点 `ping`；若超时，使用中继节点 `ping-req`。
- 可疑标记（suspect）与确认（confirm）：通过 gossip 传播状态变化，降低误报与收敛延迟。
- 扩展：Lifeguard/Swim+（降低误报、适应长尾延迟）。

形式化要点（草图）：

- 版本/代数（incarnation/version）单调：较新事件覆盖较旧事件；合并采用“更高代、更高版本优先”。
- 收敛性：随机对等传播在期望 O(log n) 轮内覆盖全网；fanout 增大可降低常数项但提高带宽占用。

### 参数指南（建议）

- fanout：每轮传播的对端数量（建议 3~5）；增大可降收敛时间但放大带宽。
- 超时：直连 ping 超时 `Tping` 与间接 `Treq`，与集群延迟分布（P95/P99）匹配；建议 `Treq ≥ 2×Tping`。
- 间隔：探测周期 `Tcycle`；建议随机抖动避免同步。

### 误报率—超时曲线与收敛时间测量

- 方法：在可控延迟/丢包环境中扫描 `Tping` 与 `Treq`；统计误报率与收敛时间（Alive→Suspect→Confirm）。
- 结果：画出误报率-超时曲线，选择“可接受误报率”的最短收敛配置。

## 视图一致性与分布式路由

- 视图：节点→状态（Alive/Suspect/Dead/Left）与元数据（zone/rack/weight）。
- 与 `topology` 协作：仅在 Alive 集合上进行放置与路由；对 Suspect 采取降级读或多路读。

## 故障检测参数

- 超时与重试：依据 `scheduling` 的计时设施与全局延迟分布调参。
- 传播风暴抑制：整形 gossip 速率，避免大规模变更时的放大。

## 进一步阅读

- Wiki：`SWIM (consistency protocol)`, `Gossip protocol`
- 课程：MIT 6.824（Fault detection & Membership）
- 论文：SWIM、Lifeguard、Gossip-based failure detection

## 练习与思考

1. 基于 SWIM 实现成员探测模拟器：支持可疑/确认状态、间接探测与gossip传播，测量收敛时间与误报率。
2. 设计视图与拓扑联动：当某分区大量 Suspect 时，自动提升副本因子或触发多路读降级策略。
3. 在高延迟长尾环境中调参：对比默认SWIM、Lifeguard变体的误报率与收敛延迟。
4. 构建故障风暴抑制策略：在大规模状态变更时整形 gossip 速率，验证抖动对收敛的影响。

## 快速导航

- 分布式系统总纲：`../README.md`
- 故障处理：`../failure/README.md`
- 拓扑与放置：`../topology/README.md`
- 复制机制：`../replication/README.md`

## 成员管理（Membership）

- 覆盖：静态/动态成员、配置变更、故障探测（SWIM）
- 接口：`ClusterMembership`, `ClusterNodeId`
- 课程参考：MIT 6.824、CMU 15-440

## SWIM 故障探测

- 事件：`SwimEvent { node_id, state }`，状态：`Alive`/`Suspect`/`Faulty`
- 传输接口：`SwimTransport`；节点：`SwimNode<T: SwimTransport>`
- 实验建议：随机探测、间接探测（ping-req）、反熵传播

### 间接探测（ping-req）

- 当直接 ping 失败时，通过中继节点对目标进行间接探测；若任一中继确认可达，则视为 Alive。

### 反熵视图（Anti-Entropy View）

- 结构：`MembershipView { members: HashMap<NodeId, (state, version)> }`
- 接口：`local_update` 增版本、`gossip_payload` 生成传播数据、`merge_from` 基于版本合并

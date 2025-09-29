# 拓扑（Topology）

> 关键不变量：一致性哈希副本唯一、路由稳定；扩缩容迁移比例 ~1/N（期望）。

- Sharding、Consistent Hashing、Ring、Re-sharding
- 接口：`ShardId`, `ClusterTopology`

## Consistent Hashing

- 结构：`ConsistentHashRing { ring: BTreeMap<u64, String>, replicas }`
- 示例：

```rust
use distributed::topology::ConsistentHashRing;
let mut ring = ConsistentHashRing::new(50);
ring.add_node("n1");
ring.add_node("n2");
let owner = ring.route(&"order-1001").unwrap();
```

### 虚节点（Virtual Nodes）

- 每个物理节点对应多个虚节点（`replicas`）；将节点哈希多次放置在环上，平滑负载并降低重平衡抖动。
- 经验：`replicas` 取 50~200 之间，按规模与键分布调参。

#### 倾斜实验与图示（建议）

- 实验：固定节点数与键数，改变 `replicas`，测量每节点键数分布的 P95/P99；绘制随 `replicas` 变化的倾斜度曲线。
- 预期：`replicas` 增大，倾斜度下降且迁移更平滑，但路由内存与重建成本上升。

### 重平衡（Rebalance）与代价

- 加入/移除节点时，仅相邻区间的键会迁移（O(键/节点数)）。
- 实现要点：
  - 更新环后，增量地迁移受影响分片；对读取路径采用“双读”或“读修复”以降低短期不一致。
  - 与 `replication` 搭配时，先在新放置上写入副本并等待达标，再逐步淘汰旧副本。

#### 迁移比例推导（要点）

- 单节点加入：期望迁移比例约为 `1/(现有节点数+1)`；
- K 节点扩容：期望迁移比例近似 `K/(N+K)`；
- 变更代价与虚节点正相关，但可降低瞬时热点集中。

不变量（放置与副本唯一）：

- `nodes_for(key, n)` 在环上按顺时针取 n 个不同节点，确保副本唯一且顺序确定；去重避免同节点重复计数。
- 在相同环配置下，对同一 `key` 的路由稳定（deterministic），除非环发生变更。

### API 速览

- `ConsistentHashRing::new(replicas: usize)`：创建环。
- `add_node(node_id: &str)` / `remove_node(node_id: &str)`：增删节点。
- `route<K: Hash>(key: &K) -> Option<String>`：主节点路由。
- `nodes_for<K: Hash>(key: &K, n: usize) -> Vec<String>`：返回前 n 个节点，供复制使用。

### 故障域（Failure Domain）与放置

- 机架/机房感知：环节点携带 `rack/zone/region` 元数据，放置时尽量分散到不同故障域。
- 多层环：按区域构建上层环，区域内再构建子环，跨地域复制时优先选择不同区域。
- 失败时路由：在同域副本不可用时回退至跨域副本，结合 `consistency` 决定读的一致性级别。

### 热点迁移与倾斜（Skew）

- 热点键可通过：
  - 提高虚节点数（更细的切片）。
  - 对热点键启用副本扩散（仅对该键提升复制因子）。
  - 前缀分片：将热点前缀拆分为多个子分片。
- 监控：结合 `tracing` 与指标系统跟踪每节点 QPS/延迟并自适应调整。

#### 粘性路由（Session Affinity）与副作用

- 定义：在会话存续期将同一客户端/键路由至同一节点，减少跨节点状态搬迁与缓存未命中。
- 风险：在扩容/节点失衡时放大热点；需要与一致性哈希配合，使用 `client_id`/`session_id` 作为路由键并限制会话寿命。
- 建议：
  - 只对需要会话状态的读路径开启粘性；写路径仍按键路由，避免写放大。
  - 当监测到节点倾斜超过阈值时，逐步打破粘性（缩短 TTL、增加随机扰动），并触发后台迁移。

## 评估指标与命令

- 指标：
  - 键迁移比例（扩/缩容一次）：期望接近 1/N。
  - 倾斜度：P95/P99 分布，随 `replicas` 增大而下降。
  - 放置亲和：跨故障域分散度（zone/region 的去重率）。
- 实验命令：`cargo test -p distributed --test hashring_properties`

## 进一步阅读

- Wiki：`Consistent hashing`, `Sharding`
- 课程：MIT 6.824（Sharding & Rebalancing）
- 论文/实践：Dynamo、Cassandra（Vnode/Token Range）、TiDB Placement Rules、Alibaba CPS

## 与放置/路由接口

- `nodes_for(key, n)`：按环顺时针取 n 个不同节点，供 `replication` 使用。
- `route(key)`：返回主负责节点（在多主策略中为首选节点）。

## 实验入口与评估指标

- 运行：`cargo test -p distributed --test hashring_properties`
- 评估：键迁移比例接近 1/N；倾斜度（P95/P99）随 `replicas` 增大而下降；扩缩容期间尾延迟控制。

## 代码锚点

- `topology::ConsistentHashRing`
- 与 `replication` 的接口：基于 `nodes_for(key, n)` 的副本选择策略

## 练习与思考

1. 实现带虚节点的环并测量扩容/缩容时的键迁移比例与倾斜度（P95/P99）。
2. 设计跨故障域放置策略：在不同 `zone/region` 分散副本，验证在分区/故障下的可用性。
3. 为热点键实现副本扩散与前缀分片，对比不同方案的尾延迟与资源利用率。
4. 构建重平衡器：在节点加入/退出时增量迁移，并在读路径使用双读/读修复控制一致性。

## 快速导航

- 分布式系统总纲：`../README.md`
- 复制机制：`../replication/README.md`
- 一致性模型：`../consistency/README.md`
- 成员管理：`../membership/README.md`

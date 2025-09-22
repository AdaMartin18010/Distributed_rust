# c20_distributed — 分布式系统（Rust 1.89 对齐）

本 crate 聚焦分布式系统核心：成员管理、拓扑与分片、复制与一致性、共识抽象、事务与补偿、SWIM 故障检测与反熵、调度与逻辑时钟、存储与幂等。

## 快速导航

- 课程对标：`docs/COURSE_ALIGNMENT.md`
- Wiki 映射：`docs/WIKI_MAPPING.md`
- 概念模型（CRA）：`docs/CONCEPT_MODEL.md`
- 关键论证与公式：`docs/FORMAL_ARGUMENTS.md`
- 路线图：`docs/ROADMAP.md`
- 实验指南：`docs/EXPERIMENT_GUIDE.md`
- 常见陷阱：`docs/PITFALLS.md`

## 模块索引（与导出项）

- `membership`（成员） | `swim`（故障检测）
- `topology`（一致性哈希） | `partitioning`（分片）
- `replication`（法定人数/复制） | `consistency`（一致性级别）
- `consensus`（抽象；可选 `consensus-raft`）
- `transactions`（Saga） | `storage`（状态机/日志/幂等）
- `scheduling`（时钟/定时） | `transport`（RPC 抽象） | `codec`

## 特性开关

- `runtime-tokio`：启用 Tokio 运行时与异步功能。
- `consensus-raft`：启用最小 Raft 实验性实现与结构。

## 学习建议

按 `docs/COURSE_ALIGNMENT.md` 顺序实验：membership→topology→replication→consistency→storage→transactions→consensus（可选）。

## 运行与基准

- 运行测试：`cargo test -p c20_distributed`
- 运行示例：
  - Saga：`cargo run -p c20_distributed --example e2e_saga`
  - 复制：`cargo run -p c20_distributed --example e2e_replication`
  - 服务发现+负载均衡+动态配置：`cargo run -p c20_distributed --example e2e_discovery_lb_config`
  - 最小示例（仅治理热更新）：`cargo run -p c20_distributed --example e2e_governance_min`
  - 最小示例（仅混沌注入）：`cargo run -p c20_distributed --example e2e_chaos_min`
  - 最小示例（仅负载均衡切换）：`cargo run -p c20_distributed --example e2e_load_balancer_min`
- 基准（Criterion）：`cargo bench -p c20_distributed`

可观测性（可选）：启用 feature 并设置日志级别

- `cargo run -p c20_distributed --features observability --example e2e_discovery_lb_config`
- 设置级别：`RUST_LOG=info` 或 `RUST_LOG=debug`

## Quickstart（3分钟上手）

1) 准备 `app.json`（复制下文示例到项目根目录）：
   - 至少包含：`lb.strategy`、`rl.client.*`、`cb.user_service.*`、`chaos.*`、`acl.rules`（可选）

2) 运行最小示例，分别验证三类能力：
   - 治理热更新：`cargo run -p c20_distributed --example e2e_governance_min`
   - 混沌注入：`cargo run -p c20_distributed --example e2e_chaos_min`
   - 负载均衡切换：`cargo run -p c20_distributed --example e2e_load_balancer_min`

3) 联动示例（服务发现+负载均衡+动态配置）：
   - 基础：`cargo run -p c20_distributed --example e2e_discovery_lb_config`
   - 启用日志：`RUST_LOG=info cargo run -p c20_distributed --features observability --example e2e_discovery_lb_config`

4) 修改 `app.json` 并保存，观察控制台输出变化（限流/熔断/混沌/策略切换均可热更新）。

## 参考输出（Expected Logs）

### 联动示例：服务发现 + 负载均衡 + 动态配置

```text
[config] lb.strategy = RoundRobin, instances = 2
tick=0 req=0 -> user-service 127.0.0.1:8080 ok=true
tick=0 req=1 -> user-service 127.0.0.1:8081 ok=true
[override] switch to RoundRobin
[override] switch to ConsistentHash
[override] enable chaos: latency/drop/partition
```

### 仅治理热更新（限流/熔断/ACL）

```text
[governance] rl(cap=50,refill=50), cb(th=5,open_ms=1000)
0: result=false
1: result=true
...
10: rate-limited
...
22: circuit-open
```

### 仅混沌注入

```text
[chaos] latency=10ms jitter=5ms drop_rate=0.1 partition=false
0: ok
1: dropped
2: ok
...
```

### 仅负载均衡策略切换

```text
tick=0 req=0 -> user-service 127.0.0.1:8080
tick=0 req=1 -> user-service 127.0.0.1:8081
[switch] -> WeightedRoundRobin
[switch] -> Random
[switch] -> ConsistentHash
```

## 配置键约定（示例）

以下键通过 `ConfigManager` 可热更新，示例在 `examples/e2e_discovery_lb_config.rs`：

- 负载均衡
  - `lb.strategy`: `weighted_rr` | `round_robin` | `least_conn` | `random` | `weighted_random` | `least_rt` | `geo` | `consistent_hash`

- 混沌注入（示例：开启延迟/丢包/分区）
  - 动态更新建议通过代码调用 `ChaosInjector::update(ChaosConfig { .. })`（示例已演示 tick=3 开启）
  - 若需外部文件承载，可在 `app.json` 写入业务自定键并在订阅回调中映射到 `ChaosConfig`
    - 建议键：
      - `chaos.latency_ms`: 数值（u64）
      - `chaos.jitter_ms`: 数值（u64）
      - `chaos.drop_rate`: 数值（0.0~1.0）
      - `chaos.partition_enabled`: 布尔
      - `chaos.partition_peers`: 数组（服务名列表）

- 安全与治理
  - ACL（示例通过 `AclManager::replace_rules` 设置）
    - 建议键：`acl.rules`（数组）→ 应用层在订阅中解析为 `Vec<AclRule>`
  - 限流（令牌桶）
    - 建议键：
      - `rl.client.capacity`: 数值（u64）
      - `rl.client.refill_per_sec`: 数值（u64）
  - 熔断器
    - 建议键：
      - `cb.user_service.error_threshold`: 数值（u32）
      - `cb.user_service.open_ms`: 数值（u64）

说明：README 中提供的是建议键；示例演示了如何在订阅回调中读取键并更新 `LoadBalancerManager`、`ChaosInjector`、`Governance` 等组件。

### 可直接拷贝的最小 `app.json` 示例

将以下内容保存为项目根或运行目录下的 `app.json`，示例二进制会自动加载：

```text
{
  "lb.strategy": "weighted_rr",
  "rl.client.capacity": 100,
  "rl.client.refill_per_sec": 100,
  "cb.user_service.error_threshold": 5,
  "cb.user_service.open_ms": 1000,
  "chaos.latency_ms": 0,
  "chaos.jitter_ms": 0,
  "chaos.drop_rate": 0.0,
  "chaos.partition_enabled": false,
  "chaos.partition_peers": [],
  "acl.rules": [
    { "principal": "service:client", "resource": "user-service", "action": "read", "allow": true }
  ]
}
```

运行时可动态修改该文件并触发 `ConfigManager` 订阅刷新（示例中 `FileSource` 已加入）。

## 新增：Raft 作用域回调（避免 'static 闭包）

当启用 `consensus-raft` 特性时，可使用作用域回调避免在测试中引入 `Arc<Mutex<...>>`：

```rust
use c20_distributed::consensus_raft::{MinimalRaft, RaftNode, AppendEntriesReq, Term, LogIndex};

let mut raft: MinimalRaft<Vec<u8>> = MinimalRaft::new();
let mut buf: Vec<Vec<u8>> = Vec::new();
{
    let mut apply = |e: &Vec<u8>| buf.push(e.clone());
    let mut scoped = raft.set_apply_scoped(&mut apply);
    let req = AppendEntriesReq { term: Term(1), leader_id: "n1".into(), prev_log_index: LogIndex(0), prev_log_term: Term(0), entries: vec![b"x".to_vec()], leader_commit: LogIndex(1) };
    let _ = scoped.handle_append_entries(req);
}
assert_eq!(buf, vec![b"x".to_vec()]);
```

说明：`set_apply_scoped` 返回守卫对象 `ScopedApply`，在该作用域内对 Raft 的 `handle_append_entries` 调用将使用提供的非 `'static` 回调。

## 新增：Property Tests（随机化性质测试）

本 crate 使用 `proptest` 为 SWIM 与复制模块增加性质测试：

- `tests/prop_swim.rs`：
  - 直连探测与可达性一致（Alive/Suspect）。
  - 版本号优先级合并（高版本覆盖低版本）。
- `tests/prop_replication.rs`：
  - 多数仲裁性质：`required_acks >= floor(N/2)+1` 且 `<= N`。
  - 复制成功当且仅当 `acks >= required_acks`。

运行：`cargo test -p c20_distributed --all-features`

# 时间与时钟（Time & Clocks）

- 物理时钟：NTP、PTP、时钟漂移与误差界
- 逻辑时钟：Lamport Clock、Vector Clock、DSV
- TrueTime 与外部一致性（Spanner）

## 物理时钟与误差

- NTP：网络时间同步，误差通常在毫秒级；对称路径与抖动影响测量。
- PTP：硬件时间戳，数据中心可达亚微秒级；部署成本更高。
- 误差界 ε：系统应保守估计 `now() ∈ [t-ε, t+ε]`，用于租约与安全读的上界分析。

## 逻辑时钟

- Lamport 时钟：提供事件的偏序；`LC(x) < LC(y)` ⇒ x→y，但非充要。
- Vector Clock：提供因果关系的必要充分条件；用于因果一致与冲突检测。
- DSV（Dependency/Version Vector）：跨分片跟踪依赖的紧凑表示。

## TrueTime/外部一致性

- TrueTime 提供 `TT.now()` 返回 `[earliest, latest]` 区间。
- 外部一致性：事务提交在真实时间上保持顺序；`commit wait` 等待到 `TT.after(commit_ts)` 才对外可见。

## 工程要点

- 租约（lease read）：需要安全时钟界和心跳；避免时钟回拨导致安全性破坏。
- 单调时钟：优先使用单调时间源（如 TSC/monotonic），避免墙钟回拨影响。
- 时钟异常：检测跃变，触发降级（禁用基于时间的乐观优化，转为多数派校验）。

## 租约读（Lease Read）安全检查清单

- 时钟安全界：维护 ε 的上界估计；当 ε 超阈值时禁用租约读。
- 心跳预算：领导者心跳周期 T_hb，租约期限 L 满足 L ≥ k·T_hb（k≥3 建议）。
- 单调源：租约判断使用单调时钟；墙钟回拨不得影响判定。
- 续约窗口：在 L−δ 内触发续约；若续约失败则切换到多数派读。
- 故障恢复：恢复后的节点在确认任期/视图之前不得提供租约读。

## 实验与命令

- 回拨模拟：`tests/lease_read.rs`（规划）模拟 `ε` 增大与心跳丢失，验证拒绝路径与降级到多数派读。
- 运行：`cargo test -p c20_distributed --test lease_read -- --nocapture`

## 接口锚点

- `scheduling::LogicalClock`、`TimerService`：提供单调时间与定时。
- `replication`/`consensus`：若实现租约读，需在领导者任期内校验时钟与心跳预算。

## 实验与检查点

- 实验：在 `tests/` 中模拟心跳丢失/时钟回拨，验证租约读的拒绝路径与回退到多数派读。
- 检查点：当 `ε` 超阈值或心跳过期时，读路径必须执行屏障或降级策略。

## 进一步阅读

- Wiki：`NTP`, `PTP`, `Lamport timestamps`, `Vector clock`, `Google TrueTime`
- 课程：MIT 6.824（Time, Clocks）、UWash CSE452（Time & Sync）
- 论文：Spanner、Time, Clocks, and the Ordering of Events in a Distributed System（Lamport）

## 练习与思考

1. 实现 `TrueTime` 风格的时间区间接口（earliest/latest），在租约读中引入 `commit wait`，验证外部一致性。
2. 构建时钟异常检测器：模拟墙钟回拨/漂移增大，验证租约读自动降级到多数派读。
3. 设计跨分片的因果跟踪（DSV/向量时钟），实现最小的因果一致读路径。
4. 对比单调时钟与墙钟在调度/租约/一致性判定中的差异与风险。

## 快速导航

- 分布式系统总纲：`../README.md`
- 调度与定时：`../scheduling/README.md`
- 一致性模型：`../consistency/README.md`
- 共识机制：`../consensus/README.md`

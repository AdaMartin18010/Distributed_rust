# 调度与流控（Scheduling & Flow Control）

- 背压、限流、优先级、公平性、队列与隔离
- 延迟预算与截止时间传递（deadline propagation）

## 目录

- [调度与流控（Scheduling \& Flow Control）](#调度与流控scheduling--flow-control)
  - [目录](#目录)
  - [背压与限流](#背压与限流)
    - [令牌桶 vs 漏桶（对比）](#令牌桶-vs-漏桶对比)
    - [令牌桶最小示例](#令牌桶最小示例)
  - [调度与隔离](#调度与隔离)
    - [优先级调度示意](#优先级调度示意)
  - [截止时间传递](#截止时间传递)
    - [在 RPC 层的传播细则](#在-rpc-层的传播细则)
    - [参数耦合指南（与共识心跳/选举）](#参数耦合指南与共识心跳选举)
  - [进一步阅读](#进一步阅读)
  - [调度（Scheduling）](#调度scheduling)
  - [逻辑时钟（Logical Clock）](#逻辑时钟logical-clock)
  - [定时器与心跳（TimerService \& Heartbeat）](#定时器与心跳timerservice--heartbeat)
  - [重试与退避（Retry \& Backoff）](#重试与退避retry--backoff)
    - [抖动策略示例（Rust）](#抖动策略示例rust)
  - [截止时间（Deadline）与取消（Cancellation）](#截止时间deadline与取消cancellation)
  - [与其它模块的关系](#与其它模块的关系)
  - [实践建议](#实践建议)
  - [练习与思考](#练习与思考)
  - [快速导航](#快速导航)

## 背压与限流

- 令牌桶/漏桶：控制突发与平均速率；支持优先级队列化。
- 拒绝策略：超载时早拒绝，保护尾延迟；结合重试策略避免放大。
- 客户端退避：指数退避+抖动；读取服务端信号（429/`Overloaded`）。

### 令牌桶 vs 漏桶（对比）

- 令牌桶：允许突发至 `capacity`，长期速率受 `refill_rate` 限制，适合需要吸收瞬时抖动的在线服务。
- 漏桶：输出速率严格、平滑；突发被排队或丢弃，适合需要稳定带宽的后台任务/批处理。
- 实践：前端采用令牌桶吸收抖动，后台/出站链路对齐漏桶，二者可级联。

### 令牌桶最小示例

```rust
pub struct TokenBucket { capacity: u64, tokens: u64, refill_per_ms: f64, last: std::time::Instant }
impl TokenBucket {
  pub fn allow(&mut self, now: std::time::Instant) -> bool {
    let dt = now.duration_since(self.last).as_millis() as f64;
    self.tokens = (self.tokens as f64 + dt * self.refill_per_ms).min(self.capacity as f64) as u64;
    self.last = now;
    if self.tokens > 0 { self.tokens -= 1; true } else { false }
  }
}
```

## 调度与隔离

- 多级队列：不同租户/优先级分池；保障关键流量。
- 资源配额与自适应调参：基于实时指标（QPS/延迟/丢弃率）调整权重与并发度。

### 优先级调度示意

- 多队列：`high/normal/low`，按配额轮询；当高优先级积压时抢占低优先级。
- 动态权重：依据 `p95_latency` 超阈值提升高优先级权重，恢复后逐步回落（hysteresis）。

## 截止时间传递

- 请求携带总预算（如 200ms）；每次 RPC 更新剩余预算，避免无界排队与重试风暴。
- 与 `transport` 的 `RetryPolicy` 协同，按剩余预算计算下次超时与退避。

### 在 RPC 层的传播细则

- Header/Context：在入口计算 `deadline_at` 并放入上下文；跨服务调用时透传并在每跳更新剩余时间。
- 超时预算分配：`rpc_timeout = min(deadline_left * α, upper_bound)`，α 建议 0.5~0.8；重试累计不得超过预算。
- 失败分类：`Timeout/Overloaded/Unavailable` 才可重试；`Invalid`/`PermissionDenied` 等立即失败，不消耗预算。

### 参数耦合指南（与共识心跳/选举）

- 心跳周期 Thb 与选举超时 E：建议 E ∈ [2.5Thb, 5Thb]，避免同时竞选。
- Deadline 预算 B 与重试次数 R：保证 Σ(超时+退避) ≤ B；若 E 接近 B，读写应降级以避免误判失败。

## 进一步阅读

- Wiki：`Rate limiting`, `Backpressure (computing)`
- 课程：MIT 6.824（Scheduling & Tail at Scale 衍生讨论）
- 论文：The Tail at Scale、pFabric/pHost、PIAS（面向流的优先级调度）

## 调度（Scheduling）

- 逻辑时钟（Lamport）、心跳/选举超时、任务重试与退避、截止时间（Deadline）
- 接口：`LogicalClock`, `TimerService`

## 逻辑时钟（Logical Clock）

- Lamport Clock：以单调递增计数为事件全序提供可比性。适用于互斥、领导者“更新更晚获胜”等策略。
- Vector Clock（扩展思路）：可表达并发关系，但成本更高；在本仓库中默认不实现，必要时可在应用层封装。
- 与接口的映射：
  - `LogicalClock::now()` 返回逻辑时间戳（单调递增）。
  - `LogicalClock::tick()` 在本地事件/发送前递增；`merge(peer_ts)` 在接收时合并为 `max(local, peer)+1`。

最小示例：

```rust
use distributed::scheduling::LogicalClock;

fn handle_send<C: LogicalClock>(clock: &mut C) {
    let ts = clock.tick();
    tracing::debug!(?ts, "send");
}

fn handle_recv<C: LogicalClock>(clock: &mut C, peer_ts: u64) {
    let ts = clock.merge(peer_ts);
    tracing::debug!(?ts, "recv");
}
```

## 定时器与心跳（TimerService & Heartbeat）

- 接口：`TimerService` 提供 `after_ms(d, f)` 与周期性调度 `every_ms(d, f)`（若实现提供）。
- 实现建议：
  - `TokioTimer`（需启用 feature `runtime-tokio`）：基于 Tokio 的 `sleep`/`interval`。
  - 纯同步 `WheelTimer`（可选扩展）：用于无运行时环境的测试。
- 心跳（领导者→跟随者）：周期 T 心跳；跟随者在超时 E 未收到心跳时触发选举。典型取值关系：E ∈ [2.5T, 5T] 以降低同时竞选概率。

示例：

```rust
use distributed::scheduling::TimerService;

fn start_heartbeat<T: TimerService>(timer: &T, period_ms: u64, mut send_beat: impl FnMut() + Send + 'static) {
    timer.every_ms(period_ms, move || {
        send_beat();
    });
}
```

## 重试与退避（Retry & Backoff）

- 何时重试：超时/可重试错误（如 `Unavailable`、`Timeout`）。
- 幂等性：必须结合上层幂等键（参见 `transport` 与 `transactions`/`replication`）。
- 退避策略：
  - 固定退避：间隔常数；适合低冲突。
  - 指数退避：间隔倍增；需加入抖动（Jitter）避免同步风暴。
  - 线性+抖动：低复杂度且较稳健。
- 与代码：`tests/retry.rs`, `tests/retry_backoff.rs` 展示了典型策略与边界。

### 抖动策略示例（Rust）

```rust
fn equal_jitter(base: u64, attempt: u32, max: u64) -> u64 {
    use rand::{thread_rng, Rng};
    let exp = base.saturating_mul(1u64.saturating_shl(attempt.min(31)));
    let cap = exp.min(max);
    let half = cap / 2;
    let jitter: u64 = thread_rng().gen_range(0..=half);
    half + jitter
}
```

参考实现（伪代码接口）：

```rust
use distributed::transport::{RetryPolicy};

let policy = RetryPolicy {
    max_retries: 5,
    retry_on_empty: true,
    backoff_base_ms: Some(10), // 指数/线性由实现决定，建议带抖动
};
```

## 截止时间（Deadline）与取消（Cancellation）

- Deadline：在发起请求时计算 `start + budget`，中途所有重试共享预算，避免“无界重试”。
- 取消：外部可传递 `CancellationToken`（或 `Context`）以便在拓扑变化/上游关闭时尽快退出。

## 与其它模块的关系

- 与 `consensus`：领导者心跳与选举超时依赖定时器与随机化。
- 与 `transport`：重试需结合超时、幂等键与退避；调用方应传入截止时间。
- 与 `replication`：读写 `Quorum` 时，超时策略与剩余预算直接影响一致性/可用性权衡。

## 实践建议

- 默认开启抖动（Full Jitter 或 Equal Jitter）。
- 选举超时取随机区间 E ∈ [a, b]，且 b 至少为心跳周期的 3 倍。
- 重试次数与预算统一管理；禁止在多层重复设置无界重试。

## 练习与思考

1. 实现基于截止时间传递的重试器：同一请求的所有重试共享预算，比较不同退避策略的P95/P99影响。
2. 设计多级队列与租户隔离：在过载下保障高优先级流量，测量丢弃率与尾延迟变化。
3. 构建逻辑时钟与定时器接口的参考实现（Tokio/同步），完成领导者心跳与选举时间线仿真。
4. 联动 `transport` 的失败信号与客户端退避，验证避免级联过载的效果。

## 快速导航

- 分布式系统总纲：`../README.md`
- 传输与重试：`../transport/README.md`
- 共识机制：`../consensus/README.md`
- 时间与时钟：`../time/README.md`

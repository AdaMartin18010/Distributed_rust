# 传输（Transport）

- RPC 抽象、流控与背压、超时与重试、幂等性
- 接口：`RpcClient`, `RpcServer`

## 重试策略

```rust
use c20_distributed::transport::{InMemoryRpcServer, InMemoryRpcClient, RetryClient, RetryPolicy};

let mut srv = InMemoryRpcServer::new();
srv.register("echo", Box::new(|b| b.to_vec()));
let cli = InMemoryRpcClient::new(srv);
let retry = RetryClient { inner: cli, policy: RetryPolicy { max_retries: 3, retry_on_empty: true, backoff_base_ms: Some(10) } };
let _ = retry.call("echo", b"hi");
```

## 超时与截止时间（Deadline）

- 为每次请求设置总预算（如 200ms），重试共享同一截止时间，避免“无界重试”。
- 与 `scheduling` 配合：根据剩余预算动态计算下一次超时与退避间隔。

## 幂等键与去重缓存

- 客户端携带 `idempotency_key`；服务端维护去重缓存（LRU/时间窗）。
- 返回上次成功的结果或拒绝重复副作用，确保在重试/乱序场景下安全。

## 背压（Backpressure）

- 通过限流器/令牌桶防止过载；在客户端侧可根据错误信号进行指数退避。

## 接口清单（API 概览）

- RpcClient：`call(op: &str, payload: &[u8]) -> Result<Vec<u8>, DistributedError>`
- RpcServer：`register(op: &str, handler: Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>)`
- RetryPolicy：`{ max_retries, retry_on_empty, backoff_base_ms }`
- RetryClient：装饰器，复用同一截止时间；支持抖动退避（建议）。

## 失败注入与可观测性

- 失败类型：超时、连接拒绝、半开（首包丢失）、服务端 5xx。
- 注入建议：在 `InMemoryRpcServer` handler 中按概率返回错误或延迟。
- 指标：`
  - rpc.client.retry.count / rpc.client.retry.giveup
  - rpc.latency.ms{op} P50/P95/P99
  - rpc.server.dedup.hit/miss（幂等去重命中率）
`

## 测试与示例命令

- 运行最小重试用例：`cargo test -p c20_distributed --test retry_basic -- --nocapture`
- 运行管线/背压用例（若存在）：`cargo test -p c20_distributed --test pipeline`
- Bench（含退避对尾延迟影响）：`cargo bench -p c20_distributed`

## 进一步阅读

- Wiki：`Remote procedure call`, `Exponential backoff`, `Backpressure`, `Idempotence`
- 课程：MIT 6.824（RPC, Failures and Timeouts）、UWash CSE452（Communication）
- 实践：gRPC Retry/hedging 指南、Envoy 连接池与熔断、Finagle/Hystrix 模式

## 相关实验/测试与代码锚点

- 测试：`tests/retry_*.rs`, `tests/pipeline.rs`（若存在/规划）。
- 基准：与重试/退避相关可并入 `cargo bench -p c20_distributed` 的延迟分布对比。
- 代码锚点：`transport::{InMemoryRpcServer, InMemoryRpcClient, RetryClient, RetryPolicy}`；与 `scheduling::{TimerService}` 协作实现超时与退避。

## 练习与思考

1. 实现统一的 `IdempotencyKey` 协议：客户端生成、服务端去重缓存（LRU + TTL），在乱序/重试下验证幂等性。
2. 设计“共享截止时间”的重试器：同一请求的所有重试共享一次截止时间预算，并支持抖动退避；测量 P50/P95/P99 延迟。
3. 编写失败注入基准：按比例注入超时/半开/5xx，比较重试策略（固定/指数/带抖动）对尾延迟与成功率的影响。

## 快速导航

- 分布式系统总纲：`../README.md`
- 一致性与事务：`../transactions/README.md`（若存在）
- 观测与SLO：`../observability/README.md`（若存在）

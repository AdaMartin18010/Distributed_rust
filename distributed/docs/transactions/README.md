# 事务（Transactions）

- SAGA/TCC/2PC/幂等与隔离级别

## SAGA 最小示例

```rust
use c20_distributed::transactions::{Saga, SagaStep};

struct Step;
impl SagaStep for Step {
  fn execute(&mut self) -> Result<(), c20_distributed::DistributedError> { Ok(()) }
  fn compensate(&mut self) -> Result<(), c20_distributed::DistributedError> { Ok(()) }
}

let saga = Saga::new().then(Box::new(Step));
let _ = saga.run();
```

## SAGA 状态机与补偿

- 状态：Pending → Executing(i) → Completed | Compensating(j) → Compensated | Failed
- 每步提供 `execute/compensate`，要求补偿操作可重入且幂等。
- 失败策略：遇到不可恢复错误时自后向前补偿已完成步骤。

## TCC（Try-Confirm-Cancel）

- Try：预留资源；Confirm：提交最终变更；Cancel：释放资源。
- 与 SAGA 的差异：TCC 对资源做“先锁定后提交”，对一致性更强，但需要服务支持两阶段接口。

## 2PC 与协调者

- 协议：Prepare/Commit 两阶段；协调者负责收集参与者的投票并广播提交/中止。
- 局限：协调者单点、阻塞问题（需三阶段或基于共识的协调补救）。
- 工程：将协调者运行在具共识的日志上（如 Raft）以避免脑裂/遗失决定。

## 隔离级别矩阵（示意）

- 级别：Read Uncommitted | Read Committed | Repeatable Read | Serializable
- 现象：脏读、不可重复读、幻读、写偏差
- 建议：
  - Saga 通常目标为业务不变式而非强隔离；通过幂等/校验与读屏障降低异常现象影响。
  - 若需要强隔离，考虑单分区内强一致提交，跨分区用补偿或全序日志（如 Calvin）。

| 级别 | 脏读 | 不可重复读 | 幻读 | 写偏差 |
| ---- | ---- | ---------- | ---- | ------ |
| RU   | 可能 | 可能       | 可能 | 可能   |
| RC   | 否   | 可能       | 可能 | 可能   |
| RR   | 否   | 否         | 可能 | 可能   |
| SER  | 否   | 否         | 否   | 否     |

## 幂等与重复消息

- 每个事务/步骤带 `idempotency_key`；服务端记录并去重。
- 与 `transport::RetryPolicy` 协作，确保在重试/超时场景下不会产生重复副作用。

## 隔离级别与分布式事务

- 隔离级别：读未提交、读已提交、可重复读、可串行化。
- 可串行化实现：两阶段锁（2PL）或乐观并发控制（OCC）+ 验证；分布式需时间戳或全序日志。
- 外部一致性：依赖 `time/TrueTime` 并在提交时 `commit wait`（Spanner）。

## 进一步阅读

- Wiki：`Saga pattern`, `Two-phase commit`, `TCC`, `Isolation (database systems)`
- 课程：MIT 6.824（Transactions）、CMU 15-445（Concurrency/Recovery）
- 论文/实践：Spanner、Calvin（Deterministic DB）、Sagas（原论文）

## 与复制/一致性的交互

- 在 `replication` 采用多数派写入以确保确认后的步骤不会丢失。
- 在 `consistency` 选择合适的 R/W 配置以平衡延迟与一致性需求。

## 实验与测试入口

- 运行：`cargo test -p c20_distributed --test saga`；示例：`cargo run -p c20_distributed --example e2e_saga`
- 观察：失败步触发逆序补偿与重复补偿的幂等性。

## 示例扩展（两步业务伪代码）

```rust
use c20_distributed::transactions::{Saga, SagaStep};

struct ReserveInventory; // Try/Confirm/Cancel 可映射为 execute/compensate
impl SagaStep for ReserveInventory {
  fn execute(&mut self) -> Result<(), c20_distributed::DistributedError> { Ok(()) }
  fn compensate(&mut self) -> Result<(), c20_distributed::DistributedError> { Ok(()) }
}

struct ChargePayment;
impl SagaStep for ChargePayment {
  fn execute(&mut self) -> Result<(), c20_distributed::DistributedError> { Err("payment failed".into()) }
  fn compensate(&mut self) -> Result<(), c20_distributed::DistributedError> { Ok(()) }
}

let saga = Saga::new().then(Box::new(ReserveInventory)).then(Box::new(ChargePayment));
let _ = saga.run(); // 触发逆序补偿
```

## 代码导航与检查清单

- `transactions::{Saga, SagaStep}` 是否对补偿重入友好（可多次调用无副作用）。
- 与 `storage::IdempotencyStore` 的去重是否在 execute 与 compensate 均生效。
- 与 `replication` 的写入路径：提交前读路径是否有屏障/会话保证，避免读到中间态。

## 代码锚点

- `transactions::{Saga, SagaStep}`，`storage::IdempotencyStore`；与 `replication` 结合确保持久与可见性。

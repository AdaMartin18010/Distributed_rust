# 持续推进路线图（c20_distributed）

目标：按照“课程对标→概念落地→可验证实现→实验复现”的路径，逐步完善，并为每个里程碑提供可操作的交付与验收标准。

---

## 里程碑 M1：基础完备（当前）

- 目标：建立概念→代码→实验的最小闭环。
- 文档交付：课程对标、Wiki 映射、概念模型、关键论证、README 导航（已完成）。
- 代码现状：`replication` 多数派、`transactions::Saga`、`topology` 一致性哈希、`storage` 幂等接口（已存在）。
- 需要补齐的交付：
  - 单元/属性测试样例：`tests/replication_local.rs`、`tests/hashring_properties.rs`、`tests/saga.rs` 的最小断言覆盖。
  - 最小端到端 Demo：`examples/e2e_saga.rs` 串起路由→复制→幂等→补偿。
- 验收标准：
  - `cargo test -p c20_distributed` 通过且包含上述测试文件。
  - `cargo run -p c20_distributed --example e2e_saga` 正常结束，输出含补偿路径观测。
- 关联实验/测试：详见《实验指南》1、2、3 小节。
- 风险与缓解：
  - 风险：测试对时间参数敏感 → 缓解：使用逻辑时钟/可配置超时模拟。

### 文件与路径核对清单（M1）

- 测试：
  - `crates/c20_distributed/tests/replication_local.rs`
  - `crates/c20_distributed/tests/hashring_properties.rs`
  - `crates/c20_distributed/tests/saga.rs`
- 示例：
  - `crates/c20_distributed/examples/e2e_saga.rs`
- 文档：
  - `crates/c20_distributed/docs/EXPERIMENT_GUIDE.md`
  - `crates/c20_distributed/docs/WIKI_MAPPING.md`
  - `crates/c20_distributed/docs/CONCEPT_MODEL.md`
  - `crates/c20_distributed/docs/COURSE_ALIGNMENT.md`

## 里程碑 M2：一致性与复制增强

- 目标：提供 Strong/Quorum/Eventual 可观察差异与可插拔仲裁策略。
- 交付物：
  - 在 `replication` 增加读/写分离策略与 `required_acks` 表驱动实现。
  - 提供 Jepsen 风格本地历史生成器与基本检查器（线化/会话保证子集）。
- 验收标准：
  - `tests/consistency_matrix.rs` 展示不同 Level 下的读写可见性差异并通过。
  - `cargo bench -p c20_distributed` 输出 `ack_quorum_table`，包含不同 N、Level 的统计。
- 关联实验/测试：实验 1、5。
- 风险与缓解：
  - 风险：线化检查复杂度高 → 缓解：先做小规模 bounded 检查与会话保证子集。

## 里程碑 M3：SWIM 参数化与可视化

- 目标：让成员探测行为可配置且可观测。
- 交付物：
  - `swim` 支持探测周期、fanout、间接探测开关；事件轨迹导出（CSV/JSON）。
  - 提供可视化脚本（Python/gnuplot）生成收敛曲线与误报率图。
- 验收标准：
  - `cargo test -p c20_distributed --test swim_pingreq -- --nocapture` 打印包含间接成功样例。
  - 可视化脚本对样本数据生成图像文件（存入 `target/tmp/`）。
- 关联实验/测试：实验 4。
- 风险与缓解：
  - 风险：日志量过大 → 缓解：采样与压缩输出；按轮次分段持久化。

## 里程碑 M4：最小 Raft 实验路径

- 目标：启用最小 Raft 协议流转，验证日志复制与领导者切换。
- 交付物：
  - `consensus-raft` 特性：`AppendEntries`/`RequestVote` 状态与消息处理骨架。
  - 持久化接口与快照 stub（草案）；与 `storage` 对接基本 Apply。
- 验收标准：
  - `tests/raft_minimal.rs`：领导者选举、前缀匹配、提交索引单调性断言通过。
  - `cargo bench -p c20_distributed` 输出 Raft 心跳与复制延迟的粗略统计（非稳定）。
- 关联实验/测试：新增“Raft 最小路径”实验小节。
- 风险与缓解：
  - 风险：状态爆炸与时序竞态 → 缓解：单线程模拟 + 驱动步进器，先验收安全性再优化活性。

## 里程碑 M5：端到端事务样例

- 目标：用 Saga+幂等存储实现跨分片两步业务并具备补偿能力。
- 交付物：
  - `examples/e2e_payment_inventory.rs`（或整合 `e2e_saga`）：下单→锁库存→支付，任一步失败触发逆序补偿。
  - `transactions` 提供幂等键建议与补偿重试策略接口。
- 验收标准：
  - `cargo run -p c20_distributed --example e2e_saga` 输出包含补偿重试日志且最终不变式保持。
  - `tests/saga_compensation.rs` 覆盖重复补偿与跨分区写入路径。

---

## 快速验收脚本建议（本地）

```powershell
# 1) 单元与属性测试
cargo test -p c20_distributed --all-features -- --nocapture

# 2) 示例
cargo run -p c20_distributed --example e2e_saga

# 3) 基准（可选）
cargo bench -p c20_distributed
```

- 关联实验/测试：实验 2。
- 风险与缓解：
  - 风险：外部副作用不可回滚 → 缓解：通过外部适配层引入“预留与超时释放”策略。

---

## 工程化与质量

- 目标：可重复、可度量、可观测。
- 交付物：
  - 属性测试与基准：`benches/` 增加再均衡与 ACK 延迟分布基准。
  - 文档：《实验指南》与《常见陷阱》保持与实现同步更新。
- 验收标准：
  - `cargo bench -p c20_distributed` 可在本地生成摘要；关键图表说明写入 `docs/experiments/`。

## 时间建议

- 建议：M1→M2 1-2 周，M3 1 周，M4 2-3 周，M5 1-2 周（部分可并行）。

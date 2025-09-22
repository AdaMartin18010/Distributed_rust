# 实验指南（c20_distributed）

1) 复制与一致性

   - 运行：`cargo test -p c20_distributed --test replication_local`
   - 前置：确保启用默认特性；无网络依赖。
   - 步骤：观察不同 `ConsistencyLevel` 的 `required_acks` 与写入成功/失败条件；注入少量失败（若测试覆盖）。
   - 期望输出：在 Quorum/Strong 下，ACK 达阈值前不应标记提交；Eventual 可能提前返回且读到旧值。
   - 失败排查：检查 `required_acks(N, level)` 表是否正确；超时是否导致假失败；随机种子固定重试。

2) Saga 补偿

   - 运行：`cargo test -p c20_distributed --test saga` 或示例 `cargo run -p c20_distributed --example e2e_saga`
   - 前置：`storage::IdempotencyStore` 可用；示例会模拟一步失败。
   - 步骤：触发第二步失败，验证补偿逆序执行；重复触发补偿以验证幂等。
   - 期望输出：最终不变式保持（例如余额/库存守恒）；重复补偿无额外副作用。
   - 失败排查：幂等键选择是否稳定；补偿是否可重复；外部副作用需以适配层虚拟化。

3) 一致性哈希与再均衡

   - 运行：`cargo test -p c20_distributed --test hashring_properties`
   - 前置：配置虚拟节点数（如 16/32）。
   - 步骤：扩容 1 个节点，采样大量键并统计迁移比例。
   - 期望输出：迁移比例接近 1/N（含小样本波动）；倾斜度随虚拟节点数增大而降低。
   - 失败排查：哈希函数是否均匀；虚拟节点数过小；采样量不足（增大样本）。

4) SWIM 探测

   - 运行：`cargo test -p c20_distributed --test swim_pingreq -- --nocapture`
   - 前置：启用间接探测；设置合适的超时与 fanout。
   - 步骤：对一个节点故意丢弃直接探测，观察 `ping-req` 间接成功事件。
   - 期望输出：间接探测成功率较直接高；Suspect→Confirm Dead 的时间分布可观测。
   - 失败排查：超时设置过小/过大；日志级别不足导致看不到事件。

5) 基准（Criterion）

   - 运行：`cargo bench -p c20_distributed`
   - 前置：安装 Criterion；禁用不必要的日志以减少干扰。
   - 步骤：运行 `ack_quorum_table` 与哈希环再均衡基准。
   - 期望输出：输出摘要包含 P50/P95/P99；不同 N 与 Level 的对比明显。
   - 失败排查：CPU 省电/频率波动影响稳定性；建议固定性能模式并多次取中位。

## 联动与命令

- 文档：`WIKI_MAPPING.md`、`CONCEPT_MODEL.md`、`COURSE_ALIGNMENT.md`、`ROADMAP.md`
- 一键执行（建议本地 PowerShell）：

```powershell
cargo test -p c20_distributed --all-features -- --nocapture
cargo run -p c20_distributed --example e2e_saga
cargo bench -p c20_distributed
```

---

## 环境与可复现性建议

- 固定随机种子：

```powershell
$env:RUST_TEST_THREADS=1; $env:RUST_LOG="info,c20_distributed=debug"; $env:RUST_BACKTRACE=1
# 对使用 rand 的测试，可通过环境变量或测试参数注入 SEED
```

- 指标与日志：
  - 输出心跳/选举/ACK 统计，便于定位 `required_acks` 与超时导致的抖动
  - 建议在基准中降低日志级别以减少干扰

- CPU/电源策略：
  - Windows：设置“高性能/最佳性能”电源计划，避免频率波动
  - Linux：`cpupower frequency-set -g performance`

## 结果度量建议

- 复制与一致性：统计提交延迟分布（P50/P95/P99）、ACK 次数、失败重试率
- Raft 选举：收敛时间分布、双领导者出现率（应为 0 在稳定期）
- 哈希环再均衡：键迁移比例与倾斜系数随虚拟节点数变化曲线

## 常见故障与排错速查

- 测试偶现失败：增大超时或固定种子，确认是否数据竞争/不正确的 await
- 基准波动大：关闭后台任务/杀毒/索引，固定 CPU 频率，增大采样次数
- Saga 漏补偿：检查幂等键是否跨重试稳定；对外部副作用加隔离层模拟

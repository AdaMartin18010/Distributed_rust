# 分布式测试与混沌工程（Testing & Chaos Engineering）

- 单元/属性测试、仿真、故障注入、Jepsen
- 可重复实验与指标验证

## 分层测试

- 单元测试：接口与不变式（`storage`/`transport`/`consensus`）。
- 属性测试：模型检查/无穷测试（proptest/quickcheck）。
- 仿真：离线模拟网络（丢包/乱序/延迟/重复）与节点故障。

## Jepsen 与线性化检查

- 模型：为系统定义操作历史与预期一致性（如线性一致）。
- 检查：以 Knossos/Porcupine 等工具验证历史是否可线性化。
- 工作流：构造器（产生并发操作）→ 故障注入 → 收集历史 → 检查。

## 混沌工程

- 目标：在生产相近环境中验证系统在异常下的稳定性与恢复速度。
- 方法：受控地注入失败（kill -9、网络分区、磁盘满、时钟跃迁）。
- 指标：可用性、SLO 违约率、恢复时间、尾延迟（P99/P999）。

## 工程建议

- 可重复性：随机种子固定、场景脚本化、指标统一采集（tracing/metrics）。
- 安全网：灰度/金丝雀发布、自动回滚、读写保护闸。

## 实验脚手架与历史检查

- 建议在 `tests/` 中提供历史记录器（append-only），格式 JSON/CSV；引入最小线化检查或会话保证检查。
- 通过 feature `testing-sim` 启用网络仿真器：控制延迟/丢包/乱序/重复。
- 将检查结果与指标一起输出，便于回归与可视化。

## 注意事项（混沌）

- 将“故障注入”与“限流/熔断”联动，避免级联故障。
- 在多租户环境中隔离混沌作用域；使用专用命名空间与资源配额。

## 进一步阅读

- Wiki：`Chaos engineering`, `Jepsen`、`Linearizability`
- 课程：MIT 6.824（Labs, Testing）、UWash CSE452（Testing distributed systems）
- 论文/实践：Jepsen 报告合集、Netflix Chaos Monkey、Porcupine（线性化检测）

## 练习与思考

1. 搭建最小Jepsen风格历史检查：生成并发操作、注入分区与时钟抖动、导出历史并用Porcupine验证线性化。
2. 设计可重复仿真框架：固定随机种子、脚本化故障场景、统一指标采集用于回归对比。
3. 构建混沌实验编排器：与限流/熔断联动，验证避免级联故障的策略有效性。
4. 将一致性检查与指标/trace打通，产出带证据的验证报告（历史+追踪+指标）。

## 快速导航

- 分布式系统总纲：`../README.md`
- 一致性模型：`../consistency/README.md`
- 故障处理：`../failure/README.md`
- 可观测性：`../observability/README.md`

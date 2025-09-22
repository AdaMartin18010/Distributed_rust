# 可观测性（Observability）

- Tracing、Metrics、Logging 与 SLO/SLA
- 端到端追踪、指标分层与告警

## Tracing

- 分布式追踪：为请求分配 `trace_id`/`span_id`，传播上下文；采样率自适应控制。
- 指标联动：在关键 span 上打点（直方图/计数器），与延迟预算/截止时间协同。

## Metrics

- RED/USE 模型：请求率（Rate）、错误（Errors）、时延（Duration）；利用资源（Utilization）、饱和度（Saturation）、错误。
- 指标维度：服务、区域、租户、优先级；避免高基数爆炸。

## Logging

- 结构化日志：字段化（json/otlp）；语义级别（debug/info/warn/error）。
- 采样与脱敏：高并发下抽样；敏感信息脱敏与访问控制。

## SLO 与告警

- SLO：如可用性 99.9%、P99 延迟；以错误预算驱动发布与降级策略。
- 告警：基于速率与窗口的复合规则；避免告警风暴与雪崩。

## 练习与思考

1. 设计RED/USE指标体系与分层告警：在突发流量与局部故障下控制告警风暴。
2. 打通Tracing与Metrics：在关键span上埋点直方图，输出端到端延迟SLO合规报告。
3. 实现采样与脱敏策略：在高基数场景下维持可观测性成本与隐私合规。
4. 构建性能回归看板：对比不同版本下的P95/P99、错误预算消耗与恢复时间。

## 快速导航

- 分布式系统总纲：`../README.md`
- 测试与混沌：`../testing/README.md`
- 调度与截止：`../scheduling/README.md`
- 传输与重试：`../transport/README.md`

## 进一步阅读

- Wiki：`Distributed tracing`, `Service-level objective`, `OpenTelemetry`
- 实践：OpenTelemetry/OTLP、Prometheus+Alertmanager、Grafana Tempo/Loki

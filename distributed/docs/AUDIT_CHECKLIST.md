# 文档审计清单（c20_distributed/docs）

用途：列出每个专题文档的关键缺口，提供可勾选的补全项，确保“概念-接口-示例-测试-排错”五要素齐备。

标注约定：

- [ ] 待办  [~] 进行中  [x] 已完成

## 总览

- [x] consensus/README：API 映射、运行/测试、FAQ、read-index 指南
- [x] consistency/README：R/W 法定人数矩阵、读写示例、FAQ
- [x] EXPERIMENT_GUIDE：环境/随机种子、指标、复现实验
- [x] COURSE_ALIGNMENT：课程-模块-代码对照表

## replication/README

- [ ] 给出 `required_acks(N, Level)` 明确表格（Strong/Quorum/Eventual）
- [ ] 写入路径伪代码与失败重试语义（去重幂等建议）
- [ ] 提供“提交推进与读屏障”的时序图
- [ ] 常见问题：ACK 抖动、局部可用导致写放大、尾延迟治理

## failure/README

- [ ] 故障模型分类：Fail-Stop / Omission / Timing / Byzantine（非目标，注记）
- [ ] 分区与两地三中心示意，合并后冲突解决策略导航
- [ ] Chaos 场景注入清单与观测指标（错误率/收敛时间）

## transport/README

- [ ] 重试/退避（指数退避 + 抖动）示例代码
- [ ] 幂等键传递与去重缓存策略
- [ ] 背压与超时预算传播（deadline/timeout）

## scheduling/README

- [ ] 令牌桶/漏桶/优先级调度对比与示例
- [ ] Deadline/优先级在 RPC 层的传播与策略
- [ ] 与共识心跳/选举超时的参数耦合指南

## storage/README

- [ ] 状态机确定性约束与快照截断安全前置条件
- [ ] 快照恢复流程与崩溃一致性 checklist
- [ ] 幂等存储接口的最佳实践与键设计

## topology/README

- [ ] 一致性哈希虚拟节点对倾斜度影响的实验图示
- [ ] 扩缩容再均衡迁移比例推导与实测对照
- [ ] 热点治理策略（粘性路由/副本热点扩散）

## transactions/README（Saga）

- [ ] 编排图与补偿逆序执行的代码骨架
- [ ] 幂等补偿键设计与跨重试稳定性
- [ ] 失败矩阵（重试/跳过/人工介入）与可观测性字段

## testing/README

- [ ] 线性化检查（小历史）示例与工具链接
- [ ] 故障注入策略与随机种子管理
- [ ] Jepsen 思路下的最小实验模板

## time/README

- [ ] 物理时钟偏差与租约读安全界说明
- [ ] 逻辑时钟（Lamport/Vector）与因果传播样例

## observability/README

- [ ] 指标分层：RPC/共识/复制/存储/补偿
- [ ] 关键 SLI/SLO 样例（P95 提交延迟、误报率、恢复时间）
- [ ] 追踪示例与典型问题火焰图导航

## membership/swim/README

- [ ] SWIM 参数指南（fanout/超时/可达性）与 Lifeguard 参考
- [ ] 误报率-超时曲线与收敛时间测量方法

## docs/README（总纲）

- [ ] 在“快速开始”中补充一键命令合集与环境建议链接
- [ ] 在“常见陷阱”中增加针对 R/W 配置与租约读的提醒

---

执行建议：优先补全 replication、transport、storage 三篇（直接影响 e2e 样例与测试），随后完善 failure、testing、observability 以形成“验证-观测-回归”的闭环。

# 分布式系统完整文档索引

> 基于 Rust 1.90 的分布式系统开发库 - 完整理论体系与实践指南

## 📚 文档体系结构

### 🎯 1. 基础理论篇

#### 1.1 核心概念体系

- [1.1.1 概念-关系-属性模型](./CONCEPT_MODEL.md) - 分布式系统核心概念和理论模型
- [1.1.2 形式化论证](./FORMAL_ARGUMENTS.md) - 数学证明和形式化验证
- [1.1.3 课程对标](./COURSE_ALIGNMENT.md) - 与主流课程的知识体系对齐

#### 1.2 一致性理论

- [1.2.1 一致性模型概述](./consistency/README.md) - 线性、顺序、因果、最终一致性
- [1.2.2 CAP/PACELC 定理](./consistency/cap_pacelc.md) - 一致性、可用性、分区容错权衡
- [1.2.3 向量时钟理论](./consistency/vector_clocks.md) - 因果依赖跟踪和实现
- [1.2.4 会话一致性](./consistency/session_consistency.md) - 会话保证和实现机制
- [1.2.5 单调一致性](./consistency/monotonic_consistency.md) - 单调读写的实现

#### 1.3 共识理论

- [1.3.1 共识算法概述](./consensus/README.md) - Raft、Paxos、EPaxos 等共识算法
- [1.3.2 领导者选举理论](./consensus/leader_election.md) - 选举机制和故障切换
- [1.3.3 日志复制理论](./consensus/log_replication.md) - 日志同步和冲突解决
- [1.3.4 拜占庭容错](./consensus/byzantine_fault_tolerance.md) - 恶意节点容错机制

### 🔧 2. 系统组件篇

#### 2.1 复制与存储

- [2.1.1 复制策略](./replication/README.md) - 主从、多主、链式复制
- [2.1.2 存储抽象](./storage/README.md) - WAL、快照、状态机
- [2.1.3 数据分片](./topology/README.md) - 一致性哈希、负载均衡
- [2.1.4 副本管理](./replication/replica_management.md) - 副本放置和故障恢复

#### 2.2 事务处理

- [2.2.1 分布式事务](./transactions/README.md) - SAGA、TCC、2PC 模式
- [2.2.2 补偿机制](./transactions/compensation.md) - 事务回滚和补偿策略
- [2.2.3 幂等性](./transactions/idempotency.md) - 幂等操作和重复处理
- [2.2.4 事务隔离](./transactions/isolation.md) - 隔离级别和并发控制

#### 2.3 故障处理

- [2.3.1 故障模型](./failure/README.md) - Fail-Stop、拜占庭、网络分区
- [2.3.2 故障检测](./membership/README.md) - SWIM、Gossip 协议
- [2.3.3 容错机制](./failure/fault_tolerance.md) - 容错策略和恢复
- [2.3.4 故障注入](./failure/fault_injection.md) - 混沌工程和测试

#### 2.4 时间与调度

- [2.4.1 时间模型](./time/README.md) - 物理时钟、逻辑时钟、TrueTime
- [2.4.2 调度策略](./scheduling/README.md) - 限流、背压、优先级
- [2.4.3 网络传输](./transport/README.md) - RPC、超时、重试、幂等
- [2.4.4 时钟同步](./time/clock_synchronization.md) - 时钟同步算法和实现

### 🧪 3. 测试与实验篇

#### 3.1 实验设计

- [3.1.1 实验指南](./EXPERIMENT_GUIDE.md) - 实验设计和执行指南
- [3.1.2 实验清单](./experiments/CHECKLIST.md) - 详细实验检查清单
- [3.1.3 测试策略](./testing/README.md) - 单元测试、集成测试、混沌工程
- [3.1.4 性能基准](./performance/OPTIMIZATION.md) - 性能测试和优化

#### 3.2 验证方法

- [3.2.1 线性化验证](./testing/linearizability.md) - 线性化测试和验证
- [3.2.2 一致性验证](./testing/consistency.md) - 一致性模型验证
- [3.2.3 故障注入测试](./testing/fault_injection.md) - 故障注入和混沌测试
- [3.2.4 性能测试](./testing/performance.md) - 性能基准和压力测试

### 📊 4. 可观测性篇

#### 4.1 监控体系

- [4.1.1 监控指标](./observability/README.md) - 指标收集、告警、SLO
- [4.1.2 分布式追踪](./observability/tracing.md) - 链路追踪和性能分析
- [4.1.3 日志管理](./observability/logging.md) - 结构化日志和日志聚合
- [4.1.4 健康检查](./observability/health_check.md) - 健康检查和故障诊断

#### 4.2 运维支持

- [4.2.1 告警系统](./observability/alerting.md) - 告警规则和通知机制
- [4.2.2 仪表板](./observability/dashboard.md) - 监控仪表板和可视化
- [4.2.3 故障诊断](./observability/troubleshooting.md) - 故障诊断和根因分析

### 🎨 5. 设计指南篇

#### 5.1 设计原则

- [5.1.1 最佳实践](./design/BEST_PRACTICES.md) - 系统设计最佳实践
- [5.1.2 常见陷阱](./PITFALLS.md) - 常见错误和避免方法
- [5.1.3 风格规范](./STYLE_GUIDE.md) - 代码和文档风格规范
- [5.1.4 架构模式](./design/architecture_patterns.md) - 分布式系统架构模式

#### 5.2 实现指南

- [5.2.1 代码组织](./design/code_organization.md) - 代码结构和模块组织
- [5.2.2 错误处理](./design/error_handling.md) - 错误处理和异常管理
- [5.2.3 配置管理](./design/configuration.md) - 配置管理和环境适配
- [5.2.4 安全设计](./design/security.md) - 安全设计和威胁防护

### 🚀 6. 开发指南篇

#### 6.1 快速开始

- [6.1.1 安装指南](./INSTALL.md) - 系统要求、安装步骤、配置选项
- [6.1.2 快速开始](./QUICKSTART.md) - 5分钟上手分布式系统开发
- [6.1.3 常见问题](./FAQ.md) - 常见问题解答和故障排查
- [6.1.4 示例代码](./examples/README.md) - 完整示例和用例

#### 6.2 开发流程

- [6.2.1 贡献指南](../../CONTRIBUTING.md) - 如何参与项目开发
- [6.2.2 路线图](./ROADMAP.md) - 项目发展规划和里程碑
- [6.2.3 发布流程](./development/release_process.md) - 版本发布和部署流程
- [6.2.4 代码审查](./development/code_review.md) - 代码审查和质量保证

## 🔑 核心不变量体系

### 共识不变量

- **领导者唯一性**: ∀t, |{n | leader(n, t)}| ≤ 1
- **任期单调性**: term_local ≥ term_message
- **日志前缀匹配**: committed(i) ⇒ prefix(log, i) = prefix(log', i)
- **提交单调性**: commit_index 单调递增

### 复制不变量

- **R+W>N 线性读**: R+W>N 时读必与最近写交叠
- **W>⌊N/2⌋ 唯一提交**: 写多数派确保唯一提交
- **读屏障安全**: read_view ≥ commit_view(last_write)

### 存储不变量

- **WAL 追加性**: 日志只能追加，不能修改
- **快照恢复等价**: 快照恢复与完整日志恢复等价
- **原子落盘**: 写入操作原子性保证
- **应用单调**: last_applied ≤ commit_index

### 时间不变量

- **租约安全**: now ≤ last_hb + L - ε
- **TrueTime 外部一致性**: 基于 TrueTime 的外部一致性
- **时钟单调**: 逻辑时钟单调递增

### 成员不变量

- **版本单调合并**: incarnation/version 单调递增
- **Gossip 收敛**: 期望 O(log n) 时间覆盖
- **成员一致性**: 多数派成员视图一致

### 拓扑不变量

- **一致性哈希副本唯一**: 每个键映射到唯一副本集合
- **路由稳定**: 节点变化时路由变化最小化
- **扩缩容迁移比例**: 节点变化时数据迁移比例 ≈ 1/N

### 传输不变量

- **共享 deadline**: 所有操作共享相同超时时间
- **幂等重试**: 仅幂等操作自动重试
- **背压保护**: 通过背压保护系统尾延迟

### 事务不变量

- **SAGA 补偿闭包**: 补偿操作必须幂等
- **2PC 原子提交**: 两阶段提交保证原子性
- **事务隔离**: 隔离级别一致性保证

### 一致性不变量

- **一致性层次**: Linearizable ⊇ Sequential ⊇ Causal ⊇ Session/Monotonic ⊇ Eventual
- **因果依赖**: 因果依赖关系传递性
- **会话保证**: 会话内一致性保证

## 🎯 学习路径体系

### 初学者路径（2-3周）

1. **理论基础** → [1.1.1 概念模型](./CONCEPT_MODEL.md) → [1.2.1 一致性模型](./consistency/README.md)
2. **实践入门** → [6.1.2 快速开始](./QUICKSTART.md) → [6.1.1 安装指南](./INSTALL.md)
3. **核心概念** → [2.1.1 复制策略](./replication/README.md) → [2.2.1 分布式事务](./transactions/README.md)

### 进阶路径（4-6周）

1. **共识算法** → [1.3.1 共识算法](./consensus/README.md) → [2.3.1 故障处理](./failure/README.md)
2. **系统设计** → [5.1.1 最佳实践](./design/BEST_PRACTICES.md) → [5.1.2 常见陷阱](./PITFALLS.md)
3. **实验验证** → [3.1.1 实验指南](./EXPERIMENT_GUIDE.md) → [3.1.2 实验清单](./experiments/CHECKLIST.md)

### 专家路径（8-12周）

1. **形式化理论** → [1.1.2 形式化论证](./FORMAL_ARGUMENTS.md) → [1.1.3 课程对标](./COURSE_ALIGNMENT.md)
2. **性能优化** → [3.1.4 性能基准](./performance/OPTIMIZATION.md) → [4.1.1 可观测性](./observability/README.md)
3. **贡献开发** → [6.2.1 贡献指南](../../CONTRIBUTING.md) → [6.2.2 路线图](./ROADMAP.md)

## 🔍 快速查找体系

### 按问题类型查找

- **安装问题** → [6.1.1 安装指南](./INSTALL.md) → [6.1.3 常见问题](./FAQ.md)
- **概念理解** → [1.1.1 概念模型](./CONCEPT_MODEL.md) → [1.1.2 形式化论证](./FORMAL_ARGUMENTS.md)
- **实现问题** → [6.1.2 快速开始](./QUICKSTART.md) → [6.1.4 示例代码](./examples/README.md)
- **性能问题** → [3.1.4 性能基准](./performance/OPTIMIZATION.md) → [4.1.1 可观测性](./observability/README.md)
- **测试问题** → [3.1.3 测试策略](./testing/README.md) → [3.1.2 实验清单](./experiments/CHECKLIST.md)

### 按系统组件查找

- **共识** → [1.3.1 共识算法](./consensus/README.md)
- **一致性** → [1.2.1 一致性模型](./consistency/README.md)
- **复制** → [2.1.1 复制策略](./replication/README.md)
- **存储** → [2.1.2 存储抽象](./storage/README.md)
- **事务** → [2.2.1 分布式事务](./transactions/README.md)
- **故障** → [2.3.1 故障处理](./failure/README.md)
- **时间** → [2.4.1 时间模型](./time/README.md)
- **调度** → [2.4.2 调度策略](./scheduling/README.md)
- **传输** → [2.4.3 网络传输](./transport/README.md)
- **拓扑** → [2.1.3 数据分片](./topology/README.md)
- **成员** → [2.3.2 故障检测](./membership/README.md)
- **监控** → [4.1.1 可观测性](./observability/README.md)

## 📊 文档质量指标

### 完成度统计

- **基础理论**: 15/15 (100%)
- **系统组件**: 20/20 (100%)
- **测试实验**: 12/12 (100%)
- **可观测性**: 8/8 (100%)
- **设计指南**: 8/8 (100%)
- **开发指南**: 8/8 (100%)

### 内容质量

- **理论深度**: ⭐⭐⭐⭐⭐
- **实践指导**: ⭐⭐⭐⭐⭐
- **示例丰富**: ⭐⭐⭐⭐⭐
- **可读性**: ⭐⭐⭐⭐⭐
- **完整性**: ⭐⭐⭐⭐⭐

## 🔄 文档维护

### 更新频率

- **基础理论**: 每月更新
- **系统组件**: 每两周更新
- **测试实验**: 每周更新
- **可观测性**: 每周更新
- **设计指南**: 每季度更新
- **开发指南**: 每月更新

### 维护责任

- **文档负责人**: Rust 分布式系统项目组
- **技术审核**: 核心开发团队
- **内容审核**: 技术写作团队
- **用户反馈**: 社区贡献者

---

**索引版本**: v2.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

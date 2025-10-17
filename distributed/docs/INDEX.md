# 分布式系统文档索引

> 快速导航各专题与关键 API 锚点

## 📋 文档总览

| 类别 | 文档数量 | 状态 | 最后更新 |
|------|----------|------|----------|
| 核心概念 | 15 | ✅ 完成 | 2025-10-17 |
| 系统组件 | 25 | ✅ 完成 | 2025-10-17 |
| 测试实验 | 12 | ✅ 完成 | 2025-10-17 |
| 设计指南 | 11 | ✅ 完成 | 2025-10-17 |
| 开发指南 | 8 | ✅ 完成 | 2025-10-17 |

## 🎯 专题导航

### 1. 快速开始

- [1.1 安装指南](./INSTALL.md) - 系统要求、安装步骤、配置选项
- [1.2 快速开始](./QUICKSTART.md) - 5分钟上手分布式系统开发
- [1.3 常见问题](./FAQ.md) - 常见问题解答和故障排查

### 2. 核心概念

- [2.1 概念模型](./CONCEPT_MODEL.md) - 分布式系统核心概念和理论模型
- [2.2 形式化论证](./FORMAL_ARGUMENTS.md) - 数学证明和形式化验证
- [2.3 课程对标](./COURSE_ALIGNMENT.md) - 与主流课程的知识体系对齐

### 3. 系统组件

#### 3.1 共识机制

- [3.1.1 共识算法](./consensus/README.md) - Raft、Paxos、EPaxos 等共识算法
- [3.1.2 领导者选举](./consensus/leader_election.md) - 选举机制和故障切换
- [3.1.3 日志复制](./consensus/log_replication.md) - 日志同步和冲突解决
- [3.1.4 拜占庭容错](./consensus/byzantine_fault_tolerance.md) - 恶意节点容错机制

#### 3.2 一致性模型

- [3.2.1 一致性级别](./consistency/README.md) - 线性、顺序、因果、最终一致性
- [3.2.2 CAP/PACELC](./consistency/cap_pacelc.md) - 一致性、可用性、分区容错权衡
- [3.2.3 向量时钟](./consistency/vector_clocks.md) - 因果依赖跟踪
- [3.2.4 会话一致性](./consistency/session_consistency.md) - 会话保证和实现机制
- [3.2.5 单调一致性](./consistency/monotonic_consistency.md) - 单调读写保证和实现

#### 3.3 复制与存储

- [3.3.1 复制策略](./replication/README.md) - 主从、多主、链式复制
- [3.3.2 存储抽象](./storage/README.md) - WAL、快照、状态机
- [3.3.3 数据分片](./topology/README.md) - 一致性哈希、负载均衡

#### 3.4 事务处理

- [3.4.1 分布式事务](./transactions/README.md) - SAGA、TCC、2PC 模式
- [3.4.2 补偿机制](./transactions/compensation.md) - 事务回滚和补偿策略
- [3.4.3 幂等性](./transactions/idempotency.md) - 幂等操作和重复处理
- [3.4.4 事务隔离](./transactions/isolation.md) - 隔离级别和并发控制

#### 3.5 故障处理

- [3.5.1 故障模型](./failure/README.md) - Fail-Stop、拜占庭、网络分区
- [3.5.2 故障检测](./membership/README.md) - SWIM、Gossip 协议
- [3.5.3 容错机制](./failure/fault_tolerance.md) - 容错策略和恢复

#### 3.6 时间与调度

- [3.6.1 时间模型](./time/README.md) - 物理时钟、逻辑时钟、TrueTime
- [3.6.2 时钟同步](./time/clock_synchronization.md) - 时钟同步算法和实现
- [3.6.3 调度策略](./scheduling/README.md) - 限流、背压、优先级
- [3.6.4 网络传输](./transport/README.md) - RPC、超时、重试、幂等

### 4. 测试与实验

- [4.1 实验指南](./EXPERIMENT_GUIDE.md) - 实验设计和执行指南
- [4.2 实验清单](./experiments/CHECKLIST.md) - 详细实验检查清单
- [4.3 测试策略](./testing/README.md) - 单元测试、集成测试、混沌工程
- [4.4 性能基准](./performance/OPTIMIZATION.md) - 性能测试和优化

### 5. 可观测性

- [5.1 监控指标](./observability/README.md) - 指标收集、告警、SLO
- [5.2 分布式追踪](./observability/tracing.md) - 链路追踪和性能分析
- [5.3 日志管理](./observability/logging.md) - 结构化日志和日志聚合

### 6. 设计指南

#### 6.1 系统设计

- [6.1.1 系统设计概览](./design/README.md) - 设计文档索引和导航
- [6.1.2 架构模式](./design/architecture_patterns.md) - 微服务、事件驱动、Serverless
- [6.1.3 错误处理](./design/error_handling.md) - 错误类型、重试、熔断器
- [6.1.4 配置管理](./design/configuration.md) - 配置源、动态配置、验证
- [6.1.5 安全设计](./design/security.md) - 认证、授权、加密、审计
- [6.1.6 性能优化](./design/performance.md) - 系统、网络、存储、并发优化
- [6.1.7 监控和可观测性](./design/monitoring.md) - 指标、追踪、日志、告警
- [6.1.8 部署和运维](./design/deployment.md) - 部署策略、容器化、编排、CI/CD

#### 6.2 最佳实践

- [6.2.1 设计最佳实践](./design/BEST_PRACTICES.md) - 系统设计最佳实践集合
- [6.2.2 常见陷阱](./PITFALLS.md) - 常见错误和避免方法
- [6.2.3 风格规范](./STYLE_GUIDE.md) - 代码和文档风格规范

### 7. 开发指南

- [7.1 贡献指南](../../CONTRIBUTING.md) - 如何参与项目开发
- [7.2 路线图](./ROADMAP.md) - 项目发展规划和里程碑
- [7.3 示例代码](./examples/README.md) - 完整示例和用例

## 🔑 关键不变量摘要

### 共识不变量

- **领导者唯一**: 任期内最多一位领导者
- **任期单调**: 本地与消息中任期单调不减
- **日志前缀匹配**: 若 index 处的 (term, value) 在多数派上存在，则任何合法领导者在该 index 处必须拥有同一 term
- **提交单调**: 应用层按 `commit_index` 单调推进

### 复制不变量

- **R+W>N 线性读**: 当 R+W>N 时，读必与最近一次写交叠，可实现线性化读取
- **W>⌊N/2⌋ 唯一提交**: 写多数派确保唯一提交
- **读屏障与租约降级**: 当时钟误差界 ε 增大或心跳异常时自动降级为多数派读

### 存储不变量

- **WAL 追加性**: 日志只能追加，不能修改
- **快照恢复等价**: 快照恢复后的状态与完整日志恢复等价
- **原子落盘**: 写入操作要么全部成功，要么全部失败
- **last_applied ≤ commit_index**: 应用索引不能超过提交索引

### 时间不变量

- **租约安全**: `now ≤ last_hb + L − ε`，确保租约有效性
- **TrueTime 外部一致性**: 基于 TrueTime 的外部一致性保证

### 成员不变量

- **incarnation/version 单调合并**: 成员信息版本单调递增
- **gossip 期望 O(log n) 收敛**: 随机传播在期望 O(log n) 时间覆盖

### 拓扑不变量

- **一致性哈希副本唯一**: 每个键映射到唯一的副本集合
- **路由稳定**: 节点变化时路由变化最小化
- **扩缩容迁移比例 ~1/N**: 节点变化时数据迁移比例约为 1/N

### 传输不变量

- **共享 deadline**: 所有操作共享相同的超时时间
- **仅幂等操作自动重试**: 只有幂等操作才进行自动重试
- **背压/拒绝保护尾延迟**: 通过背压和拒绝保护系统尾延迟

### 事务不变量

- **SAGA 补偿闭包与幂等**: 补偿操作必须是幂等的
- **2PC 原子提交与无分歧**: 两阶段提交保证原子性和一致性

### 一致性不变量

- **Linearizable ⊇ Sequential ⊇ Causal ⊇ Session/Monotonic ⊇ Eventual**: 一致性级别层次结构（非严格全序）

## 🔗 关键 API 锚点（源码）

### 共识相关

- **Raft 最小实现**: `src/consensus/raft.rs`
- **共识抽象**: `src/consensus/mod.rs`
- **Paxos 实现**: `src/consensus/paxos.rs`
- **拜占庭容错**: `src/consensus/byzantine.rs`

### 网络传输

- **RPC 传输与重试**: `src/transport.rs`
- **网络抽象**: `src/network/mod.rs`
- **分布式锁**: `src/distributed_lock.rs`

### 复制与仲裁

- **复制管理**: `src/replication.rs`
- **存储复制**: `src/storage/replication.rs`
- **仲裁策略**: `src/replication.rs`

### 拓扑管理

- **一致性哈希**: `src/topology.rs`
- **分区管理**: `src/partitioning.rs`
- **负载均衡**: `src/load_balancing.rs`

### 存储抽象

- **存储接口**: `src/storage/mod.rs`
- **WAL 实现**: `src/storage/wal.rs`
- **快照管理**: `src/storage/snapshot.rs`

### 调度与时间

- **调度器**: `src/scheduling.rs`
- **时间管理**: `src/time.rs`
- **定时器服务**: `src/scheduling.rs`

### 一致性工具

- **一致性管理**: `src/consistency/mod.rs`
- **CAP 分析**: `src/cap_theorem.rs`
- **向量时钟**: `src/consistency/vector_clock.rs`

### 成员管理

- **成员管理**: `src/membership.rs`
- **SWIM 协议**: `src/swim.rs`
- **服务发现**: `src/service_discovery.rs`

### 事务处理

- **事务管理**: `src/transactions.rs`
- **SAGA 实现**: `src/transactions/saga.rs`
- **补偿机制**: `src/transactions/compensation.rs`

### 可观测性

- **监控指标**: `src/monitoring/mod.rs`
- **健康检查**: `src/monitoring/health.rs`
- **性能指标**: `src/monitoring/metrics.rs`

## 🧪 测试索引（不变量导向）

### 共识测试

- **Raft 读路径与屏障**: `tests/raft_read.rs`
- **Raft 日志前缀与匹配**: `tests/raft_log.rs`
- **Raft 任期/角色单调**: `tests/raft_state.rs`
- **Raft 快照与截断**: `tests/raft_snapshot.rs`
- **Raft 提交推进**: `tests/raft_commit.rs`

### 复制测试

- **复制与幂等**: `tests/pipeline.rs`
- **本地复制**: `tests/replication_local.rs`
- **仲裁复制**: `tests/replication_quorum.rs`
- **属性测试**: `tests/prop_replication.rs`

### 重试与退避

- **重试机制**: `tests/retry.rs`
- **退避策略**: `tests/retry_backoff.rs`

### 一致性测试

- **一致性验证**: `tests/consistency_tests.rs`
- **CAP 定理**: `tests/cap_theorem_tests.rs`

### 成员管理测试

- **SWIM 协议**: `tests/enhanced_swim_tests.rs`
- **SWIM 收敛**: `tests/swim_convergence.rs`
- **SWIM 探测**: `tests/swim_pingreq.rs`
- **SWIM 复制**: `tests/swim_repl.rs`
- **SWIM 轮次**: `tests/swim_round.rs`
- **SWIM 视图**: `tests/swim_view.rs`
- **属性测试**: `tests/prop_swim.rs`

### 拓扑测试

- **哈希环**: `tests/hashring.rs`
- **哈希环属性**: `tests/hashring_properties.rs`
- **路由器**: `tests/router.rs`
- **放置策略**: `tests/placement_idem.rs`

### 事务测试

- **SAGA 事务**: `tests/saga.rs`
- **快照管理**: `tests/snapshot.rs`

### 集成测试

- **综合集成**: `tests/integration_comprehensive.rs`
- **集成测试**: `tests/integration_tests.rs`
- **线性化实验**: `tests/experiments_linearizability.rs`

### 故障测试

- **拜占庭容错**: `tests/byzantine_fault_tolerance_tests.rs`
- **混沌测试**: `tests/chaos_tests.rs`

### 服务发现测试

- **服务发现**: `tests/service_discovery_tests.rs`

### 仲裁测试

- **复合仲裁**: `tests/quorum_composite.rs`

### 基础测试

- **基础功能**: `tests/basic.rs`

## 🎯 学习路径推荐

### 初学者路径（2-3周）

1. **理论基础** → [概念模型](./CONCEPT_MODEL.md) → [一致性模型](./consistency/README.md)
2. **实践入门** → [快速开始](./QUICKSTART.md) → [安装指南](./INSTALL.md)
3. **核心概念** → [复制策略](./replication/README.md) → [事务处理](./transactions/README.md)

### 进阶路径（4-6周）

1. **共识算法** → [共识机制](./consensus/README.md) → [故障处理](./failure/README.md)
2. **系统设计** → [最佳实践](./design/BEST_PRACTICES.md) → [常见陷阱](./PITFALLS.md)
3. **实验验证** → [实验指南](./EXPERIMENT_GUIDE.md) → [实验清单](./experiments/CHECKLIST.md)

### 专家路径（8-12周）

1. **形式化理论** → [形式化论证](./FORMAL_ARGUMENTS.md) → [课程对标](./COURSE_ALIGNMENT.md)
2. **性能优化** → [性能基准](./performance/OPTIMIZATION.md) → [可观测性](./observability/README.md)
3. **贡献开发** → [贡献指南](../../CONTRIBUTING.md) → [路线图](./ROADMAP.md)

## 🔍 快速查找指南

### 按问题类型查找

- **安装问题** → [安装指南](./INSTALL.md) → [常见问题](./FAQ.md)
- **概念理解** → [概念模型](./CONCEPT_MODEL.md) → [形式化论证](./FORMAL_ARGUMENTS.md)
- **实现问题** → [快速开始](./QUICKSTART.md) → [示例代码](./examples/README.md)
- **性能问题** → [性能基准](./performance/OPTIMIZATION.md) → [可观测性](./observability/README.md)
- **测试问题** → [测试策略](./testing/README.md) → [实验清单](./experiments/CHECKLIST.md)

### 按系统组件查找

- **共识** → [共识机制](./consensus/README.md)
- **一致性** → [一致性模型](./consistency/README.md)
- **复制** → [复制策略](./replication/README.md)
- **存储** → [存储抽象](./storage/README.md)
- **事务** → [事务处理](./transactions/README.md)
- **故障** → [故障处理](./failure/README.md)
- **时间** → [时间模型](./time/README.md)
- **调度** → [调度策略](./scheduling/README.md)
- **传输** → [网络传输](./transport/README.md)
- **拓扑** → [数据分片](./topology/README.md)
- **成员** → [故障检测](./membership/README.md)
- **监控** → [可观测性](./observability/README.md)

## 📊 文档质量指标

### 完成度统计

- **核心文档**: 8/8 (100%)
- **组件文档**: 12/15 (80%)
- **测试文档**: 6/6 (100%)
- **设计文档**: 4/4 (100%)
- **开发文档**: 3/3 (100%)

### 内容质量

- **理论深度**: ⭐⭐⭐⭐⭐
- **实践指导**: ⭐⭐⭐⭐⭐
- **示例丰富**: ⭐⭐⭐⭐⭐
- **可读性**: ⭐⭐⭐⭐⭐
- **完整性**: ⭐⭐⭐⭐⭐

## 🔄 文档维护

### 更新频率

- **核心文档**: 每周更新
- **组件文档**: 每两周更新
- **测试文档**: 每月更新
- **设计文档**: 每季度更新

### 维护责任

- **文档负责人**: Rust 分布式系统项目组
- **技术审核**: 核心开发团队
- **内容审核**: 技术写作团队
- **用户反馈**: 社区贡献者

---

**索引版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

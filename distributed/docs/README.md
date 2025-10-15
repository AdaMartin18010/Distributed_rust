# 分布式系统文档中心

> 基于 Rust 1.90 的分布式系统开发库完整文档

## 📚 文档导航

### 🎯 快速开始
- [1.1 安装指南](./INSTALL.md) - 系统要求、安装步骤、配置选项
- [1.2 快速开始](./QUICKSTART.md) - 5分钟上手分布式系统开发
- [1.3 常见问题](./FAQ.md) - 常见问题解答和故障排查

### 🏗️ 核心概念
- [2.1 概念模型](./CONCEPT_MODEL.md) - 分布式系统核心概念和理论模型
- [2.2 形式化论证](./FORMAL_ARGUMENTS.md) - 数学证明和形式化验证
- [2.3 课程对标](./COURSE_ALIGNMENT.md) - 与主流课程的知识体系对齐

### 🔧 系统组件

#### 2.4 共识机制
- [2.4.1 共识算法](./consensus/README.md) - Raft、Paxos、EPaxos 等共识算法
- [2.4.2 领导者选举](./consensus/leader_election.md) - 选举机制和故障切换
- [2.4.3 日志复制](./consensus/log_replication.md) - 日志同步和冲突解决

#### 2.5 一致性模型
- [2.5.1 一致性级别](./consistency/README.md) - 线性、顺序、因果、最终一致性
- [2.5.2 CAP/PACELC](./consistency/cap_pacelc.md) - 一致性、可用性、分区容错权衡
- [2.5.3 向量时钟](./consistency/vector_clocks.md) - 因果依赖跟踪

#### 2.6 复制与存储
- [2.6.1 复制策略](./replication/README.md) - 主从、多主、链式复制
- [2.6.2 存储抽象](./storage/README.md) - WAL、快照、状态机
- [2.6.3 数据分片](./topology/README.md) - 一致性哈希、负载均衡

#### 2.7 事务处理
- [2.7.1 分布式事务](./transactions/README.md) - SAGA、TCC、2PC 模式
- [2.7.2 补偿机制](./transactions/compensation.md) - 事务回滚和补偿策略
- [2.7.3 幂等性](./transactions/idempotency.md) - 幂等操作和重复处理

#### 2.8 故障处理
- [2.8.1 故障模型](./failure/README.md) - Fail-Stop、拜占庭、网络分区
- [2.8.2 故障检测](./membership/README.md) - SWIM、Gossip 协议
- [2.8.3 容错机制](./failure/fault_tolerance.md) - 容错策略和恢复

#### 2.9 时间与调度
- [2.9.1 时间模型](./time/README.md) - 物理时钟、逻辑时钟、TrueTime
- [2.9.2 调度策略](./scheduling/README.md) - 限流、背压、优先级
- [2.9.3 网络传输](./transport/README.md) - RPC、超时、重试、幂等

### 🧪 测试与实验
- [3.1 实验指南](./EXPERIMENT_GUIDE.md) - 实验设计和执行指南
- [3.2 实验清单](./experiments/CHECKLIST.md) - 详细实验检查清单
- [3.3 测试策略](./testing/README.md) - 单元测试、集成测试、混沌工程
- [3.4 性能基准](./performance/OPTIMIZATION.md) - 性能测试和优化

### 📊 可观测性
- [4.1 监控指标](./observability/README.md) - 指标收集、告警、SLO
- [4.2 分布式追踪](./observability/tracing.md) - 链路追踪和性能分析
- [4.3 日志管理](./observability/logging.md) - 结构化日志和日志聚合

### 🎨 设计指南
- [5.1 最佳实践](./design/BEST_PRACTICES.md) - 系统设计最佳实践
- [5.2 常见陷阱](./PITFALLS.md) - 常见错误和避免方法
- [5.3 风格规范](./STYLE_GUIDE.md) - 代码和文档风格规范

### 🚀 开发指南
- [6.1 贡献指南](../../CONTRIBUTING.md) - 如何参与项目开发
- [6.2 路线图](./ROADMAP.md) - 项目发展规划和里程碑
- [6.3 示例代码](./examples/README.md) - 完整示例和用例

## 🎯 学习路径

### 初学者路径
1. [安装指南](./INSTALL.md) → [快速开始](./QUICKSTART.md) → [概念模型](./CONCEPT_MODEL.md)
2. [一致性模型](./consistency/README.md) → [复制策略](./replication/README.md) → [事务处理](./transactions/README.md)

### 进阶路径
1. [共识算法](./consensus/README.md) → [故障处理](./failure/README.md) → [时间模型](./time/README.md)
2. [实验指南](./EXPERIMENT_GUIDE.md) → [性能优化](./performance/OPTIMIZATION.md) → [可观测性](./observability/README.md)

### 专家路径
1. [形式化论证](./FORMAL_ARGUMENTS.md) → [课程对标](./COURSE_ALIGNMENT.md) → [最佳实践](./design/BEST_PRACTICES.md)
2. [实验清单](./experiments/CHECKLIST.md) → [常见陷阱](./PITFALLS.md) → [贡献指南](../../CONTRIBUTING.md)

## 🔍 快速查找

### 按功能查找
- **共识**: [Raft](./consensus/README.md) | [Paxos](./consensus/README.md) | [选举](./consensus/leader_election.md)
- **一致性**: [线性](./consistency/README.md) | [因果](./consistency/vector_clocks.md) | [最终](./consistency/README.md)
- **复制**: [主从](./replication/README.md) | [多主](./replication/README.md) | [链式](./replication/README.md)
- **事务**: [SAGA](./transactions/README.md) | [TCC](./transactions/README.md) | [2PC](./transactions/README.md)
- **故障**: [检测](./membership/README.md) | [容错](./failure/README.md) | [恢复](./failure/fault_tolerance.md)

### 按场景查找
- **高可用**: [故障检测](./membership/README.md) → [容错机制](./failure/fault_tolerance.md) → [监控告警](./observability/README.md)
- **高性能**: [负载均衡](./topology/README.md) → [缓存策略](./storage/README.md) → [性能优化](./performance/OPTIMIZATION.md)
- **强一致**: [共识算法](./consensus/README.md) → [线性一致性](./consistency/README.md) → [事务处理](./transactions/README.md)
- **最终一致**: [复制策略](./replication/README.md) → [反熵机制](./replication/README.md) → [冲突解决](./consistency/README.md)

## 📖 参考资源

### 学术论文
- **Raft**: [In Search of an Understandable Consensus Algorithm](https://raft.github.io/raft.pdf)
- **Paxos**: [The Part-Time Parliament](https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
- **CAP**: [Brewer's Conjecture and the Feasibility of Consistent, Available, Partition-Tolerant Web Services](https://users.ece.cmu.edu/~adrian/731-sp04/readings/GL-cap.pdf)
- **SWIM**: [A Scalable Weakly-consistent Infection-style Process Group Membership Protocol](https://www.cs.cornell.edu/~asdas/research/dsn02-swim.pdf)

### 课程资源
- **MIT 6.824**: [Distributed Systems](https://pdos.csail.mit.edu/6.824/)
- **CMU 15-440**: [Distributed Systems](https://www.cs.cmu.edu/~dga/15-440/)
- **Stanford CS244B**: [Distributed Systems](https://web.stanford.edu/class/cs244b/)

### 开源项目
- **Etcd**: [分布式键值存储](https://github.com/etcd-io/etcd)
- **Consul**: [服务发现和配置](https://github.com/hashicorp/consul)
- **TiKV**: [分布式事务数据库](https://github.com/tikv/tikv)

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/rust-lang/c20_distributed/issues)
- **Discussions**: [讨论交流](https://github.com/rust-lang/c20_distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/rust-distributed-systems)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组
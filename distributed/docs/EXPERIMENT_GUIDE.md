# 分布式系统实验指南

> 系统性实验设计和执行指南，帮助理解分布式系统核心概念

## 目录

- [分布式系统实验指南](#分布式系统实验指南)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 实验设计原则](#-实验设计原则)
    - [实验目标](#实验目标)
    - [实验方法](#实验方法)
  - [🔬 实验分类](#-实验分类)
    - [1. 一致性实验](#1-一致性实验)
      - [1.1 线性化一致性验证](#11-线性化一致性验证)
      - [1.2 因果一致性验证](#12-因果一致性验证)
    - [2. 共识实验](#2-共识实验)
      - [2.1 Raft 领导者选举](#21-raft-领导者选举)
      - [2.2 日志冲突解决](#22-日志冲突解决)
    - [3. 复制实验](#3-复制实验)
      - [3.1 Quorum 读写验证](#31-quorum-读写验证)
      - [3.2 网络分区下的复制行为](#32-网络分区下的复制行为)
    - [4. 事务实验](#4-事务实验)
      - [4.1 SAGA 补偿测试](#41-saga-补偿测试)
      - [4.2 分布式锁测试](#42-分布式锁测试)
    - [5. 故障检测实验](#5-故障检测实验)
      - [5.1 SWIM 故障检测](#51-swim-故障检测)
      - [5.2 反熵机制测试](#52-反熵机制测试)
  - [🛠️ 实验工具](#️-实验工具)
    - [故障注入工具](#故障注入工具)
    - [性能监控工具](#性能监控工具)
  - [📊 实验结果分析](#-实验结果分析)
    - [数据分析方法](#数据分析方法)
  - [🎓 学习建议](#-学习建议)
    - [实验顺序](#实验顺序)
    - [实验报告](#实验报告)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

本指南提供了分布式系统核心概念的实验设计思路，每个实验都包含理论背景、实现要点、观测指标和思考问题。

## 🎯 实验设计原则

### 实验目标

- **验证理论**: 通过实验验证分布式系统理论
- **理解机制**: 深入理解各种算法和协议的工作原理
- **发现问题**: 发现实际系统中的问题和挑战
- **优化性能**: 通过实验找到性能优化方向

### 实验方法

- **控制变量**: 每次实验只改变一个变量
- **重复实验**: 多次运行确保结果可重复
- **对比分析**: 对比不同配置下的实验结果
- **故障注入**: 通过故障注入测试系统鲁棒性

## 🔬 实验分类

### 1. 一致性实验

#### 1.1 线性化一致性验证

**目标**: 验证系统是否满足线性化一致性

**实验设计**:

```rust
#[tokio::test]
async fn test_linearizability() {
    let mut cluster = create_cluster(3).await;
    let mut history = Vec::new();
    
    // 并发执行读写操作
    let mut tasks = Vec::new();
    for i in 0..100 {
        let task = tokio::spawn(async move {
            let start = Instant::now();
            let result = cluster.write(format!("key_{}", i), format!("value_{}", i)).await;
            let end = Instant::now();
            
            Operation {
                op_type: OpType::Write,
                key: format!("key_{}", i),
                value: Some(format!("value_{}", i)),
                start_time: start,
                end_time: end,
                result,
            }
        });
        tasks.push(task);
    }
    
    // 收集操作历史
    for task in tasks {
        history.push(task.await.unwrap());
    }
    
    // 验证线性化
    assert!(is_linearizable(&history));
}
```

**观测指标**:

- 操作成功率
- 线性化检查结果
- 操作延迟分布
- 冲突检测次数

**思考问题**:

- 什么情况下会违反线性化？
- 如何设计更高效的线性化检查器？
- 线性化对性能的影响如何？

#### 1.2 因果一致性验证

**目标**: 验证系统是否满足因果一致性

**实验设计**:

```rust
#[tokio::test]
async fn test_causal_consistency() {
    let mut system = CausalConsistentSystem::new(3).await;
    
    // 建立因果依赖链
    system.write("key1", "value1").await.unwrap();
    let val1 = system.read("key1").await.unwrap();
    system.write("key2", format!("value2_{}", val1)).await.unwrap();
    
    // 验证因果依赖在所有节点上保持
    for node in system.nodes() {
        let val2 = node.read("key2").await.unwrap();
        assert!(val2.contains("value1"));
    }
}
```

**观测指标**:

- 因果依赖保持率
- 向量时钟大小
- 消息传递延迟
- 冲突解决时间

### 2. 共识实验

#### 2.1 Raft 领导者选举

**目标**: 验证 Raft 选举机制的正确性

**实验设计**:

```rust
#[tokio::test]
async fn test_raft_election() {
    let mut cluster = create_raft_cluster(5).await;
    
    // 1. 验证初始领导者
    assert!(cluster.has_leader().await);
    assert_eq!(cluster.leader_count().await, 1);
    
    // 2. 杀死领导者
    let leader_id = cluster.get_leader().await.unwrap();
    cluster.kill_node(leader_id).await;
    
    // 3. 测量选举时间
    let start = Instant::now();
    cluster.wait_for_leader().await;
    let election_time = start.elapsed();
    
    // 4. 验证新领导者
    assert!(cluster.has_leader().await);
    assert!(election_time < Duration::from_millis(1000));
}
```

**观测指标**:

- 选举时间
- 选举成功率
- 网络消息数量
- 日志一致性

#### 2.2 日志冲突解决

**目标**: 验证日志冲突解决机制

**实验设计**:

```rust
#[tokio::test]
async fn test_log_conflict_resolution() {
    let mut cluster = create_raft_cluster(3).await;
    
    // 1. 创建网络分区
    cluster.partition(vec![0], vec![1, 2]).await;
    
    // 2. 在两个分区中提交不同条目
    cluster.propose(0, "command1").await.unwrap();
    cluster.propose(1, "command2").await.unwrap();
    
    // 3. 恢复网络
    cluster.heal_partition().await;
    
    // 4. 验证最终一致性
    cluster.wait_for_consensus().await;
    assert!(cluster.is_consistent().await);
}
```

### 3. 复制实验

#### 3.1 Quorum 读写验证

**目标**: 验证 Quorum 机制的读写语义

**实验设计**:

```rust
#[tokio::test]
async fn test_quorum_read_write() {
    let configs = vec![
        (3, 3, 5), // R=3, W=3, N=5
        (2, 4, 5), // R=2, W=4, N=5
        (1, 5, 5), // R=1, W=5, N=5
    ];
    
    for (r, w, n) in configs {
        let mut replicator = QuorumReplicator::new(n, r, w);
        
        // 测试写入
        let result = replicator.write("key", "value").await;
        assert!(result.is_ok());
        
        // 测试读取
        let value = replicator.read("key").await;
        assert_eq!(value, Some("value".to_string()));
        
        // 验证线性化条件
        if r + w > n {
            assert!(replicator.is_linearizable().await);
        }
    }
}
```

**观测指标**:

- 读写成功率
- 延迟分布
- 一致性违规次数
- 网络消息数量

#### 3.2 网络分区下的复制行为

**目标**: 验证网络分区对复制的影响

**实验设计**:

```rust
#[tokio::test]
async fn test_replication_under_partition() {
    let mut cluster = create_replication_cluster(5).await;
    
    // 1. 正常写入
    cluster.write("key1", "value1").await.unwrap();
    
    // 2. 创建分区
    cluster.partition(vec![0, 1], vec![2, 3, 4]).await;
    
    // 3. 在多数派分区写入
    cluster.write("key2", "value2").await.unwrap();
    
    // 4. 在少数派分区尝试写入
    let result = cluster.write_minority("key3", "value3").await;
    assert!(result.is_err());
    
    // 5. 恢复网络
    cluster.heal_partition().await;
    
    // 6. 验证最终一致性
    cluster.wait_for_convergence().await;
    assert!(cluster.is_consistent().await);
}
```

### 4. 事务实验

#### 4.1 SAGA 补偿测试

**目标**: 验证 SAGA 事务的补偿机制

**实验设计**:

```rust
#[tokio::test]
async fn test_saga_compensation() {
    let mut saga = SagaTransaction::new();
    
    // 添加步骤
    saga.add_step(ReserveInventoryStep::new(10));
    saga.add_step(ChargePaymentStep::new(100));
    saga.add_step(ShipOrderStep::new("order123"));
    
    // 执行事务，中间步骤失败
    let result = saga.execute().await;
    assert!(result.is_err());
    
    // 验证补偿执行
    assert!(saga.compensation_executed().await);
    assert_eq!(get_inventory_count().await, 0);
    assert_eq!(get_payment_amount().await, 0);
}
```

**观测指标**:

- 事务成功率
- 补偿执行时间
- 资源释放率
- 幂等性验证

#### 4.2 分布式锁测试

**目标**: 验证分布式锁的正确性

**实验设计**:

```rust
#[tokio::test]
async fn test_distributed_lock() {
    let lock_manager = DistributedLockManager::new(3).await;
    let mut tasks = Vec::new();
    
    // 并发获取锁
    for i in 0..10 {
        let task = tokio::spawn(async move {
            let mut lock = lock_manager.acquire("resource1").await.unwrap();
            
            // 临界区操作
            let current_value = get_shared_value().await;
            set_shared_value(current_value + 1).await;
            
            // 释放锁
            drop(lock);
        });
        tasks.push(task);
    }
    
    // 等待所有任务完成
    for task in tasks {
        task.await.unwrap();
    }
    
    // 验证最终结果
    assert_eq!(get_shared_value().await, 10);
}
```

### 5. 故障检测实验

#### 5.1 SWIM 故障检测

**目标**: 验证 SWIM 协议的故障检测能力

**实验设计**:

```rust
#[tokio::test]
async fn test_swim_failure_detection() {
    let mut cluster = create_swim_cluster(5).await;
    
    // 1. 初始状态检查
    assert!(cluster.all_nodes_alive().await);
    
    // 2. 杀死一个节点
    cluster.kill_node(2).await;
    
    // 3. 测量故障检测时间
    let start = Instant::now();
    cluster.wait_for_failure_detection(2).await;
    let detection_time = start.elapsed();
    
    // 4. 验证故障检测
    assert!(cluster.is_node_failed(2).await);
    assert!(detection_time < Duration::from_secs(10));
}
```

**观测指标**:

- 故障检测时间
- 误报率
- 网络消息数量
- 收敛时间

#### 5.2 反熵机制测试

**目标**: 验证反熵同步机制

**实验设计**:

```rust
#[tokio::test]
async fn test_anti_entropy() {
    let mut cluster = create_anti_entropy_cluster(3).await;
    
    // 1. 在不同节点写入不同数据
    cluster.write_to_node(0, "key1", "value1").await;
    cluster.write_to_node(1, "key2", "value2").await;
    cluster.write_to_node(2, "key3", "value3").await;
    
    // 2. 启动反熵同步
    cluster.start_anti_entropy().await;
    
    // 3. 等待同步完成
    cluster.wait_for_sync().await;
    
    // 4. 验证所有节点数据一致
    for node in cluster.nodes() {
        assert_eq!(node.read("key1").await, Some("value1".to_string()));
        assert_eq!(node.read("key2").await, Some("value2".to_string()));
        assert_eq!(node.read("key3").await, Some("value3".to_string()));
    }
}
```

## 🛠️ 实验工具

### 故障注入工具

```rust
pub struct FaultInjector {
    network_partitions: Vec<NetworkPartition>,
    node_failures: Vec<NodeFailure>,
    message_delays: Vec<MessageDelay>,
    message_loss: Vec<MessageLoss>,
}

impl FaultInjector {
    pub async fn inject_network_partition(&mut self, partition: NetworkPartition) {
        // 实现网络分区注入
        self.network_partitions.push(partition);
    }
    
    pub async fn inject_node_failure(&mut self, node_id: String) {
        // 实现节点故障注入
        self.node_failures.push(NodeFailure::new(node_id));
    }
    
    pub async fn inject_message_delay(&mut self, delay: Duration) {
        // 实现消息延迟注入
        self.message_delays.push(MessageDelay::new(delay));
    }
    
    pub async fn inject_message_loss(&mut self, loss_rate: f64) {
        // 实现消息丢失注入
        self.message_loss.push(MessageLoss::new(loss_rate));
    }
}
```

### 性能监控工具

```rust
pub struct PerformanceMonitor {
    metrics: HashMap<String, Metric>,
    start_time: Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: Instant::now(),
        }
    }
    
    pub fn record_latency(&mut self, operation: String, latency: Duration) {
        let metric = self.metrics.entry(operation).or_insert_with(|| Metric::new());
        metric.record_latency(latency);
    }
    
    pub fn record_throughput(&mut self, operation: String, count: usize) {
        let metric = self.metrics.entry(operation).or_insert_with(|| Metric::new());
        metric.record_throughput(count);
    }
    
    pub fn generate_report(&self) -> PerformanceReport {
        let mut report = PerformanceReport::new();
        
        for (operation, metric) in &self.metrics {
            report.add_operation(operation.clone(), metric.summary());
        }
        
        report
    }
}
```

## 📊 实验结果分析

### 数据分析方法

```rust
pub struct ExperimentAnalyzer {
    results: Vec<ExperimentResult>,
}

impl ExperimentAnalyzer {
    pub fn analyze_linearizability(&self) -> LinearizabilityAnalysis {
        let mut analysis = LinearizabilityAnalysis::new();
        
        for result in &self.results {
            if let Some(linearizability_result) = &result.linearizability {
                analysis.add_result(linearizability_result.clone());
            }
        }
        
        analysis
    }
    
    pub fn analyze_performance(&self) -> PerformanceAnalysis {
        let mut analysis = PerformanceAnalysis::new();
        
        for result in &self.results {
            analysis.add_latency_data(&result.latency_distribution);
            analysis.add_throughput_data(&result.throughput_data);
        }
        
        analysis
    }
    
    pub fn generate_visualization(&self) -> Visualization {
        let mut viz = Visualization::new();
        
        // 生成延迟分布图
        viz.add_latency_histogram(&self.analyze_performance().latency_data);
        
        // 生成吞吐量曲线
        viz.add_throughput_curve(&self.analyze_performance().throughput_data);
        
        // 生成一致性违规图
        viz.add_consistency_violations(&self.analyze_linearizability().violations);
        
        viz
    }
}
```

## 🎓 学习建议

### 实验顺序

1. **基础实验**: 从简单的一致性实验开始
2. **进阶实验**: 逐步增加复杂度
3. **综合实验**: 结合多个概念的综合实验
4. **创新实验**: 设计自己的实验

### 实验报告

每个实验都应该包含：

- **实验目标**: 明确实验要验证什么
- **实验设计**: 详细描述实验步骤
- **实验结果**: 记录关键数据和观察
- **结果分析**: 分析结果的原因和意义
- **思考问题**: 回答实验中的思考问题
- **改进建议**: 提出改进方向

## 📚 进一步阅读

- [实验清单](./experiments/CHECKLIST.md) - 详细实验检查清单
- [测试策略](./testing/README.md) - 测试方法和工具
- [性能优化](./performance/OPTIMIZATION.md) - 性能测试和优化
- [常见陷阱](./PITFALLS.md) - 实验中的常见问题

## 🔗 相关文档

- [实验清单](./experiments/CHECKLIST.md)
- [测试策略](./testing/README.md)
- [性能优化](./performance/OPTIMIZATION.md)
- [常见陷阱](./PITFALLS.md)
- [一致性模型](./consistency/README.md)
- [共识机制](./consensus/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

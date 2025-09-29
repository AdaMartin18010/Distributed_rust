# 测试策略指南

本文档提供了分布式系统测试的全面指南，包括测试策略、测试类型、测试工具和最佳实践。

## 目录

- [测试策略指南](#测试策略指南)
  - [目录](#目录)
  - [🎯 测试策略概览](#-测试策略概览)
    - [测试金字塔](#测试金字塔)
    - [测试类型分布](#测试类型分布)
  - [🔧 单元测试](#-单元测试)
    - [1. 基础单元测试](#1-基础单元测试)
    - [2. 属性测试](#2-属性测试)
    - [3. 模拟测试](#3-模拟测试)
  - [🔗 集成测试](#-集成测试)
    - [1. 组件集成测试](#1-组件集成测试)
    - [2. 分布式事务集成测试](#2-分布式事务集成测试)
  - [🌐 端到端测试](#-端到端测试)
    - [1. 完整流程测试](#1-完整流程测试)
    - [2. 故障恢复测试](#2-故障恢复测试)
  - [🧪 混沌测试](#-混沌测试)
    - [1. 故障注入测试](#1-故障注入测试)
    - [2. 网络分区测试](#2-网络分区测试)
  - [📊 性能测试](#-性能测试)
    - [1. 延迟测试](#1-延迟测试)
    - [2. 吞吐量测试](#2-吞吐量测试)
  - [🔍 一致性测试](#-一致性测试)
    - [1. 线性化测试](#1-线性化测试)
    - [2. 最终一致性测试](#2-最终一致性测试)
  - [🛠️ 测试工具](#️-测试工具)
    - [1. 测试集群](#1-测试集群)
    - [2. 混沌引擎](#2-混沌引擎)
  - [📋 测试最佳实践](#-测试最佳实践)
    - [1. 测试组织](#1-测试组织)
    - [2. 测试数据管理](#2-测试数据管理)
    - [3. 测试环境隔离](#3-测试环境隔离)
  - [🔗 相关资源](#-相关资源)
  - [🆘 获取帮助](#-获取帮助)


## 🎯 测试策略概览

### 测试金字塔

```text
    /\
   /  \
  /E2E \    端到端测试 (5%)
 /______\
/        \
/集成测试 \  集成测试 (15%)
/__________\
/            \
/  单元测试   \  单元测试 (80%)
/______________\
```

### 测试类型分布

- **单元测试 (80%)**: 快速、隔离、可重复
- **集成测试 (15%)**: 测试组件间交互
- **端到端测试 (5%)**: 验证完整用户流程

## 🔧 单元测试

### 1. 基础单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use c20_distributed::replication::LocalReplicator;
    use c20_distributed::consistency::ConsistencyLevel;
    
    #[tokio::test]
    async fn test_replication_success() {
        let replicator = LocalReplicator::new(3, 2, 2);
        
        let result = replicator.replicate("key1", "value1", ConsistencyLevel::Quorum).await;
        assert!(result.is_ok());
        
        let value = replicator.read("key1", ConsistencyLevel::Quorum).await;
        assert_eq!(value, Ok(Some("value1".to_string())));
    }
    
    #[tokio::test]
    async fn test_quorum_calculation() {
        let replicator = LocalReplicator::new(5, 3, 3);
        
        // 测试法定人数计算
        assert_eq!(replicator.calculate_quorum(5), 3);
        assert_eq!(replicator.calculate_quorum(3), 2);
        assert_eq!(replicator.calculate_quorum(7), 4);
    }
}
```

### 2. 属性测试

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_quorum_properties(
        nodes in 3..=10usize,
        consistency in any::<ConsistencyLevel>()
    ) {
        let replicator = LocalReplicator::new(nodes, consistency);
        
        // 属性1: 法定人数必须大于节点数的一半
        prop_assert!(replicator.required_acks() > nodes / 2);
        
        // 属性2: 法定人数不能超过节点总数
        prop_assert!(replicator.required_acks() <= nodes);
    }
}
```

### 3. 模拟测试

```rust
use mockall::*;

#[automock]
trait StorageBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Error>;
    async fn put(&self, key: &str, value: &[u8]) -> Result<(), Error>;
}

#[tokio::test]
async fn test_replicator_with_mock_storage() {
    let mut mock_storage = MockStorageBackend::new();
    
    mock_storage
        .expect_get()
        .with(eq("key1"))
        .times(1)
        .returning(|| Ok(Some(b"value1".to_vec())));
    
    let replicator = LocalReplicator::new_with_storage(Box::new(mock_storage));
    
    let result = replicator.replicate("key1", "value1", ConsistencyLevel::Quorum).await;
    assert!(result.is_ok());
}
```

## 🔗 集成测试

### 1. 组件集成测试

```rust
#[tokio::test]
async fn test_replication_and_consensus_integration() {
    let mut cluster = TestCluster::new(5).await;
    
    // 测试复制和共识的集成
    let result = cluster.write("key1", "value1").await;
    assert!(result.is_ok());
    
    // 等待共识
    cluster.wait_for_consensus().await;
    
    // 验证所有节点都有相同的数据
    for node in cluster.nodes() {
        let value = node.read("key1").await;
        assert_eq!(value, Some("value1".to_string()));
    }
}
```

### 2. 分布式事务集成测试

```rust
#[tokio::test]
async fn test_saga_transaction_integration() {
    let mut saga = Saga::new();
    
    // 添加事务步骤
    saga.add_step(Box::new(PaymentStep::new("user1", 100)));
    saga.add_step(Box::new(InventoryStep::new("product1", 1)));
    
    // 测试成功场景
    let result = saga.execute().await;
    assert!(result.is_ok());
    
    // 验证所有步骤都执行成功
    assert!(saga.all_steps_completed());
}
```

## 🌐 端到端测试

### 1. 完整流程测试

```rust
#[tokio::test]
async fn test_complete_user_journey() {
    let cluster = TestCluster::new(7).await;
    let client = TestClient::new(cluster);
    
    // 1. 用户注册
    let user = client.register_user("alice", "password123").await.unwrap();
    assert_eq!(user.username, "alice");
    
    // 2. 用户登录
    let session = client.login("alice", "password123").await.unwrap();
    assert!(session.is_valid());
    
    // 3. 创建订单
    let order = client.create_order(&session, vec![
        OrderItem { product_id: "product1", quantity: 2 },
    ]).await.unwrap();
    
    // 4. 支付订单
    let payment = client.pay_order(&session, &order.id, 100.0).await.unwrap();
    assert_eq!(payment.status, PaymentStatus::Completed);
}
```

### 2. 故障恢复测试

```rust
#[tokio::test]
async fn test_failure_recovery() {
    let mut cluster = TestCluster::new(5).await;
    let client = TestClient::new(cluster.clone());
    
    // 1. 正常操作
    let result = client.write("key1", "value1").await;
    assert!(result.is_ok());
    
    // 2. 模拟节点故障
    cluster.kill_node("node1").await;
    
    // 3. 验证系统仍然可用
    let result = client.write("key2", "value2").await;
    assert!(result.is_ok());
    
    // 4. 恢复节点
    cluster.restart_node("node1").await;
    
    // 5. 验证数据一致性
    cluster.wait_for_consistency().await;
}
```

## 🧪 混沌测试

### 1. 故障注入测试

```rust
#[tokio::test]
async fn test_chaos_engineering() {
    let mut cluster = TestCluster::new(5).await;
    let chaos_engine = ChaosEngine::new(cluster.clone());
    
    let fault_scenarios = vec![
        FaultScenario::NodeCrash,
        FaultScenario::NetworkPartition,
        FaultScenario::MessageLoss,
    ];
    
    for scenario in fault_scenarios {
        // 注入故障
        chaos_engine.inject_fault(scenario).await;
        
        // 等待系统稳定
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // 验证系统仍然可用
        let result = cluster.write("test_key", "test_value").await;
        assert!(result.is_ok());
        
        // 恢复故障
        chaos_engine.recover_fault(scenario).await;
        
        // 验证系统恢复
        cluster.wait_for_consistency().await;
    }
}
```

### 2. 网络分区测试

```rust
#[tokio::test]
async fn test_network_partition() {
    let mut cluster = TestCluster::new(5).await;
    
    // 1. 创建网络分区
    cluster.partition(vec![0, 1], vec![2, 3, 4]).await;
    
    // 2. 验证多数派分区可以继续操作
    let majority_partition = cluster.get_partition(vec![2, 3, 4]);
    let result = majority_partition.write("key1", "value1").await;
    assert!(result.is_ok());
    
    // 3. 验证少数派分区不能操作
    let minority_partition = cluster.get_partition(vec![0, 1]);
    let result = minority_partition.write("key2", "value2").await;
    assert!(result.is_err());
    
    // 4. 恢复网络分区
    cluster.heal_partition().await;
    
    // 5. 验证数据一致性
    cluster.wait_for_consistency().await;
}
```

## 📊 性能测试

### 1. 延迟测试

```rust
#[tokio::test]
async fn test_latency_performance() {
    let cluster = TestCluster::new(5).await;
    let client = TestClient::new(cluster);
    
    let mut latencies = Vec::new();
    
    // 测量延迟
    for i in 0..1000 {
        let start = Instant::now();
        let result = client.write(&format!("key_{}", i), &format!("value_{}", i)).await;
        let latency = start.elapsed();
        
        assert!(result.is_ok());
        latencies.push(latency);
    }
    
    // 计算统计信息
    latencies.sort();
    let p50 = latencies[500];
    let p95 = latencies[950];
    let p99 = latencies[990];
    
    println!("P50 延迟: {:?}", p50);
    println!("P95 延迟: {:?}", p95);
    println!("P99 延迟: {:?}", p99);
    
    // 验证性能要求
    assert!(p50 < Duration::from_millis(10));
    assert!(p95 < Duration::from_millis(50));
    assert!(p99 < Duration::from_millis(100));
}
```

### 2. 吞吐量测试

```rust
#[tokio::test]
async fn test_throughput_performance() {
    let cluster = TestCluster::new(5).await;
    let client = TestClient::new(cluster);
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    // 启动多个并发客户端
    for i in 0..100 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let mut count = 0;
            for j in 0..100 {
                let result = client.write(&format!("key_{}_{}", i, j), &format!("value_{}_{}", i, j)).await;
                if result.is_ok() {
                    count += 1;
                }
            }
            count
        });
        handles.push(handle);
    }
    
    // 等待所有客户端完成
    let mut total_operations = 0;
    for handle in handles {
        total_operations += handle.await.unwrap();
    }
    
    let duration = start.elapsed();
    let throughput = total_operations as f64 / duration.as_secs_f64();
    
    println!("总操作数: {}", total_operations);
    println!("总时间: {:?}", duration);
    println!("吞吐量: {:.2} OPS", throughput);
    
    // 验证性能要求
    assert!(throughput > 1000.0);
}
```

## 🔍 一致性测试

### 1. 线性化测试

```rust
#[tokio::test]
async fn test_linearizability() {
    let cluster = TestCluster::new(5).await;
    let client = TestClient::new(cluster);
    
    let mut operations = Vec::new();
    
    // 生成并发操作
    for i in 0..100 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let result = client.write(&format!("key_{}", i), &format!("value_{}", i)).await;
            let end = Instant::now();
            
            Operation {
                id: i,
                start_time: start,
                end_time: end,
                operation_type: OperationType::Write,
                key: format!("key_{}", i),
                value: Some(format!("value_{}", i)),
                result: result.is_ok(),
            }
        });
        operations.push(handle);
    }
    
    // 等待所有操作完成
    let mut operation_history = Vec::new();
    for handle in operations {
        operation_history.push(handle.await.unwrap());
    }
    
    // 验证线性化
    let checker = LinearizabilityChecker::new();
    let is_linearizable = checker.verify(&operation_history);
    
    assert!(is_linearizable);
}
```

### 2. 最终一致性测试

```rust
#[tokio::test]
async fn test_eventual_consistency() {
    let cluster = TestCluster::new(5).await;
    let client = TestClient::new(cluster);
    
    // 写入数据
    let result = client.write("key1", "value1").await;
    assert!(result.is_ok());
    
    // 等待最终一致性
    let mut attempts = 0;
    let max_attempts = 100;
    
    while attempts < max_attempts {
        let mut consistent = true;
        
        for node in cluster.nodes() {
            let value = node.read("key1").await;
            if value != Some("value1".to_string()) {
                consistent = false;
                break;
            }
        }
        
        if consistent {
            break;
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }
    
    assert!(attempts < max_attempts, "最终一致性未达成");
}
```

## 🛠️ 测试工具

### 1. 测试集群

```rust
pub struct TestCluster {
    nodes: Vec<TestNode>,
    load_balancer: LoadBalancer,
    failure_detector: FailureDetector,
}

impl TestCluster {
    pub async fn new(node_count: usize) -> Self {
        let mut nodes = Vec::new();
        
        for i in 0..node_count {
            let node = TestNode::new(format!("node_{}", i)).await;
            nodes.push(node);
        }
        
        let load_balancer = LoadBalancer::new(nodes.clone());
        let failure_detector = FailureDetector::new(nodes.clone());
        
        Self {
            nodes,
            load_balancer,
            failure_detector,
        }
    }
    
    pub async fn write(&self, key: &str, value: &str) -> Result<(), Error> {
        self.load_balancer.select_server()
            .ok_or(Error::NoAvailableServers)?
            .write(key, value)
            .await
    }
    
    pub async fn kill_node(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.kill().await;
        }
    }
    
    pub async fn restart_node(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.restart().await;
        }
    }
    
    pub async fn wait_for_consistency(&self) {
        let mut consistent = false;
        
        while !consistent {
            consistent = true;
            
            for i in 0..self.nodes.len() {
                for j in i+1..self.nodes.len() {
                    if !self.nodes[i].is_consistent_with(&self.nodes[j]).await {
                        consistent = false;
                        break;
                    }
                }
                if !consistent {
                    break;
                }
            }
            
            if !consistent {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
```

### 2. 混沌引擎

```rust
pub struct ChaosEngine {
    cluster: TestCluster,
    fault_injectors: Vec<Box<dyn FaultInjector>>,
}

impl ChaosEngine {
    pub fn new(cluster: TestCluster) -> Self {
        let fault_injectors: Vec<Box<dyn FaultInjector>> = vec![
            Box::new(NodeCrashInjector::new()),
            Box::new(NetworkPartitionInjector::new()),
            Box::new(MessageLossInjector::new()),
        ];
        
        Self {
            cluster,
            fault_injectors,
        }
    }
    
    pub async fn inject_fault(&mut self, scenario: FaultScenario) {
        for injector in &mut self.fault_injectors {
            if injector.can_inject(scenario) {
                injector.inject(&mut self.cluster, scenario).await;
                break;
            }
        }
    }
    
    pub async fn recover_fault(&mut self, scenario: FaultScenario) {
        for injector in &mut self.fault_injectors {
            if injector.can_inject(scenario) {
                injector.recover(&mut self.cluster, scenario).await;
                break;
            }
        }
    }
}
```

## 📋 测试最佳实践

### 1. 测试组织

```rust
// 测试模块组织
#[cfg(test)]
mod tests {
    // 单元测试
    mod unit_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_basic_functionality() {
            // 测试基本功能
        }
    }
    
    // 集成测试
    mod integration_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_component_integration() {
            // 测试组件集成
        }
    }
    
    // 性能测试
    mod performance_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_performance() {
            // 测试性能
        }
    }
}
```

### 2. 测试数据管理

```rust
// 测试数据生成器
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: format!("user_{}", rand::random::<u32>()),
            email: format!("user_{}@example.com", rand::random::<u32>()),
            created_at: Utc::now(),
        }
    }
    
    pub fn generate_order(user_id: Uuid) -> Order {
        Order {
            id: Uuid::new_v4(),
            user_id,
            items: vec![
                OrderItem {
                    product_id: format!("product_{}", rand::random::<u32>()),
                    quantity: rand::random::<u32>() % 10 + 1,
                }
            ],
            total_amount: rand::random::<f64>() * 1000.0,
            status: OrderStatus::Pending,
        }
    }
}
```

### 3. 测试环境隔离

```rust
// 测试环境配置
pub struct TestEnvironment {
    pub database_url: String,
    pub redis_url: String,
    pub port: u16,
}

impl TestEnvironment {
    pub async fn setup() -> Self {
        // 设置测试环境
        let port = find_free_port().await;
        let database_url = format!("sqlite://test_{}.db", Uuid::new_v4());
        let redis_url = format!("redis://localhost:{}/0", port);
        
        Self {
            database_url,
            redis_url,
            port,
        }
    }
    
    pub async fn cleanup(&self) {
        // 清理测试环境
        tokio::fs::remove_file(&self.database_url).await.ok();
    }
}
```

## 🔗 相关资源

- [快速开始指南](../QUICKSTART.md)
- [系统设计最佳实践](../design/BEST_PRACTICES.md)
- [性能优化技巧](../performance/OPTIMIZATION.md)
- [实验检查清单](../experiments/CHECKLIST.md)
- [常见陷阱与调试](../PITFALLS.md)

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/c20_distributed/issues)
- **Discussions**: [讨论交流](https://github.com/your-org/c20_distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/c20-distributed)

---

**测试驱动开发！** 🚀 建立完善的测试体系，确保分布式系统的可靠性和稳定性。

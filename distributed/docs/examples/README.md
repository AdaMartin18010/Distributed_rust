# 示例代码中心

欢迎来到 `distributed` 的示例代码中心！这里提供了丰富的示例代码，帮助您快速理解和掌握分布式系统的核心概念和实现。

## 📚 示例分类

### 🎯 基础示例

- [Hello World](./basic/hello_world.rs) - 最简单的入门示例
- [基本复制](./basic/basic_replication.rs) - 本地复制演示
- [一致性级别](./basic/consistency_levels.rs) - 不同一致性级别对比

### 🔄 复制与一致性

- [强一致性复制](./replication/strong_consistency.rs) - 强一致性实现
- [最终一致性复制](./replication/eventual_consistency.rs) - 最终一致性实现
- [Quorum 读写](./replication/quorum_read_write.rs) - 法定人数读写
- [读修复机制](./replication/read_repair.rs) - 自动修复不一致

### 🗳️ 共识算法

- [Raft 基础](./consensus/raft_basic.rs) - Raft 算法基本用法
- [Raft 集群](./consensus/raft_cluster.rs) - 多节点 Raft 集群
- [领导者选举](./consensus/leader_election.rs) - 领导者选举过程
- [日志复制](./consensus/log_replication.rs) - 日志复制机制

### 💰 分布式事务

- [Saga 模式](./transactions/saga_pattern.rs) - Saga 分布式事务
- [2PC 协议](./transactions/two_phase_commit.rs) - 两阶段提交
- [TCC 模式](./transactions/try_confirm_cancel.rs) - TCC 分布式事务
- [补偿事务](./transactions/compensating_transaction.rs) - 补偿事务模式

### 🔍 故障检测

- [SWIM 协议](./membership/swim_protocol.rs) - SWIM 故障检测
- [心跳检测](./membership/heartbeat_detection.rs) - 心跳故障检测
- [故障恢复](./membership/failure_recovery.rs) - 故障自动恢复

### ⚖️ 负载均衡

- [轮询负载均衡](./load_balancing/round_robin.rs) - 轮询算法
- [一致性哈希](./load_balancing/consistent_hash.rs) - 一致性哈希
- [加权负载均衡](./load_balancing/weighted_balancing.rs) - 加权负载均衡

### 🛡️ 安全与限流

- [令牌桶限流](./security/token_bucket.rs) - 令牌桶算法
- [熔断器](./security/circuit_breaker.rs) - 熔断器模式
- [速率限制](./security/rate_limiting.rs) - 速率限制实现

### 🌐 网络通信

- [RPC 客户端](./network/rpc_client.rs) - RPC 客户端实现
- [RPC 服务端](./network/rpc_server.rs) - RPC 服务端实现
- [消息传递](./network/message_passing.rs) - 异步消息传递

### 📊 监控与可观测性

- [指标收集](./observability/metrics_collection.rs) - 指标收集和上报
- [链路追踪](./observability/distributed_tracing.rs) - 分布式链路追踪
- [日志聚合](./observability/log_aggregation.rs) - 日志聚合和分析

### 🧪 测试与验证

- [单元测试](./testing/unit_tests.rs) - 单元测试示例
- [集成测试](./testing/integration_tests.rs) - 集成测试示例
- [混沌测试](./testing/chaos_tests.rs) - 混沌工程测试

## 🚀 快速开始

### 运行第一个示例

```bash
# 克隆仓库
git clone https://github.com/your-org/distributed.git
cd distributed

# 运行 Hello World 示例
cargo run --example hello_world

# 运行基本复制示例
cargo run --example basic_replication

# 运行 Raft 集群示例
cargo run --example raft_cluster
```

### 运行所有示例

```bash
# 运行所有示例
cargo run --examples

# 运行特定分类的示例
cargo run --example replication_*
cargo run --example consensus_*
cargo run --example transactions_*
```

## 📖 示例详解

### 1. Hello World 示例

```rust
// examples/basic/hello_world.rs
use distributed::consistency::ConsistencyLevel;
use distributed::replication::LocalReplicator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 distributed Hello World!");
    
    // 创建本地复制器
    let replicator = LocalReplicator::new(3, 2, 2);
    
    // 写入数据
    replicator.replicate("greeting", "Hello, Distributed World!", ConsistencyLevel::Quorum).await?;
    
    // 读取数据
    let value = replicator.read("greeting", ConsistencyLevel::Quorum).await?;
    println!("读取结果: {:?}", value);
    
    Ok(())
}
```

### 2. Raft 集群示例

```rust
// examples/consensus/raft_cluster.rs
use distributed::consensus_raft::{RaftNode, RaftConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗳️ 启动 Raft 集群...");
    
    // 创建 3 个节点的集群
    let mut nodes = Vec::new();
    
    for i in 0..3 {
        let config = RaftConfig {
            node_id: format!("node_{}", i),
            peers: (0..3).filter(|&j| j != i).map(|j| format!("node_{}", j)).collect(),
            election_timeout_min: Duration::from_millis(150),
            election_timeout_max: Duration::from_millis(300),
            heartbeat_interval: Duration::from_millis(50),
        };
        
        let mut node = RaftNode::new(config).await?;
        node.start().await?;
        nodes.push(node);
    }
    
    // 等待集群稳定
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 提交日志条目
    let entry = "SET key1 value1".as_bytes().to_vec();
    nodes[0].propose(entry).await?;
    
    println!("✅ 日志条目已提交到集群");
    
    Ok(())
}
```

### 3. Saga 事务示例

```rust
// examples/transactions/saga_pattern.rs
use distributed::transactions::{Saga, SagaStep};

// 支付步骤
struct PaymentStep {
    user_id: String,
    amount: u64,
    executed: bool,
}

impl SagaStep for PaymentStep {
    fn execute(&mut self) -> Result<(), String> {
        if !self.executed {
            println!("💳 从用户 {} 扣除 {} 元", self.user_id, self.amount);
            self.executed = true;
        }
        Ok(())
    }
    
    fn compensate(&mut self) -> Result<(), String> {
        if self.executed {
            println!("🔄 向用户 {} 退还 {} 元", self.user_id, self.amount);
        }
        Ok(())
    }
}

// 库存步骤
struct InventoryStep {
    product_id: String,
    quantity: u32,
    executed: bool,
}

impl SagaStep for InventoryStep {
    fn execute(&mut self) -> Result<(), String> {
        if !self.executed {
            println!("📦 减少产品 {} 库存 {} 件", self.product_id, self.quantity);
            self.executed = true;
        }
        Ok(())
    }
    
    fn compensate(&mut self) -> Result<(), String> {
        if self.executed {
            println!("🔄 恢复产品 {} 库存 {} 件", self.product_id, self.quantity);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💰 开始 Saga 分布式事务...");
    
    let mut saga = Saga::new();
    
    // 添加事务步骤
    saga.add_step(Box::new(PaymentStep {
        user_id: "user123".to_string(),
        amount: 100,
        executed: false,
    }));
    
    saga.add_step(Box::new(InventoryStep {
        product_id: "product456".to_string(),
        quantity: 1,
        executed: false,
    }));
    
    // 执行事务
    match saga.execute().await {
        Ok(_) => println!("✅ 事务执行成功"),
        Err(e) => {
            println!("❌ 事务执行失败: {}", e);
            println!("🔄 开始补偿操作...");
            saga.compensate().await?;
        }
    }
    
    Ok(())
}
```

## 🔧 自定义示例

### 创建自定义示例

```bash
# 创建新的示例文件
touch examples/custom/my_example.rs
```

```rust
// examples/custom/my_example.rs
use distributed::{
    consistency::ConsistencyLevel,
    replication::LocalReplicator,
    // 添加其他需要的模块
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 我的自定义示例");
    
    // 您的代码逻辑
    
    Ok(())
}
```

### 运行自定义示例

```bash
# 运行自定义示例
cargo run --example my_example
```

## 📊 性能测试示例

### 基准测试

```rust
// examples/performance/benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};
use distributed::replication::LocalReplicator;
use distributed::consistency::ConsistencyLevel;

fn benchmark_replication(c: &mut Criterion) {
    let mut group = c.benchmark_group("replication");
    
    group.bench_function("quorum_write", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let replicator = LocalReplicator::new(5, 3, 3);
        
        b.iter(|| {
            rt.block_on(async {
                replicator.replicate("key", "value", ConsistencyLevel::Quorum).await
            })
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_replication);
criterion_main!(benches);
```

运行基准测试：

```bash
cargo bench
```

## 🧪 测试示例

### 单元测试

```rust
// examples/testing/unit_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use distributed::testing::TestCluster;
    
    #[tokio::test]
    async fn test_basic_replication() {
        let cluster = TestCluster::new(3).await;
        
        // 测试写入
        let result = cluster.write("key1", "value1").await;
        assert!(result.is_ok());
        
        // 测试读取
        let value = cluster.read("key1").await;
        assert_eq!(value, Some("value1".to_string()));
    }
    
    #[tokio::test]
    async fn test_consensus_consistency() {
        let cluster = TestCluster::new(5).await;
        
        // 提交多个操作
        for i in 0..10 {
            cluster.propose(format!("operation_{}", i)).await;
        }
        
        // 验证一致性
        assert!(cluster.is_consistent().await);
    }
}
```

### 集成测试

```rust
// examples/testing/integration_tests.rs
#[tokio::test]
async fn test_distributed_transaction() {
    let mut saga = Saga::new();
    
    // 添加会失败的步骤
    saga.add_step(Box::new(FailingStep::new()));
    
    // 执行事务
    let result = saga.execute().await;
    assert!(result.is_err());
    
    // 验证补偿执行
    assert!(saga.compensation_executed());
}
```

## 📚 学习路径

### 初学者路径

1. **Hello World** → 了解基本概念
2. **基本复制** → 理解复制机制
3. **一致性级别** → 掌握一致性模型
4. **Raft 基础** → 学习共识算法

### 进阶路径

1. **Saga 模式** → 掌握分布式事务
2. **SWIM 协议** → 理解故障检测
3. **负载均衡** → 学习流量分发
4. **监控可观测性** → 掌握运维技能

### 专家路径

1. **混沌测试** → 验证系统健壮性
2. **性能优化** → 提升系统性能
3. **自定义实现** → 扩展系统功能
4. **生产部署** → 实际应用经验

## 🔗 相关资源

- [快速开始指南](../QUICKSTART.md)
- [安装指南](../INSTALL.md)
- [一致性模型详解](../consistency/README.md)
- [共识算法实现](../consensus/README.md)
- [测试策略](../testing/README.md)

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/distributed/issues)
- **Discussions**: [讨论交流](https://github.com/your-org/distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/c20-distributed)

---

**开始探索示例代码！** 🚀 选择适合您水平的示例，逐步掌握分布式系统的核心概念和实现技巧。

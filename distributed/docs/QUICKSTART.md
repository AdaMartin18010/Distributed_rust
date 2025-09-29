# 快速开始指南

本指南将帮助您在5分钟内快速上手 `distributed` 分布式系统库。

## 🚀 安装

### 添加依赖

在您的 `Cargo.toml` 中添加：

```toml
[dependencies]
distributed = "0.5.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### 安装依赖

```bash
cargo build
```

## 📦 核心模块概览

```rust
use distributed::{
    // 一致性模型
    consistency::{ConsistencyLevel, LinearizabilityChecker},
    
    // 复制策略
    replication::{LocalReplicator, MajorityQuorum},
    
    // 共识算法
    consensus_raft::{RaftNode, RaftConfig},
    
    // 分布式事务
    transactions::{Saga, SagaStep},
    
    // 故障检测
    membership::{SwimNode, SwimConfig},
    
    // 负载均衡
    load_balancing::{RoundRobinBalancer, ConsistentHashRing},
    
    // 限流和熔断
    security::{TokenBucket, CircuitBreaker},
    
    // 网络通信
    network::{RpcClient, RpcServer},
    
    // 可观测性
    observability::{Metrics, Tracer},
};
```

## 🎯 第一个示例：分布式键值存储

### 1. 创建本地复制器

```rust
use distributed::replication::{LocalReplicator, MajorityQuorum};
use distributed::consistency::ConsistencyLevel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建5个节点的复制器，使用多数派一致性
    let replicator = LocalReplicator::new(5, 3, 3); // N=5, R=3, W=3
    
    // 写入数据
    replicator.replicate("user:123", "Alice", ConsistencyLevel::Quorum).await?;
    
    // 读取数据
    let value = replicator.read("user:123", ConsistencyLevel::Quorum).await?;
    println!("读取结果: {:?}", value);
    
    Ok(())
}
```

### 2. 运行Raft共识

```rust
use distributed::consensus_raft::{RaftNode, RaftConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置Raft节点
    let config = RaftConfig {
        node_id: "node1".to_string(),
        peers: vec!["node2".to_string(), "node3".to_string()],
        election_timeout_min: Duration::from_millis(150),
        election_timeout_max: Duration::from_millis(300),
        heartbeat_interval: Duration::from_millis(50),
    };
    
    // 创建Raft节点
    let mut raft = RaftNode::new(config).await?;
    
    // 启动节点
    raft.start().await?;
    
    // 提交日志条目
    let entry = "SET key1 value1".as_bytes().to_vec();
    raft.propose(entry).await?;
    
    Ok(())
}
```

### 3. 实现分布式事务（Saga模式）

```rust
use distributed::transactions::{Saga, SagaStep, SagaResult};

// 定义支付步骤
struct PaymentStep {
    user_id: String,
    amount: u64,
    executed: bool,
}

impl SagaStep for PaymentStep {
    fn execute(&mut self) -> Result<(), String> {
        if !self.executed {
            println!("从用户 {} 扣除 {} 元", self.user_id, self.amount);
            self.executed = true;
        }
        Ok(())
    }
    
    fn compensate(&mut self) -> Result<(), String> {
        if self.executed {
            println!("向用户 {} 退还 {} 元", self.user_id, self.amount);
        }
        Ok(())
    }
}

// 定义库存步骤
struct InventoryStep {
    product_id: String,
    quantity: u32,
    executed: bool,
}

impl SagaStep for InventoryStep {
    fn execute(&mut self) -> Result<(), String> {
        if !self.executed {
            println!("减少产品 {} 库存 {} 件", self.product_id, self.quantity);
            self.executed = true;
        }
        Ok(())
    }
    
    fn compensate(&mut self) -> Result<(), String> {
        if self.executed {
            println!("恢复产品 {} 库存 {} 件", self.product_id, self.quantity);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        Ok(_) => println!("事务执行成功"),
        Err(e) => {
            println!("事务执行失败: {}", e);
            println!("开始补偿操作...");
            saga.compensate().await?;
        }
    }
    
    Ok(())
}
```

## 🔧 配置和调优

### 基本配置

```rust
use distributed::config::DistributedConfig;

let config = DistributedConfig {
    // 一致性配置
    consistency_level: ConsistencyLevel::Quorum,
    quorum_reads: true,
    quorum_writes: true,
    
    // 超时配置
    read_timeout: Duration::from_millis(100),
    write_timeout: Duration::from_millis(200),
    
    // 重试配置
    max_retries: 3,
    retry_backoff: Duration::from_millis(100),
    
    // 监控配置
    enable_metrics: true,
    enable_tracing: true,
    
    // 日志配置
    log_level: "info".to_string(),
};
```

### 性能调优

```rust
use distributed::performance::PerformanceConfig;

let perf_config = PerformanceConfig {
    // 连接池配置
    connection_pool_size: 100,
    connection_timeout: Duration::from_secs(5),
    
    // 批处理配置
    batch_size: 1000,
    batch_timeout: Duration::from_millis(10),
    
    // 缓存配置
    cache_size: 10000,
    cache_ttl: Duration::from_secs(300),
    
    // 并发配置
    max_concurrent_requests: 1000,
    request_queue_size: 10000,
};
```

## 📊 监控和可观测性

### 基本指标

```rust
use distributed::observability::{Metrics, Counter, Histogram};

// 创建指标收集器
let metrics = Metrics::new();

// 定义计数器
let request_counter = Counter::new("requests_total");
let error_counter = Counter::new("errors_total");

// 定义直方图
let latency_histogram = Histogram::new("request_duration_seconds");

// 记录指标
request_counter.inc();
error_counter.inc_by(5);
latency_histogram.observe(0.1);
```

### 链路追踪

```rust
use distributed::observability::Tracer;

// 创建追踪器
let tracer = Tracer::new("my-service");

// 创建span
let span = tracer.start_span("process_request");
span.set_attribute("user_id", "123");
span.set_attribute("operation", "read");

// 执行操作
let result = process_request().await;

// 记录结果
span.set_attribute("success", result.is_ok());
span.finish();
```

## 🧪 测试和验证

### 单元测试

```rust
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

## 🚨 常见问题和解决方案

### 问题1：连接超时

```rust
// 解决方案：调整超时配置
let config = DistributedConfig {
    read_timeout: Duration::from_millis(500), // 增加超时时间
    write_timeout: Duration::from_millis(1000),
    max_retries: 5, // 增加重试次数
    ..Default::default()
};
```

### 问题2：内存使用过高

```rust
// 解决方案：优化配置
let perf_config = PerformanceConfig {
    cache_size: 1000, // 减少缓存大小
    batch_size: 100, // 减少批处理大小
    connection_pool_size: 50, // 减少连接池大小
    ..Default::default()
};
```

### 问题3：性能瓶颈

```rust
// 解决方案：启用性能监控
let config = DistributedConfig {
    enable_metrics: true,
    enable_tracing: true,
    log_level: "debug".to_string(),
    ..Default::default()
};

// 分析性能指标
let metrics = Metrics::new();
let latency_p99 = metrics.get_histogram("request_duration_seconds").p99();
println!("P99延迟: {}ms", latency_p99 * 1000.0);
```

## 📚 下一步

现在您已经掌握了基本用法，可以继续深入学习：

1. **深入理解理论** → [一致性模型详解](./consistency/README.md)
2. **学习最佳实践** → [系统设计最佳实践](./design/BEST_PRACTICES.md)
3. **掌握测试技巧** → [测试策略](./testing/README.md)
4. **了解性能优化** → [性能优化技巧](./performance/OPTIMIZATION.md)

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/distributed/issues)
- **Discussions**: [讨论交流](https://github.com/your-org/distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/c20-distributed)

---

**恭喜！** 🎉 您已经成功入门 `distributed` 分布式系统库。继续探索更多高级功能，构建您的分布式应用吧！

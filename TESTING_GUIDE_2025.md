# Distributed Rust 测试指南 (2025)

## 🧪 测试策略概览

本指南提供了 Distributed Rust 项目的全面测试策略，包括单元测试、集成测试、端到端测试、性能测试和混沌测试的详细说明。

## 📊 测试金字塔

```text
        🔺 E2E Tests (5%)
       🔺🔺🔺 Integration Tests (25%)
    🔺🔺🔺🔺🔺 Unit Tests (70%)
```

### 测试分布

- **单元测试 (70%)**: 测试单个函数和模块的功能
- **集成测试 (25%)**: 测试模块间的交互和接口
- **端到端测试 (5%)**: 测试完整的用户场景

## 🔧 测试环境设置

### 开发环境

```bash
# 安装测试工具
cargo install cargo-tarpaulin  # 代码覆盖率
cargo install cargo-nextest    # 并行测试运行器
cargo install cargo-fuzz       # 模糊测试
cargo install cargo-audit      # 安全审计

# 运行测试
cargo test                    # 运行所有测试
cargo nextest run            # 使用并行运行器
cargo test --release         # 发布模式测试
cargo test --features chaos  # 启用混沌测试
```

### CI/CD 环境

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
        features: [default, chaos, observability]
    
    steps:
    - uses: actions/checkout@v3
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
    
    - name: Run tests
      run: |
        cargo test --features ${{ matrix.features }}
        cargo test --release --features ${{ matrix.features }}
    
    - name: Run benchmarks
      run: cargo bench --features ${{ matrix.features }}
    
    - name: Generate coverage
      run: cargo tarpaulin --features ${{ matrix.features }}
```

## 🧪 单元测试

### 测试结构

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Arrange - 准备测试数据
        let input = create_test_data();
        let expected = expected_result();
        
        // Act - 执行被测试的功能
        let result = function_under_test(input);
        
        // Assert - 验证结果
        assert_eq!(result, expected);
    }
    
    #[test]
    #[should_panic(expected = "错误消息")]
    fn test_error_case() {
        // 测试错误情况
        function_that_should_panic();
    }
}
```

### 测试工具

```rust
use distributed::test_utils::*;

#[test]
fn test_with_mocks() {
    // 使用 Mock 对象
    let mut mock_service = MockService::new();
    mock_service.expect_call().times(1).returning(|_| Ok("success"));
    
    let result = function_using_service(&mock_service);
    assert!(result.is_ok());
}

#[test]
fn test_with_fixtures() {
    // 使用测试夹具
    let fixture = TestFixture::new()
        .with_nodes(3)
        .with_data_size(1000)
        .build();
    
    let result = test_function(&fixture);
    assert!(result.is_ok());
}
```

## 🔗 集成测试

### 测试模块间交互

```rust
// tests/integration_tests.rs
use distributed::{
    ConsistentHashRing, LocalReplicator, ConsistencyLevel,
    LoadBalancingStrategy, RoundRobinBalancer,
};

#[test]
fn test_hash_ring_with_replication() {
    // 创建一致性哈希环
    let mut ring = ConsistentHashRing::new(16);
    let nodes = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
    
    for node in &nodes {
        ring.add_node(node);
    }
    
    // 创建复制器
    let replicator = LocalReplicator::new(ring, nodes);
    
    // 测试复制功能
    let result = replicator.replicate(100u64, ConsistencyLevel::Quorum);
    assert!(result.is_ok());
}

#[test]
fn test_load_balancer_with_services() {
    let services = create_test_services(3);
    let mut balancer = RoundRobinBalancer::new();
    
    // 测试负载均衡
    let mut distribution = std::collections::HashMap::new();
    for _ in 0..30 {
        let service = balancer.select_service(&services);
        *distribution.entry(service.id.clone()).or_insert(0) += 1;
    }
    
    // 验证分布均匀性
    for (_, count) in &distribution {
        assert!(*count >= 8 && *count <= 12);
    }
}
```

### 数据库集成测试

```rust
#[test]
fn test_database_integration() {
    // 使用测试数据库
    let db = TestDatabase::new();
    
    // 执行数据库操作
    let result = db.insert_data("key", "value");
    assert!(result.is_ok());
    
    let retrieved = db.get_data("key");
    assert_eq!(retrieved, Some("value".to_string()));
}
```

## 🚀 端到端测试

### 完整场景测试

```rust
// tests/e2e_tests.rs
use distributed::*;

#[tokio::test]
async fn test_complete_user_journey() {
    // 1. 启动分布式系统
    let system = DistributedSystem::new()
        .with_nodes(3)
        .with_replication_factor(2)
        .start()
        .await
        .unwrap();
    
    // 2. 执行用户操作
    let user_id = "user-123";
    let result = system.create_user(user_id, "John Doe").await;
    assert!(result.is_ok());
    
    // 3. 验证数据一致性
    let user = system.get_user(user_id).await.unwrap();
    assert_eq!(user.name, "John Doe");
    
    // 4. 测试故障恢复
    system.simulate_node_failure(1).await;
    
    // 5. 验证系统继续工作
    let user_after_failure = system.get_user(user_id).await.unwrap();
    assert_eq!(user_after_failure.name, "John Doe");
    
    // 6. 清理
    system.shutdown().await;
}
```

### 性能端到端测试

```rust
#[tokio::test]
async fn test_performance_under_load() {
    let system = DistributedSystem::new().start().await.unwrap();
    
    // 并发请求测试
    let handles: Vec<_> = (0..100).map(|i| {
        let system = system.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = system.process_request(format!("req-{}", i)).await;
            let duration = start.elapsed();
            (result, duration)
        })
    }).collect();
    
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // 验证性能指标
    let successful_requests: usize = results.iter()
        .filter(|(result, _)| result.is_ok())
        .count();
    
    let avg_duration: u64 = results.iter()
        .map(|(_, duration)| duration.as_millis() as u64)
        .sum::<u64>() / results.len() as u64;
    
    assert!(successful_requests > 95, "成功率应该超过 95%");
    assert!(avg_duration < 100, "平均延迟应该小于 100ms");
}
```

## 💥 混沌测试

### 故障注入测试

```rust
#[cfg(feature = "chaos")]
mod chaos_tests {
    use super::*;
    use distributed::chaos::{ChaosInjector, ChaosConfig};
    
    #[test]
    fn test_network_partition() {
        let system = DistributedSystem::new().start();
        let mut chaos = ChaosInjector::new();
        
        // 注入网络分区
        chaos.inject_partition(vec![0], vec![1, 2]);
        
        // 验证系统行为
        let result = system.process_request("test");
        assert!(result.is_err() || result.is_ok()); // 可能失败或成功
        
        // 恢复网络
        chaos.remove_partition();
        
        // 验证系统恢复
        let result = system.process_request("test");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_node_failure() {
        let system = DistributedSystem::new().with_nodes(5).start();
        
        // 模拟节点失败
        system.kill_node(1);
        
        // 验证系统继续工作
        let result = system.process_request("test");
        assert!(result.is_ok());
        
        // 验证数据一致性
        let data = system.get_data("key");
        assert!(data.is_some());
    }
    
    #[test]
    fn test_high_latency() {
        let system = DistributedSystem::new().start();
        let mut chaos = ChaosInjector::new();
        
        // 注入高延迟
        chaos.inject_latency(Duration::from_millis(1000));
        
        let start = std::time::Instant::now();
        let _result = system.process_request("test");
        let duration = start.elapsed();
        
        assert!(duration >= Duration::from_millis(1000));
    }
}
```

### 随机故障测试

```rust
#[test]
fn test_random_failures() {
    let system = DistributedSystem::new().start();
    let mut chaos = ChaosInjector::new();
    
    // 随机注入故障
    for _ in 0..100 {
        let failure_type = rand::random::<FailureType>();
        match failure_type {
            FailureType::NetworkPartition => {
                let nodes = (0..3).collect::<Vec<_>>();
                chaos.inject_partition(vec![nodes[0]], vec![nodes[1], nodes[2]]);
            }
            FailureType::NodeFailure => {
                let node_id = rand::random::<usize>() % 3;
                system.kill_node(node_id);
            }
            FailureType::HighLatency => {
                let delay = Duration::from_millis(rand::random::<u64>() % 500);
                chaos.inject_latency(delay);
            }
        }
        
        // 验证系统仍然工作
        let result = system.process_request("test");
        assert!(result.is_ok() || result.is_err()); // 允许失败
        
        // 清理故障
        chaos.clear_all();
        system.restart_all_nodes();
    }
}
```

## 📊 性能测试

### 基准测试

```rust
// benches/performance_tests.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_hash_ring_lookup(c: &mut Criterion) {
    let mut ring = ConsistentHashRing::new(64);
    for i in 0..10 {
        ring.add_node(&format!("node-{}", i));
    }
    
    c.bench_function("hash_ring_lookup", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(ring.get_node(&format!("key-{}", i)));
            }
        });
    });
}

fn benchmark_replication(c: &mut Criterion) {
    let mut ring = ConsistentHashRing::new(16);
    let nodes = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
    for node in &nodes {
        ring.add_node(node);
    }
    
    let replicator = LocalReplicator::new(ring, nodes);
    
    c.bench_function("replication_quorum", |b| {
        b.iter(|| {
            for i in 0..100 {
                black_box(replicator.replicate(i as u64, ConsistencyLevel::Quorum));
            }
        });
    });
}

criterion_group!(benches, benchmark_hash_ring_lookup, benchmark_replication);
criterion_main!(benches);
```

### 负载测试

```rust
#[tokio::test]
async fn test_load_capacity() {
    let system = DistributedSystem::new().start().await.unwrap();
    
    // 逐步增加负载
    for load in [10, 50, 100, 200, 500] {
        let start = std::time::Instant::now();
        
        let handles: Vec<_> = (0..load).map(|i| {
            let system = system.clone();
            tokio::spawn(async move {
                system.process_request(format!("load-{}", i)).await
            })
        }).collect();
        
        let results = futures::future::join_all(handles).await;
        let duration = start.elapsed();
        
        let success_rate = results.iter()
            .filter(|r| r.is_ok())
            .count() as f64 / results.len() as f64;
        
        println!("负载 {}: 成功率 {:.2}%, 耗时 {:?}", load, success_rate * 100.0, duration);
        
        // 验证性能指标
        assert!(success_rate > 0.95, "成功率应该超过 95%");
        assert!(duration.as_millis() < 1000, "处理时间应该小于 1 秒");
    }
}
```

## 🔍 测试覆盖率

### 覆盖率目标

- **单元测试覆盖率**: ≥ 90%
- **集成测试覆盖率**: ≥ 80%
- **端到端测试覆盖率**: ≥ 70%
- **整体覆盖率**: ≥ 85%

### 覆盖率报告

```bash
# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage

# 查看覆盖率报告
open coverage/tarpaulin-report.html
```

### 覆盖率配置

```toml
# Cargo.toml
[package.metadata.tarpaulin]
# 忽略测试文件和示例
exclude-files = ["tests/", "examples/", "benches/"]
# 忽略特定行
exclude-lines = ["panic!", "unreachable!", "todo!"]
# 设置覆盖率阈值
fail_under = 85
```

## 🚨 测试最佳实践

### 1. 测试命名

```rust
// 好的测试命名
#[test]
fn test_hash_ring_distributes_keys_evenly() { }

#[test]
fn test_replication_fails_with_insufficient_nodes() { }

// 不好的测试命名
#[test]
fn test1() { }

#[test]
fn test_thing() { }
```

### 2. 测试结构

```rust
#[test]
fn test_function_behavior() {
    // Arrange - 准备测试数据
    let input = create_test_input();
    let expected = expected_output();
    
    // Act - 执行被测试的功能
    let result = function_under_test(input);
    
    // Assert - 验证结果
    assert_eq!(result, expected);
    assert!(result.is_valid());
    assert!(result.performance().is_acceptable());
}
```

### 3. 测试数据管理

```rust
// 使用测试夹具
struct TestFixture {
    nodes: Vec<String>,
    data: HashMap<String, String>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            nodes: vec!["node1".to_string(), "node2".to_string()],
            data: HashMap::new(),
        }
    }
    
    fn with_nodes(mut self, count: usize) -> Self {
        self.nodes = (0..count).map(|i| format!("node{}", i)).collect();
        self
    }
    
    fn with_data(mut self, data: HashMap<String, String>) -> Self {
        self.data = data;
        self
    }
}

#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new()
        .with_nodes(3)
        .with_data(create_test_data());
    
    let result = test_function(&fixture);
    assert!(result.is_ok());
}
```

### 4. 异步测试

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let handles: Vec<_> = (0..10).map(|i| {
        tokio::spawn(async move {
            async_function_with_id(i).await
        })
    }).collect();
    
    let results = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.is_ok());
    }
}
```

### 5. 测试隔离

```rust
// 每个测试使用独立的数据
#[test]
fn test_isolated_1() {
    let system = DistributedSystem::new_with_id("test-1");
    // 测试逻辑...
}

#[test]
fn test_isolated_2() {
    let system = DistributedSystem::new_with_id("test-2");
    // 测试逻辑...
}
```

## 📋 测试检查清单

### 代码提交前检查

- [ ] 所有单元测试通过
- [ ] 集成测试通过
- [ ] 端到端测试通过
- [ ] 性能测试无回归
- [ ] 代码覆盖率达标
- [ ] 混沌测试通过
- [ ] 安全测试通过

### 发布前检查

- [ ] 完整测试套件通过
- [ ] 性能基准达标
- [ ] 兼容性测试通过
- [ ] 文档测试通过
- [ ] 安装测试通过

## 🔧 测试工具链

### 推荐工具

| 工具 | 用途 | 安装命令 |
|------|------|----------|
| `cargo test` | 基础测试运行器 | 内置 |
| `cargo nextest` | 并行测试运行器 | `cargo install cargo-nextest` |
| `cargo tarpaulin` | 代码覆盖率 | `cargo install cargo-tarpaulin` |
| `cargo fuzz` | 模糊测试 | `cargo install cargo-fuzz` |
| `cargo audit` | 安全审计 | `cargo install cargo-audit` |
| `cargo deny` | 许可证检查 | `cargo install cargo-deny` |

### IDE 集成

```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.runnables.command": "cargo",
    "rust-analyzer.runnables.args": ["test", "--package", "${crate}", "--bin", "${crate}", "--", "--exact", "${runnable}"]
}
```

---

**测试是软件质量的重要保障，持续改进测试策略和覆盖率是项目成功的关键。** 🧪

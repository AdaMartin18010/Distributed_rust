# Distributed Rust 开发指南 (2025)

## 🛠️ 开发环境设置

### 系统要求

- **操作系统**: Linux (Ubuntu 20.04+), macOS (10.15+), Windows 10/11
- **Rust 版本**: 1.90+ (最新稳定版)
- **内存**: 至少 8GB RAM (推荐 16GB)
- **存储**: 至少 10GB 可用空间
- **网络**: 稳定的互联网连接 (用于依赖下载)

### 工具链安装

```bash
# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version  # 应该显示 1.90+
cargo --version

# 安装开发工具
rustup component add clippy rustfmt
cargo install cargo-audit cargo-deny cargo-outdated
```

### IDE 配置

推荐使用 **VS Code** 或 **CLion** 配合 Rust 插件：

```json
// .vscode/settings.json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true,
    "rust-analyzer.imports.granularity.group": "module"
}
```

## 📁 项目结构详解

### 核心目录

```
distributed/
├── src/                    # 源代码目录
│   ├── lib.rs             # 库入口点
│   ├── core/              # 核心模块
│   │   ├── mod.rs         # 模块声明
│   │   ├── config.rs      # 配置管理
│   │   ├── errors.rs      # 错误定义
│   │   ├── membership.rs  # 成员管理
│   │   ├── topology.rs    # 拓扑管理
│   │   └── scheduling.rs  # 调度器
│   ├── consensus/         # 共识算法
│   │   ├── mod.rs
│   │   ├── raft.rs        # Raft 实现
│   │   ├── paxos.rs       # Paxos 实现
│   │   └── byzantine.rs   # 拜占庭容错
│   ├── consistency/       # 一致性模型
│   ├── network/           # 网络通信
│   ├── storage/           # 存储抽象
│   ├── monitoring/        # 监控系统
│   ├── security/          # 安全模块
│   ├── examples/          # 库内示例
│   └── benchmarks/        # 性能基准
├── docs/                  # 文档目录
├── tests/                 # 集成测试
├── examples/              # 可执行示例
├── benches/               # 基准测试
└── Cargo.toml            # 项目配置
```

### 解决方案目录

```
solutions/
├── foundations-datafusion/  # DataFusion 集成
├── vector-topology/        # Vector 配置
├── end-to-end-stack/       # 端到端示例
├── deployment-strategies/  # 部署配置
└── multi-language-clients/ # 多语言客户端
```

## 🔧 开发工作流

### 1. 创建新功能

```bash
# 创建功能分支
git checkout -b feature/your-feature-name

# 开发功能
# ... 编写代码 ...

# 运行测试
cargo test --workspace

# 运行基准测试
cargo bench --workspace

# 检查代码质量
cargo clippy --workspace -- -D warnings
cargo fmt --all

# 提交代码
git add .
git commit -m "feat: add your feature description"
git push origin feature/your-feature-name
```

### 2. 代码规范

#### Rust 编码风格

```rust
// 使用 snake_case 命名函数和变量
fn calculate_hash_ring_size(node_count: usize) -> usize {
    // 使用有意义的变量名
    let optimal_size = node_count * 2;
    optimal_size
}

// 使用 PascalCase 命名类型
pub struct ConsistentHashRing {
    nodes: Vec<Node>,
    ring_size: usize,
}

// 使用 SCREAMING_SNAKE_CASE 命名常量
const DEFAULT_ELECTION_TIMEOUT_MS: u64 = 150;
const MAX_LOG_ENTRIES_PER_BATCH: usize = 1000;

// 使用有意义的错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Invalid term: {term}")]
    InvalidTerm { term: u64 },
    
    #[error("Network timeout after {duration_ms}ms")]
    NetworkTimeout { duration_ms: u64 },
}
```

#### 文档注释

```rust
/// 计算一致性哈希环中键的分布情况
/// 
/// # 参数
/// 
/// * `key` - 要计算哈希的键
/// * `ring_size` - 哈希环的大小
/// 
/// # 返回值
/// 
/// 返回键在环上的位置 (0 到 ring_size-1)
/// 
/// # 示例
/// 
/// ```
/// use distributed::topology::ConsistentHashRing;
/// 
/// let ring = ConsistentHashRing::new(16);
/// let position = ring.get_position("user:123");
/// assert!(position < 16);
/// ```
pub fn get_position(&self, key: &str) -> usize {
    // 实现细节...
}
```

### 3. 测试策略

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_ring_consistency() {
        let mut ring = ConsistentHashRing::new(8);
        ring.add_node("node1");
        ring.add_node("node2");
        
        // 测试键的分布一致性
        let key = "test_key";
        let node1 = ring.get_node(key);
        
        // 多次调用应该返回相同结果
        for _ in 0..10 {
            assert_eq!(ring.get_node(key), node1);
        }
    }
    
    #[test]
    #[should_panic(expected = "Empty ring")]
    fn test_empty_ring_panic() {
        let ring = ConsistentHashRing::new(0);
        ring.get_node("key"); // 应该 panic
    }
}
```

#### 集成测试

```rust
// tests/integration_tests.rs
use distributed::consensus::raft::RaftNode;
use distributed::network::InMemoryRpcServer;

#[tokio::test]
async fn test_raft_election() {
    // 创建测试集群
    let mut cluster = create_test_cluster(3).await;
    
    // 模拟网络分区
    cluster.partition(vec![0], vec![1, 2]).await;
    
    // 验证选举结果
    let leader = cluster.wait_for_leader().await;
    assert!(leader.is_some());
}
```

#### 属性测试

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_replication_quorum_property(
        node_count in 3..10usize,
        required_acks in 1..10usize
    ) {
        // 验证 Quorum 属性: required_acks >= floor(N/2) + 1
        let minimum_quorum = (node_count / 2) + 1;
        prop_assume!(required_acks >= minimum_quorum);
        prop_assume!(required_acks <= node_count);
        
        // 验证复制成功条件
        let replication_success = required_acks >= minimum_quorum;
        prop_assert!(replication_success);
    }
}
```

### 4. 性能优化

#### 基准测试

```rust
// benches/raft_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use distributed::consensus::raft::RaftNode;

fn benchmark_raft_append_entries(c: &mut Criterion) {
    let mut group = c.benchmark_group("raft_append_entries");
    
    // 基准测试不同批次大小
    for batch_size in [1, 10, 100, 1000] {
        group.bench_function(&format!("batch_{}", batch_size), |b| {
            b.iter(|| {
                let mut raft = create_test_raft();
                let entries = create_test_entries(batch_size);
                
                black_box(raft.append_entries(entries).unwrap());
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_raft_append_entries);
criterion_main!(benches);
```

#### 性能分析

```bash
# 使用 cargo-profdata 分析性能
cargo install cargo-profdata

# 生成性能报告
cargo profdata build --release
cargo profdata show --summary

# 使用 flamegraph 可视化
cargo install flamegraph
cargo flamegraph --bench raft_performance
```

## 🧪 测试策略

### 测试分类

1. **单元测试**: 测试单个函数或模块
2. **集成测试**: 测试模块间的交互
3. **端到端测试**: 测试完整的用户场景
4. **性能测试**: 基准测试和性能回归测试
5. **混沌测试**: 故障注入和恢复测试

### 测试命令

```bash
# 运行所有测试
cargo test --workspace

# 运行特定模块测试
cargo test -p distributed --lib consensus

# 运行集成测试
cargo test --test integration_tests

# 运行基准测试
cargo bench --workspace

# 运行属性测试 (需要更多时间)
cargo test --features proptest

# 运行混沌测试
cargo test --features chaos --test chaos_tests
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --workspace --out Html

# 查看覆盖率报告
open tarpaulin-report.html
```

## 🔍 调试技巧

### 日志配置

```rust
// 在测试中启用详细日志
#[tokio::test]
async fn test_with_logging() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .try_init();
    
    // 测试代码...
}
```

### 调试工具

```bash
# 使用 rust-gdb 调试
cargo install cargo-gdb
cargo gdb --bin your_binary

# 使用 rr 进行确定性调试
cargo install cargo-rr
cargo rr test --workspace

# 内存分析
cargo install cargo-valgrind
cargo valgrind test --workspace
```

## 📦 发布流程

### 版本管理

```bash
# 更新版本号
cargo set-version 0.2.0

# 生成 CHANGELOG
cargo install cargo-changelog
cargo changelog

# 创建发布标签
git tag v0.2.0
git push origin v0.2.0
```

### 发布检查清单

- [ ] 所有测试通过
- [ ] 基准测试无性能回归
- [ ] 文档更新完整
- [ ] CHANGELOG 更新
- [ ] 版本号正确
- [ ] 依赖安全审计通过
- [ ] 许可证检查通过

## 🚀 部署指南

### Docker 部署

```dockerfile
# Dockerfile
FROM rust:1.90-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/distributed /usr/local/bin/
EXPOSE 8080
CMD ["distributed"]
```

### Kubernetes 部署

```yaml
# k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: distributed-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: distributed-service
  template:
    metadata:
      labels:
        app: distributed-service
    spec:
      containers:
      - name: distributed
        image: distributed:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

## 📚 学习资源

### 推荐阅读

1. **分布式系统理论**
   - 《分布式系统概念与设计》
   - 《数据密集型应用系统设计》
   - MIT 6.824 课程材料

2. **Rust 编程**
   - 《Rust 程序设计语言》
   - 《Rust 异步编程》
   - Rust 官方文档

3. **相关论文**
   - Raft: In Search of an Understandable Consensus Algorithm
   - The Part-Time Parliament (Paxos)
   - Dynamo: Amazon's Highly Available Key-value Store

### 在线资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Tokio 教程](https://tokio.rs/tokio/tutorial)
- [分布式系统课程](https://pdos.csail.mit.edu/6.824/)

---

**Happy Coding!** 🦀

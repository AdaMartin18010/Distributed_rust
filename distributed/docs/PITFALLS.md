# 常见陷阱与调试建议

本文档总结了分布式系统开发中的常见陷阱，并提供相应的调试建议和最佳实践。

## 🔢 数学与算法陷阱

### 多数派边界错误

**陷阱**: `w == n/2` 不是多数派；必须 `> n/2`。

```rust
// ❌ 错误：这不是真正的多数派
let required_acks = total_nodes / 2;

// ✅ 正确：真正的多数派
let required_acks = (total_nodes / 2) + 1;
// 或者使用更清晰的表达
let required_acks = total_nodes - (total_nodes / 2);
```

**调试建议**:

- 使用单元测试验证多数派计算
- 在日志中记录 `total_nodes`、`required_acks` 和实际收到的 `acks`
- 对于奇数/偶数节点数分别测试

### 一致性哈希倾斜问题

**陷阱**: 虚拟节点数过小导致热点；增加 `replicas` 并监控迁移比例。

```rust
// ❌ 错误：虚拟节点数太少
let mut ring = ConsistentHashRing::new(8);  // 太少

// ✅ 正确：足够的虚拟节点数
let mut ring = ConsistentHashRing::new(64); // 推荐 50-200
```

**监控指标**:

- 键迁移比例：扩容时应该接近 `1/N`
- 倾斜度：P95/P99 分布应该相对均匀
- 热点检测：单个节点处理的请求数不应超过平均值的 2 倍

## 🔄 一致性模型陷阱

### Eventual 一致性误解

**陷阱**: 写 1 副本成功即返回，但读到旧值是预期现象。

```rust
// ❌ 错误理解：认为 Eventual 是"立即一致"
let result = replicator.replicate(data, ConsistencyLevel::Eventual);
// 立即读取可能得到旧值，这是正常的！

// ✅ 正确理解：Eventual 允许短暂不一致
let result = replicator.replicate(data, ConsistencyLevel::Eventual);
// 需要等待反熵同步完成才能保证一致性
```

**调试建议**:

- 明确区分一致性级别的语义
- 在文档中清楚说明每个级别的保证
- 提供一致性检查工具

### 线性化读实现错误

**陷阱**: 没有正确实现 read_index 或 lease read。

```rust
// ❌ 错误：直接读取，可能读到旧值
fn read_key(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
    self.storage.get(key)  // 不安全！
}

// ✅ 正确：使用 read_index 确保线性化
fn linearizable_read(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
    let read_barrier = self.raft.read_index()?;
    self.wait_applied(read_barrier)?;
    self.storage.get(key)
}
```

## 🏗️ 事务与补偿陷阱

### Saga 补偿幂等性问题

**陷阱**: 补偿必须可重试；避免副作用不可逆导致"补偿失败"。

```rust
// ❌ 错误：补偿操作不可重入
impl SagaStep for PaymentStep {
    fn compensate(&mut self) -> Result<(), Error> {
        // 如果重复调用会导致问题
        self.bank_account.refund(self.amount);
        Ok(())
    }
}

// ✅ 正确：补偿操作幂等
impl SagaStep for PaymentStep {
    fn compensate(&mut self) -> Result<(), Error> {
        if !self.refunded {
            self.bank_account.refund(self.amount);
            self.refunded = true;
        }
        Ok(())
    }
}
```

**最佳实践**:

- 所有补偿操作都应该是幂等的
- 使用状态标记避免重复执行
- 提供补偿操作的验证机制

### TCC 资源泄漏

**陷阱**: Try 阶段预留的资源在超时后没有正确释放。

```rust
// ❌ 错误：资源可能泄漏
fn try_reserve(&mut self, resource_id: &str) -> Result<(), Error> {
    self.resources.insert(resource_id.to_string(), Reserved);
    // 如果后续 Confirm/Cancel 失败，资源就泄漏了
}

// ✅ 正确：使用 TTL 自动释放
fn try_reserve(&mut self, resource_id: &str) -> Result<(), Error> {
    let reservation = Reservation {
        id: resource_id.to_string(),
        expires_at: Instant::now() + Duration::from_secs(30),
    };
    self.reservations.insert(resource_id.to_string(), reservation);
    Ok(())
}
```

## 🌐 网络与通信陷阱

### 重试风暴问题

**陷阱**: 无限制的重试导致系统过载。

```rust
// ❌ 错误：可能导致重试风暴
fn call_with_retry(&self, operation: &str) -> Result<Vec<u8>, Error> {
    loop {
        match self.call(operation) {
            Ok(result) => return Ok(result),
            Err(_) => {
                thread::sleep(Duration::from_millis(100)); // 固定延迟
                continue; // 无限重试
            }
        }
    }
}

// ✅ 正确：有界的指数退避
fn call_with_retry(&self, operation: &str) -> Result<Vec<u8>, Error> {
    let mut delay = Duration::from_millis(10);
    for attempt in 0..self.max_retries {
        match self.call(operation) {
            Ok(result) => return Ok(result),
            Err(_) if attempt == self.max_retries - 1 => return Err(Error::MaxRetriesExceeded),
            Err(_) => {
                thread::sleep(delay);
                delay = delay * 2; // 指数退避
            }
        }
    }
    Err(Error::MaxRetriesExceeded)
}
```

### 网络分区处理错误

**陷阱**: 没有正确处理网络分区导致的数据不一致。

```rust
// ❌ 错误：忽略网络分区
fn replicate(&mut self, data: &[u8]) -> Result<(), Error> {
    for node in &self.nodes {
        // 如果网络分区，某些节点不可达，但仍会继续
        node.send(data)?;
    }
    Ok(())
}

// ✅ 正确：检测和处理网络分区
fn replicate(&mut self, data: &[u8]) -> Result<(), Error> {
    let mut acks = 0;
    let required_acks = self.quorum_size();
    
    for node in &self.nodes {
        match node.send_with_timeout(data, Duration::from_millis(100)) {
            Ok(_) => acks += 1,
            Err(Error::Timeout) => {
                // 记录超时，但不立即失败
                self.mark_node_unreachable(node);
            }
            Err(e) => return Err(e),
        }
    }
    
    if acks >= required_acks {
        Ok(())
    } else {
        Err(Error::InsufficientAcks)
    }
}
```

## ⏰ 时间与时钟陷阱

### 时钟回拨问题

**陷阱**: 使用墙钟时间导致租约读安全性破坏。

```rust
// ❌ 错误：使用墙钟时间
fn is_lease_valid(&self, lease: &Lease) -> bool {
    let now = SystemTime::now(); // 可能回拨！
    now < lease.expires_at
}

// ✅ 正确：使用单调时钟
fn is_lease_valid(&self, lease: &Lease) -> bool {
    let now = Instant::now(); // 单调时钟
    now < lease.expires_at
}
```

### 选举超时配置错误

**陷阱**: 选举超时区间配置不当导致频繁选举或收敛慢。

```rust
// ❌ 错误：超时区间过小，容易冲突
let election_timeout = Duration::from_millis(100);
let heartbeat_interval = Duration::from_millis(50);

// ✅ 正确：合理的超时配置
let heartbeat_interval = Duration::from_millis(100);
let election_timeout_min = Duration::from_millis(250);  // 2.5 * heartbeat
let election_timeout_max = Duration::from_millis(500);  // 5 * heartbeat
```

## 🔍 调试工具与技巧

### 日志记录最佳实践

```rust
// ✅ 好的日志记录
tracing::info!(
    term = %self.current_term,
    commit_index = %self.commit_index,
    last_applied = %self.last_applied,
    "Raft state updated"
);

// ✅ 关键操作的详细日志
tracing::debug!(
    node_id = %node_id,
    prev_log_index = %prev_log_index,
    prev_log_term = %prev_log_term,
    entries_count = entries.len(),
    "Sending AppendEntries"
);
```

### 监控指标设置

```rust
// ✅ 关键指标监控
let metrics = Metrics {
    election_count: Counter::new("raft_elections_total"),
    append_entries_latency: Histogram::new("raft_append_entries_duration_seconds"),
    commit_index: Gauge::new("raft_commit_index"),
    last_applied: Gauge::new("raft_last_applied"),
};
```

### 测试策略

```rust
// ✅ 边界条件测试
#[test]
fn test_majority_calculation() {
    assert_eq!(calculate_majority(3), 2);
    assert_eq!(calculate_majority(4), 3);
    assert_eq!(calculate_majority(5), 3);
}

// ✅ 故障注入测试
#[test]
fn test_network_partition_recovery() {
    let mut cluster = create_test_cluster(5);
    
    // 注入网络分区
    cluster.partition(vec![0, 1], vec![2, 3, 4]);
    
    // 验证只有多数派侧能前进
    assert!(cluster.can_progress(vec![2, 3, 4]));
    assert!(!cluster.can_progress(vec![0, 1]));
    
    // 恢复网络
    cluster.heal_partition();
    
    // 验证最终一致性
    assert!(cluster.is_consistent());
}
```

## 🚨 性能陷阱

### 基准测试错误

**陷阱**: 避免使用 `test::Bencher`，推荐 Criterion。

```rust
// ❌ 错误：使用 test::Bencher
#[bench]
fn bench_consensus(b: &mut test::Bencher) {
    b.iter(|| {
        // 基准测试代码
    });
}

// ✅ 正确：使用 Criterion
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_consensus(c: &mut Criterion) {
    c.bench_function("consensus", |b| {
        b.iter(|| {
            // 基准测试代码
        });
    });
}

criterion_group!(benches, bench_consensus);
criterion_main!(benches);
```

### 内存泄漏检测

```rust
// ✅ 定期检查内存使用
fn monitor_memory_usage() {
    let memory_usage = get_memory_usage();
    if memory_usage > MEMORY_THRESHOLD {
        tracing::warn!("High memory usage: {} bytes", memory_usage);
        // 触发垃圾回收或清理缓存
    }
}
```

## 📋 检查清单

在部署分布式系统前，请确认：

- [ ] 多数派计算正确（`> n/2`）
- [ ] 一致性级别语义明确
- [ ] 所有补偿操作都是幂等的
- [ ] 重试策略有界且有退避
- [ ] 使用单调时钟进行时间判断
- [ ] 选举超时配置合理
- [ ] 关键操作有详细日志
- [ ] 设置了必要的监控指标
- [ ] 有完整的故障注入测试
- [ ] 性能基准测试使用 Criterion

## 🔗 相关资源

- [分布式系统测试指南](./testing/README.md)
- [故障模型与容错](./failure/README.md)
- [一致性模型详解](./consistency/README.md)
- [共识算法实现](./consensus/README.md)

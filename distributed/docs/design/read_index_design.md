# Read Index 技术设计文档

**版本**: v1.0  
**日期**: 2025年10月17日  
**状态**: 设计阶段  
**优先级**: 🔴 P0 - 最高

---

## 📋 文档概览

### 目标

实现Read Index机制，提供线性化读取能力，对齐2025年Raft实现的行业标准。

### 预期收益

- **性能提升**: 读取性能提升5-10x
- **读取延迟**: 从25ms降低到2-5ms
- **吞吐量**: 读取QPS从30K提升到150K+
- **行业对标**: 与etcd、TiKV等主流实现对齐

### 资源估算

- **开发时间**: 2周
- **测试时间**: 1周
- **开发人员**: 2名核心开发者
- **审查人员**: 1名技术专家

---

## 🎯 背景和动机

### 当前问题

1. **只读查询性能低**: 所有读取都需要通过Raft日志，性能受限
2. **读取延迟高**: 每次读取需要等待日志提交，延迟约25ms
3. **资源浪费**: 只读查询占用日志空间和网络带宽
4. **扩展性差**: 无法从Follower节点读取，Leader负载过高

### 行业标准

**Bodega (2025)**: 通过Roster Leases实现线性化本地读取，性能提升5.6-13.1倍

**etcd**: 实现Read Index和Lease Read，支持高性能只读查询

**TiKV**: Multi-Raft + Read Index，支持大规模分布式读取

### 为什么需要Read Index

Read Index是Raft论文中提出的优化方案，允许Leader在不写入日志的情况下提供线性化读取：

1. **性能优化**: 避免日志写入和复制的开销
2. **延迟降低**: 只需一次心跳确认，延迟降低80%+
3. **吞吐提升**: 释放日志带宽，提升10倍以上吞吐
4. **扩展性**: 为后续Lease Read奠定基础

---

## 🏗️ 技术设计

### 核心概念

#### Read Index的工作原理

```text
1. Leader收到只读请求
2. 记录当前commit_index（read_index）
3. 向多数派发送心跳，确认自己仍是Leader
4. 等待apply_index >= read_index
5. 从状态机读取数据并返回
```

#### 线性化保证

```text
条件1: 读取时Leader身份有效
条件2: 读取的状态 >= 请求时的commit状态
条件3: commit状态包含所有已确认的写入
```

### 数据结构设计

#### ReadIndexRequest

```rust
/// Read Index请求
#[derive(Debug, Clone)]
pub struct ReadIndexRequest {
    /// 请求ID（用于追踪）
    pub request_id: u64,
    
    /// 请求时间戳
    pub timestamp: Instant,
    
    /// 请求上下文（可选，透传给应用层）
    pub context: Vec<u8>,
}
```

#### ReadIndexResponse

```rust
/// Read Index响应
#[derive(Debug, Clone)]
pub struct ReadIndexResponse {
    /// 请求ID
    pub request_id: u64,
    
    /// 可安全读取的索引
    pub read_index: LogIndex,
    
    /// 响应时间戳
    pub timestamp: Instant,
    
    /// 是否成功
    pub success: bool,
    
    /// 错误信息（如果失败）
    pub error: Option<String>,
}
```

#### ReadIndexContext

```rust
/// Read Index上下文（Leader内部维护）
struct ReadIndexContext {
    /// 请求ID
    request_id: u64,
    
    /// 读取索引
    read_index: LogIndex,
    
    /// 请求时间
    request_time: Instant,
    
    /// 等待的心跳确认数
    acks: HashSet<NodeId>,
    
    /// 需要的确认数（多数派）
    required_acks: usize,
    
    /// 回调通道
    callback: oneshot::Sender<ReadIndexResponse>,
}
```

### API设计

#### 同步API

```rust
pub trait ReadIndex {
    /// 请求Read Index
    /// 
    /// # 参数
    /// - `context`: 可选的上下文数据
    /// 
    /// # 返回
    /// - `Ok(LogIndex)`: 可安全读取的日志索引
    /// - `Err(ReadIndexError)`: 错误（非Leader、网络故障等）
    fn read_index(&mut self, context: Option<Vec<u8>>) -> Result<LogIndex, ReadIndexError>;
    
    /// 等待apply索引达到指定值
    /// 
    /// # 参数
    /// - `index`: 要等待的日志索引
    /// - `timeout`: 超时时间
    /// 
    /// # 返回
    /// - `Ok(())`: 已应用到指定索引
    /// - `Err(WaitApplyError)`: 超时或其他错误
    fn wait_applied(&self, index: LogIndex, timeout: Duration) -> Result<(), WaitApplyError>;
}
```

#### 异步API

```rust
#[cfg(feature = "runtime-tokio")]
pub trait ReadIndexAsync {
    /// 异步请求Read Index
    async fn read_index_async(&mut self, context: Option<Vec<u8>>) 
        -> Result<LogIndex, ReadIndexError>;
    
    /// 异步等待应用
    async fn wait_applied_async(&self, index: LogIndex) 
        -> Result<(), WaitApplyError>;
    
    /// 完整的线性化读取（Read Index + 等待 + 读取）
    async fn linearizable_read<F, R>(&mut self, read_fn: F) 
        -> Result<R, ReadError>
    where
        F: FnOnce(&StateMachine) -> R;
}
```

### 核心算法

#### Leader端处理流程

```rust
impl RaftNode {
    /// 处理Read Index请求
    pub fn handle_read_index(&mut self, request: ReadIndexRequest) 
        -> Result<ReadIndexResponse, ReadIndexError> {
        // 1. 检查自己是否是Leader
        if self.role != Role::Leader {
            return Err(ReadIndexError::NotLeader {
                leader_id: self.leader_id.clone(),
            });
        }
        
        // 2. 记录当前commit_index作为read_index
        let read_index = self.commit_index;
        
        // 3. 创建Read Index上下文
        let ctx = ReadIndexContext {
            request_id: request.request_id,
            read_index,
            request_time: request.timestamp,
            acks: HashSet::new(),
            required_acks: self.majority_size(),
            callback: oneshot::channel().0,
        };
        
        // 4. 如果集群只有一个节点，直接返回
        if self.peers.is_empty() {
            return Ok(ReadIndexResponse {
                request_id: request.request_id,
                read_index,
                timestamp: Instant::now(),
                success: true,
                error: None,
            });
        }
        
        // 5. 保存上下文
        self.pending_read_index.insert(request.request_id, ctx);
        
        // 6. 向所有Follower发送心跳
        self.broadcast_heartbeat()?;
        
        // 7. 等待多数派确认（异步实现中返回Future）
        Ok(ReadIndexResponse {
            request_id: request.request_id,
            read_index,
            timestamp: Instant::now(),
            success: true,
            error: None,
        })
    }
    
    /// 处理心跳响应（用于Read Index确认）
    pub fn handle_heartbeat_response(&mut self, 
        node_id: NodeId, 
        response: HeartbeatResponse
    ) -> Result<(), RaftError> {
        // 1. 更新节点状态
        self.update_peer_state(node_id.clone(), &response);
        
        // 2. 检查所有待处理的Read Index请求
        let mut completed_requests = Vec::new();
        
        for (request_id, ctx) in &mut self.pending_read_index {
            // 记录确认
            ctx.acks.insert(node_id.clone());
            
            // 检查是否达到多数派
            if ctx.acks.len() >= ctx.required_acks {
                completed_requests.push(*request_id);
            }
        }
        
        // 3. 完成达到多数派的请求
        for request_id in completed_requests {
            if let Some(ctx) = self.pending_read_index.remove(&request_id) {
                let _ = ctx.callback.send(ReadIndexResponse {
                    request_id,
                    read_index: ctx.read_index,
                    timestamp: Instant::now(),
                    success: true,
                    error: None,
                });
            }
        }
        
        Ok(())
    }
}
```

#### 客户端使用流程

```rust
/// 线性化读取完整示例
pub async fn linearizable_read_example(raft: &mut RaftNode) -> Result<Vec<u8>, ReadError> {
    // 1. 请求Read Index
    let read_index = raft.read_index(None).await?;
    
    // 2. 等待状态机应用到read_index
    raft.wait_applied(read_index).await?;
    
    // 3. 从状态机读取数据
    let data = raft.read_state_machine(|sm| {
        sm.get_data()
    })?;
    
    Ok(data)
}
```

### 错误处理

#### 错误类型定义

```rust
/// Read Index错误
#[derive(Debug, thiserror::Error)]
pub enum ReadIndexError {
    /// 不是Leader
    #[error("Not leader, current leader: {leader_id:?}")]
    NotLeader { leader_id: Option<NodeId> },
    
    /// 任期变更
    #[error("Term changed during read index request")]
    TermChanged,
    
    /// 超时
    #[error("Read index request timeout after {timeout:?}")]
    Timeout { timeout: Duration },
    
    /// 网络错误
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    
    /// 内部错误
    #[error("Internal error: {0}")]
    Internal(String),
}

/// 等待应用错误
#[derive(Debug, thiserror::Error)]
pub enum WaitApplyError {
    /// 超时
    #[error("Wait apply timeout after {timeout:?}")]
    Timeout { timeout: Duration },
    
    /// 节点角色变更
    #[error("Node role changed")]
    RoleChanged,
    
    /// 取消
    #[error("Wait apply cancelled")]
    Cancelled,
}
```

#### 错误恢复策略

```rust
/// Read Index重试策略
pub struct ReadIndexRetryPolicy {
    max_retries: usize,
    initial_backoff: Duration,
    max_backoff: Duration,
}

impl ReadIndexRetryPolicy {
    pub async fn execute_with_retry<F, T>(
        &self,
        mut f: F,
    ) -> Result<T, ReadIndexError>
    where
        F: FnMut() -> Result<T, ReadIndexError>,
    {
        let mut backoff = self.initial_backoff;
        
        for attempt in 0..=self.max_retries {
            match f() {
                Ok(result) => return Ok(result),
                Err(ReadIndexError::NotLeader { .. }) => {
                    // 不重试，直接返回
                    return Err(ReadIndexError::NotLeader { leader_id: None });
                }
                Err(ReadIndexError::Timeout { .. }) if attempt < self.max_retries => {
                    // 超时重试
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, self.max_backoff);
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(ReadIndexError::Internal("Max retries exceeded".to_string()))
    }
}
```

### 性能优化

#### 批量处理

```rust
/// 批量Read Index请求
pub struct BatchReadIndexRequest {
    requests: Vec<ReadIndexRequest>,
}

impl RaftNode {
    /// 批量处理Read Index请求
    pub fn handle_batch_read_index(&mut self, batch: BatchReadIndexRequest) 
        -> Result<Vec<ReadIndexResponse>, ReadIndexError> {
        // 1. 所有请求使用相同的read_index
        let read_index = self.commit_index;
        
        // 2. 只发送一次心跳
        self.broadcast_heartbeat()?;
        
        // 3. 等待确认后批量返回
        batch.requests.iter().map(|req| {
            Ok(ReadIndexResponse {
                request_id: req.request_id,
                read_index,
                timestamp: Instant::now(),
                success: true,
                error: None,
            })
        }).collect()
    }
}
```

#### 心跳优化

```rust
/// 心跳聚合
impl RaftNode {
    /// 智能心跳：合并Read Index和Raft心跳
    pub fn smart_heartbeat(&mut self) -> Result<(), RaftError> {
        // 1. 检查是否有待处理的Read Index请求
        let has_pending_reads = !self.pending_read_index.is_empty();
        
        // 2. 检查距离上次心跳的时间
        let elapsed = self.last_heartbeat.elapsed();
        let should_heartbeat = elapsed >= self.heartbeat_interval;
        
        // 3. 如果需要心跳或有待处理的读请求，发送心跳
        if should_heartbeat || has_pending_reads {
            self.broadcast_heartbeat()?;
            self.last_heartbeat = Instant::now();
        }
        
        Ok(())
    }
}
```

---

## 🧪 测试策略

### 单元测试

#### 测试用例列表

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试1: 单节点集群Read Index
    #[test]
    fn test_read_index_single_node() {
        let mut raft = create_single_node_raft();
        
        let read_index = raft.read_index(None).unwrap();
        assert_eq!(read_index, raft.commit_index);
    }
    
    /// 测试2: 多节点集群Read Index
    #[tokio::test]
    async fn test_read_index_multi_node() {
        let mut cluster = create_three_node_cluster();
        let leader = cluster.leader_mut();
        
        let read_index = leader.read_index(None).await.unwrap();
        assert!(read_index > LogIndex(0));
    }
    
    /// 测试3: 非Leader节点拒绝Read Index
    #[test]
    fn test_read_index_not_leader() {
        let mut cluster = create_three_node_cluster();
        let follower = cluster.follower_mut(0);
        
        let result = follower.read_index(None);
        assert!(matches!(result, Err(ReadIndexError::NotLeader { .. })));
    }
    
    /// 测试4: 心跳确认机制
    #[tokio::test]
    async fn test_read_index_heartbeat_confirmation() {
        let mut cluster = create_three_node_cluster();
        let leader = cluster.leader_mut();
        
        // 发起Read Index请求
        let read_index_future = leader.read_index(None);
        
        // 模拟心跳响应
        cluster.deliver_heartbeats();
        
        // 验证请求完成
        let read_index = read_index_future.await.unwrap();
        assert!(read_index > LogIndex(0));
    }
    
    /// 测试5: 任期变更处理
    #[tokio::test]
    async fn test_read_index_term_change() {
        let mut cluster = create_three_node_cluster();
        let leader = cluster.leader_mut();
        
        // 发起Read Index请求
        let read_index_future = leader.read_index(None);
        
        // 触发选举，任期变更
        cluster.trigger_election();
        
        // 验证请求失败
        let result = read_index_future.await;
        assert!(matches!(result, Err(ReadIndexError::TermChanged)));
    }
    
    /// 测试6: 超时处理
    #[tokio::test]
    async fn test_read_index_timeout() {
        let mut cluster = create_three_node_cluster();
        let leader = cluster.leader_mut();
        
        // 断开网络
        cluster.partition_leader();
        
        // 发起Read Index请求并等待超时
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            leader.read_index(None)
        ).await;
        
        assert!(result.is_err() || matches!(
            result.unwrap(),
            Err(ReadIndexError::Timeout { .. })
        ));
    }
    
    /// 测试7: 批量Read Index
    #[tokio::test]
    async fn test_batch_read_index() {
        let mut cluster = create_three_node_cluster();
        let leader = cluster.leader_mut();
        
        // 创建批量请求
        let batch = BatchReadIndexRequest {
            requests: (0..10).map(|i| ReadIndexRequest {
                request_id: i,
                timestamp: Instant::now(),
                context: vec![],
            }).collect(),
        };
        
        // 处理批量请求
        let responses = leader.handle_batch_read_index(batch).await.unwrap();
        
        // 验证所有请求都成功
        assert_eq!(responses.len(), 10);
        assert!(responses.iter().all(|r| r.success));
    }
}
```

### 集成测试

#### 端到端测试

```rust
#[tokio::test]
async fn test_linearizable_read_e2e() {
    // 1. 创建3节点集群
    let mut cluster = TestCluster::new(3).await;
    
    // 2. 写入数据
    cluster.write("key1", "value1").await.unwrap();
    
    // 3. 从Leader读取
    let value = cluster.linearizable_read("key1").await.unwrap();
    assert_eq!(value, "value1");
    
    // 4. 再次写入
    cluster.write("key1", "value2").await.unwrap();
    
    // 5. 验证读取到最新值
    let value = cluster.linearizable_read("key1").await.unwrap();
    assert_eq!(value, "value2");
}
```

#### 网络分区测试

```rust
#[tokio::test]
async fn test_read_index_with_partition() {
    let mut cluster = TestCluster::new(5).await;
    
    // 写入数据
    cluster.write("key1", "value1").await.unwrap();
    
    // 创建网络分区：{Leader, Node1} | {Node2, Node3, Node4}
    cluster.partition(vec![0, 1], vec![2, 3, 4]);
    
    // Leader失去多数派，Read Index应该失败
    let result = cluster.node(0).linearizable_read("key1").await;
    assert!(result.is_err());
    
    // 恢复网络
    cluster.heal_partition();
    
    // 重新选举后，Read Index应该成功
    cluster.wait_for_election().await;
    let value = cluster.linearizable_read("key1").await.unwrap();
    assert_eq!(value, "value1");
}
```

### 性能测试

#### 基准测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_read_index(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut cluster = rt.block_on(TestCluster::new(3));
    
    c.bench_function("read_index", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(cluster.leader().read_index(None).await.unwrap())
            })
        })
    });
}

fn bench_linearizable_read(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut cluster = rt.block_on(TestCluster::new(3));
    rt.block_on(cluster.write("key1", "value1")).unwrap();
    
    c.bench_function("linearizable_read", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(cluster.linearizable_read("key1").await.unwrap())
            })
        })
    });
}

criterion_group!(benches, bench_read_index, bench_linearizable_read);
criterion_main!(benches);
```

---

## 📈 性能目标

### 延迟目标

| 操作 | 当前延迟 | 目标延迟 | 改进幅度 |
|------|---------|---------|---------|
| 普通读取 | 25ms | 5ms | **80%改进** |
| Read Index | N/A | 2ms | **新功能** |
| 批量读取(10个) | 250ms | 6ms | **97%改进** |

### 吞吐量目标

| 指标 | 当前 | 目标 | 改进幅度 |
|------|------|------|---------|
| 读取QPS | 30K | 150K | **5x改进** |
| 并发读取 | 1K | 10K | **10x改进** |
| CPU使用率 | 80% | 40% | **50%改进** |

### 资源使用

| 资源 | 当前 | 目标 | 改进 |
|------|------|------|------|
| 网络带宽 | 100MB/s | 20MB/s | 节省80% |
| 磁盘IO | 1000 IOPS | 100 IOPS | 节省90% |
| 内存使用 | 2GB | 1GB | 节省50% |

---

## 📝 实施计划

### Week 1: 核心实现（Day 1-7）

**Day 1-2: 数据结构和API设计**:

- [ ] 定义ReadIndexRequest/Response
- [ ] 定义ReadIndex trait
- [ ] 实现错误类型
- [ ] 编写API文档

**Day 3-5: Leader端实现**:

- [ ] 实现handle_read_index方法
- [ ] 实现心跳确认逻辑
- [ ] 实现超时处理
- [ ] 单元测试

**Day 6-7: 客户端集成**:

- [ ] 实现wait_applied方法
- [ ] 实现线性化读取封装
- [ ] 集成测试
- [ ] 代码审查

### Week 2: 优化和测试（Day 8-14）

**Day 8-10: 性能优化**:

- [ ] 批量Read Index实现
- [ ] 心跳聚合优化
- [ ] 性能基准测试
- [ ] 性能调优

**Day 11-13: 完整测试**:

- [ ] 端到端测试
- [ ] 网络分区测试
- [ ] 压力测试
- [ ] 混沌测试

**Day 14: 文档和发布**:

- [ ] 完善文档
- [ ] 更新示例代码
- [ ] 准备发布说明
- [ ] 代码合并

---

## 🔗 参考资料

### 学术论文

1. **Raft论文**: "In Search of an Understandable Consensus Algorithm" (Ongaro & Ousterhout, 2014)
   - Section 6.4: Processing read-only queries more efficiently

2. **Bodega论文**: "Bodega: Local Read Linearizability" (2025)
   - 最新的线性化读取优化技术

### 开源实现

1. **etcd**: <https://github.com/etcd-io/etcd>
   - `raft/read_only.go`: Read Index实现参考

2. **TiKV**: <https://github.com/tikv/tikv>
   - `components/raftstore/`: Multi-Raft + Read Index

3. **Hashicorp Raft**: <https://github.com/hashicorp/raft>
   - Leader lease实现参考

### 博客文章

1. **etcd博客**: "Linearizable Reads in etcd"
2. **TiKV博客**: "How TiKV Reads and Writes"
3. **Raft官网**: <https://raft.github.io/>

---

## ✅ 验收标准

### 功能完整性

- [ ] Read Index API完整实现
- [ ] 支持同步和异步调用
- [ ] 正确的错误处理和重试
- [ ] 完整的测试覆盖

### 性能要求

- [ ] Read Index延迟 < 5ms (P99)
- [ ] 吞吐量 >= 150K QPS
- [ ] CPU使用率 < 50%
- [ ] 内存使用增加 < 100MB

### 质量要求

- [ ] 单元测试覆盖率 >= 85%
- [ ] 集成测试通过率 100%
- [ ] 无clippy警告
- [ ] 文档完整

### 兼容性

- [ ] 向后兼容现有API
- [ ] 支持单节点和多节点集群
- [ ] 与现有Raft实现兼容

---

## 🚀 后续计划

### Phase 2: Lease Read (Week 3-4)

在Read Index基础上实现Lease Read，进一步降低延迟：

- 基于时间的租约机制
- 租约续期和过期处理
- 时钟偏移检测和处理

### Phase 3: Follower Read (Month 2)

支持从Follower读取，分散Leader负载：

- Follower Read Index
- 读取一致性保证
- 负载均衡策略

---

**文档维护者**: Core Development Team  
**最后更新**: 2025年10月17日  
**下次审核**: 开发完成后

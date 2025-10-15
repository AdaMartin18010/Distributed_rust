# 实验清单（Executable Checklist）

本文档提供了分布式系统各组件的详细实验检查清单，每个实验都包含前置条件、操作步骤、观测指标和通过判据。

## 目录

- [实验清单（Executable Checklist）](#实验清单executable-checklist)
  - [目录](#目录)
  - [📋 实验总览](#-实验总览)
  - [🔄 一致性实验](#-一致性实验)
    - [1. 线性化一致性验证](#1-线性化一致性验证)
    - [2. 因果一致性验证](#2-因果一致性验证)
    - [3. 最终一致性收敛测试](#3-最终一致性收敛测试)
  - [📊 复制实验](#-复制实验)
    - [4. Quorum 读写验证](#4-quorum-读写验证)
    - [5. 网络分区下的复制行为](#5-网络分区下的复制行为)
  - [🗳️ 共识实验](#️-共识实验)
    - [6. Raft 领导者选举](#6-raft-领导者选举)
    - [7. 日志冲突与回退](#7-日志冲突与回退)
  - [💾 存储实验](#-存储实验)
    - [8. WAL 持久化测试](#8-wal-持久化测试)
  - [🔄 事务实验](#-事务实验)
    - [9. SAGA 补偿测试](#9-saga-补偿测试)
  - [🌐 传输/调度实验](#-传输调度实验)
    - [10. 重试退避测试](#10-重试退避测试)
  - [👥 成员管理实验](#-成员管理实验)
    - [11. SWIM 故障检测测试](#11-swim-故障检测测试)
  - [📊 可观测性实验](#-可观测性实验)
    - [12. 分布式追踪测试](#12-分布式追踪测试)
  - [🔧 实验工具和命令](#-实验工具和命令)
    - [常用测试命令](#常用测试命令)
    - [故障注入工具](#故障注入工具)
    - [监控和调试](#监控和调试)
  - [📝 实验报告模板](#-实验报告模板)
    - [实验报告结构](#实验报告结构)
  - [🔗 相关资源](#-相关资源)

## 📋 实验总览

| 类别 | 实验数量 | 预计时间 | 难度 |
|------|----------|----------|------|
| 一致性 | 8 | 4小时 | 中等 |
| 复制 | 6 | 3小时 | 中等 |
| 共识 | 10 | 6小时 | 高 |
| 存储 | 5 | 3小时 | 中等 |
| 事务 | 7 | 4小时 | 高 |
| 传输/调度 | 6 | 3小时 | 中等 |
| 成员管理 | 5 | 2小时 | 中等 |
| 可观测性 | 4 | 2小时 | 低 |

## 🔄 一致性实验

### 1. 线性化一致性验证

**前置条件**:

- [ ] 3节点集群运行正常
- [ ] 所有节点日志级别设为 `debug`
- [ ] Jepsen 风格测试框架已配置

**操作步骤**:

```bash
# 1. 启动集群
cargo run --example consistency_demo -- --nodes 3

# 2. 运行线性化测试
cargo test --test experiments_linearizability -- --nocapture

# 3. 注入网络分区
sudo tc qdisc add dev lo root netem delay 100ms loss 10%
```

**观测指标**:

- 操作历史记录
- 线性化检查结果
- 分区恢复时间

**通过判据**:

- [ ] 所有操作历史通过线性化检查
- [ ] 分区期间最多只有一个分区能前进
- [ ] 分区恢复后系统状态一致

### 2. 因果一致性验证

**前置条件**:

- [ ] 向量时钟实现可用
- [ ] 多客户端并发测试工具

**操作步骤**:

```rust
// 测试代码示例
#[test]
fn test_causal_consistency() {
    let mut clients = create_clients(5);
    
    // 建立因果依赖：A -> B -> C
    clients[0].write("key1", "value1").unwrap();
    let val1 = clients[0].read("key1").unwrap();
    clients[1].write("key2", format!("value2-{}", val1)).unwrap();
    
    // 验证因果顺序在所有节点上一致
    verify_causal_order(&clients);
}
```

**通过判据**:

- [ ] 因果依赖关系在所有节点上保持
- [ ] 没有因果违规（读到未来的值）

### 3. 最终一致性收敛测试

**前置条件**:

- [ ] 反熵机制已实现
- [ ] 网络延迟模拟器

**操作步骤**:

```bash
# 1. 启动多节点集群
cargo run --example e2e_replication -- --nodes 5 --consistency eventual

# 2. 写入数据到不同节点
curl -X POST http://node1:8080/write -d '{"key":"test","value":"data1"}'
curl -X POST http://node2:8080/write -d '{"key":"test","value":"data2"}'

# 3. 等待收敛
sleep 30

# 4. 检查所有节点状态
for i in {1..5}; do
    curl http://node$i:8080/read/test
done
```

**通过判据**:

- [ ] 所有节点最终收敛到相同状态
- [ ] 收敛时间在预期范围内（< 30秒）

## 📊 复制实验

### 4. Quorum 读写验证

**前置条件**:

- [ ] 5节点集群
- [ ] 可配置的 R/W/N 参数

**操作步骤**:

```rust
#[test]
fn test_quorum_read_write() {
    let configs = vec![
        (3, 3, 5), // R=3, W=3, N=5
        (2, 4, 5), // R=2, W=4, N=5
        (1, 5, 5), // R=1, W=5, N=5
    ];
    
    for (r, w, n) in configs {
        let mut replicator = LocalReplicator::new(n, r, w);
        
        // 测试写入
        let result = replicator.replicate("test_data", ConsistencyLevel::Quorum);
        assert!(result.is_ok());
        
        // 测试读取
        let value = replicator.read("test_data", ConsistencyLevel::Quorum);
        assert_eq!(value, Some("test_data".to_string()));
    }
}
```

**通过判据**:

- [ ] 满足 R+W>N 的配置能保证线性化读
- [ ] 不满足条件的配置可能出现读旧值

### 5. 网络分区下的复制行为

**前置条件**:

- [ ] 网络分区模拟工具
- [ ] 分区检测机制

**操作步骤**:

```bash
# 1. 启动5节点集群
cargo run --example e2e_replication -- --nodes 5

# 2. 分区：节点1,2 vs 节点3,4,5
iptables -A INPUT -s 127.0.0.2 -j DROP  # 阻止节点1访问节点3,4,5
iptables -A INPUT -s 127.0.0.3 -j DROP  # 阻止节点2访问节点3,4,5

# 3. 尝试写入
curl -X POST http://node1:8080/write -d '{"key":"partition_test","value":"from_minority"}'
curl -X POST http://node3:8080/write -d '{"key":"partition_test","value":"from_majority"}'

# 4. 恢复网络
iptables -F

# 5. 检查最终状态
```

**通过判据**:

- [ ] 多数派分区的写入成功
- [ ] 少数派分区的写入被拒绝或回滚
- [ ] 网络恢复后系统状态一致

## 🗳️ 共识实验

### 6. Raft 领导者选举

**前置条件**:

- [ ] Raft 实现可用
- [ ] 选举超时配置合理

**操作步骤**:

```rust
#[test]
fn test_raft_leader_election() {
    let mut nodes = create_raft_cluster(5);
    
    // 1. 初始状态检查
    assert!(nodes.iter().any(|n| n.state() == RaftState::Leader));
    
    // 2. 杀死领导者
    let leader_id = find_leader(&nodes);
    nodes[leader_id].kill();
    
    // 3. 等待新领导者选出
    thread::sleep(Duration::from_millis(1000));
    
    // 4. 验证新领导者
    let new_leader = find_leader(&nodes);
    assert_ne!(new_leader, leader_id);
    assert!(nodes[new_leader].state() == RaftState::Leader);
}
```

**通过判据**:

- [ ] 任何时候最多只有一个领导者
- [ ] 领导者故障后能在选举超时内选出新领导者
- [ ] 新领导者的日志包含所有已提交的条目

### 7. 日志冲突与回退

**前置条件**:

- [ ] 双领导者场景模拟器
- [ ] 日志冲突检测

**操作步骤**:

```rust
#[test]
fn test_log_conflict_and_rollback() {
    let mut nodes = create_raft_cluster(5);
    
    // 1. 创建网络分区导致双领导者
    partition_network(&nodes, vec![0, 1], vec![2, 3, 4]);
    
    // 2. 两个分区都写入数据
    nodes[0].propose("data_from_partition_1");
    nodes[2].propose("data_from_partition_2");
    
    // 3. 恢复网络
    heal_network(&nodes);
    
    // 4. 验证日志冲突解决
    let final_state = get_final_log_state(&nodes);
    assert!(final_state.is_consistent());
}
```

**通过判据**:

- [ ] 冲突的日志条目被正确回退
- [ ] 最终所有节点的日志一致
- [ ] 已提交的条目不会被覆盖

## 💾 存储实验

### 8. WAL 持久化测试

**前置条件**:

- [ ] WAL 实现可用
- [ ] 崩溃恢复机制

**操作步骤**:

```rust
#[test]
fn test_wal_persistence() {
    let mut storage = WalStorage::new("/tmp/test_wal");
    
    // 1. 写入数据
    storage.append_entry(&Entry { term: 1, index: 1, data: b"test_data" });
    storage.flush().unwrap();
    
    // 2. 模拟崩溃
    drop(storage);
    
    // 3. 恢复并验证数据
    let recovered_storage = WalStorage::new("/tmp/test_wal");
    let entries = recovered_storage.read_all_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].data, b"test_data");
}
```

**通过判据**:

- [ ] 崩溃后数据不丢失
- [ ] 恢复时间在可接受范围内
- [ ] WAL 文件完整性验证通过

## 🔄 事务实验

### 9. SAGA 补偿测试

**前置条件**:

- [ ] SAGA 实现可用
- [ ] 补偿操作幂等性验证

**操作步骤**:

```rust
#[test]
fn test_saga_compensation() {
    let mut saga = Saga::new();
    
    // 1. 添加多个步骤
    saga.add_step(Box::new(ReserveInventoryStep { amount: 10 }));
    saga.add_step(Box::new(ChargePaymentStep { amount: 100 }));
    saga.add_step(Box::new(ShipOrderStep { order_id: "123" }));
    
    // 2. 执行 SAGA，中间步骤失败
    let result = saga.execute();
    assert!(result.is_err());
    
    // 3. 验证补偿执行
    assert!(saga.compensation_executed());
    assert!(inventory_reserved() == 0); // 库存已释放
    assert!(payment_charged() == 0);    // 付款已退还
}
```

**通过判据**:

- [ ] 失败步骤之前的所有步骤都被补偿
- [ ] 补偿操作是幂等的
- [ ] 补偿后的系统状态与执行前一致

## 🌐 传输/调度实验

### 10. 重试退避测试

**前置条件**:

- [ ] 重试机制实现
- [ ] 退避策略配置

**操作步骤**:

```rust
#[test]
fn test_retry_backoff() {
    let mut client = RetryClient::new(
        FailingServer::new(0.5), // 50% 失败率
        RetryConfig {
            max_retries: 5,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
        }
    );
    
    let start = Instant::now();
    let result = client.call("test_operation");
    let duration = start.elapsed();
    
    // 验证退避效果
    assert!(duration >= Duration::from_millis(10 + 20 + 40 + 80 + 160));
    assert!(duration < Duration::from_millis(2000));
}
```

**通过判据**:

- [ ] 退避延迟按预期增长
- [ ] 总重试时间有上界
- [ ] 重试次数不超过配置的最大值

## 👥 成员管理实验

### 11. SWIM 故障检测测试

**前置条件**:

- [ ] SWIM 实现
- [ ] 故障模拟器

**操作步骤**:

```rust
#[test]
fn test_swim_failure_detection() {
    let mut nodes = create_swim_cluster(5);
    
    // 1. 初始状态检查
    assert!(all_nodes_alive(&nodes));
    
    // 2. 杀死一个节点
    nodes[2].kill();
    
    // 3. 等待故障检测
    thread::sleep(Duration::from_millis(2000));
    
    // 4. 验证故障检测
    let membership = nodes[0].get_membership();
    assert_eq!(membership.get_state(&nodes[2].id()), SwimState::Faulty);
}
```

**通过判据**:

- [ ] 故障节点能在超时内被检测到
- [ ] 误报率低于配置阈值
- [ ] 故障检测结果在所有节点上一致

## 📊 可观测性实验

### 12. 分布式追踪测试

**前置条件**:

- [ ] 分布式追踪实现
- [ ] 追踪数据收集器

**操作步骤**:

```rust
#[test]
fn test_distributed_tracing() {
    let tracer = DistributedTracer::new();
    
    // 1. 创建追踪上下文
    let trace_id = tracer.start_trace("user_request");
    
    // 2. 模拟跨服务调用
    let span1 = tracer.start_span(trace_id, "auth_service");
    tracer.finish_span(span1);
    
    let span2 = tracer.start_span(trace_id, "user_service");
    tracer.finish_span(span2);
    
    // 3. 完成追踪
    tracer.finish_trace(trace_id);
    
    // 4. 验证追踪数据
    let trace_data = tracer.get_trace(trace_id);
    assert_eq!(trace_data.spans.len(), 3); // root + 2 spans
    assert!(trace_data.duration > Duration::from_millis(0));
}
```

**通过判据**:

- [ ] 追踪数据完整记录
- [ ] 跨服务调用链路正确
- [ ] 追踪数据可以正确导出

## 🔧 实验工具和命令

### 常用测试命令

```bash
# 运行所有实验
cargo test --test experiments_linearizability
cargo test --test experiments_replication
cargo test --test experiments_consensus

# 运行基准测试
cargo bench

# 运行示例
cargo run --example e2e_replication
cargo run --example e2e_saga
cargo run --example comprehensive_demo
```

### 故障注入工具

```bash
# 网络延迟
sudo tc qdisc add dev lo root netem delay 100ms

# 网络丢包
sudo tc qdisc add dev lo root netem loss 10%

# 网络分区
sudo iptables -A INPUT -s 127.0.0.2 -j DROP

# 进程杀死
kill -9 <pid>

# 磁盘满
fallocate -l 1G /tmp/disk_full
```

### 监控和调试

```bash
# 查看日志
tail -f /var/log/distributed.log

# 查看指标
curl http://localhost:9090/metrics

# 查看追踪
curl http://localhost:16686/api/traces

# 性能分析
perf record -g cargo run --example performance_test
perf report
```

## 📝 实验报告模板

### 实验报告结构

```markdown
    # 实验名称

    ## 实验目标
    - 验证什么功能
    - 测试什么场景

    ## 实验环境
    - 硬件配置
    - 软件版本
    - 网络环境

    ## 实验步骤
    1. 准备阶段
    2. 执行阶段
    3. 验证阶段

    ## 实验结果
    - 关键指标
    - 性能数据
    - 错误日志

    ## 结论
    - 功能是否正常
    - 性能是否达标
    - 发现的问题

    ## 改进建议
    - 优化方向
    - 配置调整
    - 代码改进
```

## 🔗 相关资源

- [分布式系统测试指南](../testing/README.md)
- [故障模型与容错](../failure/README.md)
- [可观测性实践](../observability/README.md)
- [常见陷阱与调试](../PITFALLS.md)

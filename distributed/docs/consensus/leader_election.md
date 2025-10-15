# 领导者选举机制

> 分布式系统中的领导者选举算法和故障切换机制

## 目录

- [领导者选举机制](#领导者选举机制)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [选举目标](#选举目标)
    - [选举条件](#选举条件)
  - [🔧 Raft 选举机制](#-raft-选举机制)
    - [选举流程](#选举流程)
    - [选举超时机制](#选举超时机制)
    - [投票请求处理](#投票请求处理)
  - [🔄 故障切换机制](#-故障切换机制)
    - [领导者故障检测](#领导者故障检测)
    - [选举过程实现](#选举过程实现)
  - [⚡ 性能优化](#-性能优化)
    - [选举超时优化](#选举超时优化)
    - [预投票机制](#预投票机制)
  - [🧪 测试策略](#-测试策略)
    - [选举测试](#选举测试)
  - [🔍 故障排查](#-故障排查)
    - [常见问题](#常见问题)
      - [1. 选举超时过长](#1-选举超时过长)
      - [2. 频繁选举](#2-频繁选举)
      - [3. 双领导者](#3-双领导者)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

领导者选举是分布式共识算法的核心组件，确保在任何时刻系统中最多只有一个领导者，并在领导者故障时能够快速选出新的领导者。

## 🎯 核心概念

### 选举目标

- **唯一性**: 任何时候最多只有一个领导者
- **活性**: 在网络稳定的情况下最终能选出领导者
- **安全性**: 不会产生相互矛盾的领导者

### 选举条件

- **多数派支持**: 领导者必须获得多数派节点的支持
- **任期单调**: 选举基于单调递增的任期号
- **日志完整性**: 新领导者必须包含所有已提交的日志条目

## 🔧 Raft 选举机制

### 选举流程

```rust
// Raft 选举状态
#[derive(Debug, Clone, PartialEq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

// 选举配置
#[derive(Debug, Clone)]
pub struct ElectionConfig {
    pub election_timeout_min: Duration,
    pub election_timeout_max: Duration,
    pub heartbeat_interval: Duration,
    pub max_election_timeout: Duration,
}

impl Default for ElectionConfig {
    fn default() -> Self {
        Self {
            election_timeout_min: Duration::from_millis(150),
            election_timeout_max: Duration::from_millis(300),
            heartbeat_interval: Duration::from_millis(50),
            max_election_timeout: Duration::from_millis(1000),
        }
    }
}
```

### 选举超时机制

```rust
use rand::Rng;
use tokio::time::{sleep, Duration};

pub struct ElectionTimer {
    config: ElectionConfig,
    current_timeout: Duration,
}

impl ElectionTimer {
    pub fn new(config: ElectionConfig) -> Self {
        let mut timer = Self {
            config,
            current_timeout: Duration::from_millis(0),
        };
        timer.reset_timeout();
        timer
    }
    
    pub fn reset_timeout(&mut self) {
        let mut rng = rand::thread_rng();
        let timeout_range = self.config.election_timeout_max 
            - self.config.election_timeout_min;
        let random_offset = rng.gen_range(0..=timeout_range.as_millis()) as u64;
        self.current_timeout = self.config.election_timeout_min 
            + Duration::from_millis(random_offset);
    }
    
    pub async fn wait_for_timeout(&mut self) {
        sleep(self.current_timeout).await;
    }
}
```

### 投票请求处理

```rust
#[derive(Debug, Clone)]
pub struct RequestVoteRequest {
    pub term: u64,
    pub candidate_id: String,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

pub struct RaftNode {
    state: RaftState,
    current_term: u64,
    voted_for: Option<String>,
    log: Vec<LogEntry>,
    // ... 其他字段
}

impl RaftNode {
    pub fn handle_request_vote(&mut self, request: RequestVoteRequest) -> RequestVoteResponse {
        // 1. 检查任期
        if request.term > self.current_term {
            self.current_term = request.term;
            self.voted_for = None;
            self.state = RaftState::Follower;
        }
        
        // 2. 检查投票条件
        let vote_granted = if request.term == self.current_term {
            // 检查是否已经投票给其他候选者
            let not_voted = self.voted_for.is_none() || 
                self.voted_for.as_ref() == Some(&request.candidate_id);
            
            // 检查日志完整性
            let log_up_to_date = self.is_log_up_to_date(
                request.last_log_index, 
                request.last_log_term
            );
            
            not_voted && log_up_to_date
        } else {
            false
        };
        
        // 3. 记录投票
        if vote_granted {
            self.voted_for = Some(request.candidate_id.clone());
        }
        
        RequestVoteResponse {
            term: self.current_term,
            vote_granted,
        }
    }
    
    fn is_log_up_to_date(&self, last_log_index: u64, last_log_term: u64) -> bool {
        let our_last_log = self.log.last();
        
        match our_last_log {
            None => true, // 我们的日志为空，任何日志都是最新的
            Some(our_entry) => {
                // 比较最后的日志任期
                if last_log_term > our_entry.term {
                    true
                } else if last_log_term == our_entry.term {
                    // 任期相同，比较索引
                    last_log_index >= our_entry.index
                } else {
                    false
                }
            }
        }
    }
}
```

## 🔄 故障切换机制

### 领导者故障检测

```rust
pub struct LeaderFailureDetector {
    last_heartbeat: Option<Instant>,
    heartbeat_timeout: Duration,
}

impl LeaderFailureDetector {
    pub fn new(heartbeat_timeout: Duration) -> Self {
        Self {
            last_heartbeat: None,
            heartbeat_timeout,
        }
    }
    
    pub fn on_heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
    }
    
    pub fn is_leader_alive(&self) -> bool {
        match self.last_heartbeat {
            None => false,
            Some(last) => {
                let elapsed = last.elapsed();
                elapsed < self.heartbeat_timeout
            }
        }
    }
}
```

### 选举过程实现

```rust
impl RaftNode {
    pub async fn start_election(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 转换为候选者状态
        self.state = RaftState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.node_id.clone());
        
        // 2. 重置选举超时
        self.election_timer.reset_timeout();
        
        // 3. 发送投票请求
        let votes = self.request_votes().await?;
        
        // 4. 检查是否获得多数派支持
        if votes >= self.majority_count() {
            self.become_leader().await?;
        } else {
            self.state = RaftState::Follower;
        }
        
        Ok(())
    }
    
    async fn request_votes(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut votes = 1; // 自己的一票
        let mut tasks = Vec::new();
        
        for peer in &self.peers {
            let request = RequestVoteRequest {
                term: self.current_term,
                candidate_id: self.node_id.clone(),
                last_log_index: self.last_log_index(),
                last_log_term: self.last_log_term(),
            };
            
            let peer_id = peer.id.clone();
            let task = tokio::spawn(async move {
                // 发送投票请求到对等节点
                self.send_request_vote(peer_id, request).await
            });
            
            tasks.push(task);
        }
        
        // 等待投票结果
        for task in tasks {
            if let Ok(Ok(response)) = task.await {
                if response.vote_granted {
                    votes += 1;
                }
            }
        }
        
        Ok(votes)
    }
    
    async fn become_leader(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = RaftState::Leader;
        
        // 初始化领导者状态
        for peer in &self.peers {
            self.next_index.insert(peer.id.clone(), self.last_log_index() + 1);
            self.match_index.insert(peer.id.clone(), 0);
        }
        
        // 发送初始心跳
        self.send_heartbeats().await?;
        
        // 启动心跳定时器
        self.start_heartbeat_timer().await?;
        
        Ok(())
    }
}
```

## ⚡ 性能优化

### 选举超时优化

```rust
pub struct AdaptiveElectionTimeout {
    base_timeout: Duration,
    network_latency: Duration,
    failure_rate: f64,
}

impl AdaptiveElectionTimeout {
    pub fn calculate_timeout(&self) -> Duration {
        // 根据网络延迟和故障率调整超时时间
        let latency_factor = 1.0 + (self.network_latency.as_millis() as f64 / 100.0);
        let failure_factor = 1.0 + self.failure_rate;
        
        let adjusted_timeout = self.base_timeout.as_millis() as f64 
            * latency_factor * failure_factor;
        
        Duration::from_millis(adjusted_timeout as u64)
    }
}
```

### 预投票机制

```rust
#[derive(Debug, Clone)]
pub struct PreVoteRequest {
    pub term: u64,
    pub candidate_id: String,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

impl RaftNode {
    pub async fn start_pre_vote(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let pre_vote_term = self.current_term + 1;
        let mut pre_votes = 1; // 自己的一票
        
        for peer in &self.peers {
            let request = PreVoteRequest {
                term: pre_vote_term,
                candidate_id: self.node_id.clone(),
                last_log_index: self.last_log_index(),
                last_log_term: self.last_log_term(),
            };
            
            // 发送预投票请求
            if let Ok(response) = self.send_pre_vote(peer.id.clone(), request).await {
                if response.vote_granted {
                    pre_votes += 1;
                }
            }
        }
        
        Ok(pre_votes >= self.majority_count())
    }
}
```

## 🧪 测试策略

### 选举测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_leader_election() {
        let mut cluster = create_raft_cluster(5).await;
        
        // 1. 初始状态检查
        assert!(cluster.has_leader().await);
        assert_eq!(cluster.leader_count().await, 1);
        
        // 2. 杀死领导者
        let leader_id = cluster.get_leader().await.unwrap();
        cluster.kill_node(leader_id).await;
        
        // 3. 等待新领导者选出
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // 4. 验证新领导者
        assert!(cluster.has_leader().await);
        assert_eq!(cluster.leader_count().await, 1);
        
        let new_leader_id = cluster.get_leader().await.unwrap();
        assert_ne!(new_leader_id, leader_id);
    }
    
    #[tokio::test]
    async fn test_election_timeout() {
        let mut node = RaftNode::new("node1".to_string()).await;
        let start_time = Instant::now();
        
        // 启动选举
        node.start_election().await.unwrap();
        
        // 验证选举超时
        let elapsed = start_time.elapsed();
        assert!(elapsed >= Duration::from_millis(150));
        assert!(elapsed <= Duration::from_millis(300));
    }
    
    #[tokio::test]
    async fn test_split_vote() {
        let mut cluster = create_raft_cluster(3).await;
        
        // 创建网络分区
        cluster.partition(vec![0], vec![1, 2]).await;
        
        // 等待选举超时
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // 验证没有领导者（因为分区导致无法获得多数派）
        assert!(!cluster.has_leader().await);
        
        // 恢复网络
        cluster.heal_partition().await;
        
        // 等待新领导者选出
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // 验证有领导者
        assert!(cluster.has_leader().await);
    }
}
```

## 🔍 故障排查

### 常见问题

#### 1. 选举超时过长

**症状**: 领导者故障后很长时间没有新领导者选出
**原因**: 选举超时配置不合理
**解决方案**:

```rust
let config = ElectionConfig {
    election_timeout_min: Duration::from_millis(150),
    election_timeout_max: Duration::from_millis(300),
    heartbeat_interval: Duration::from_millis(50),
    max_election_timeout: Duration::from_millis(1000),
};
```

#### 2. 频繁选举

**症状**: 系统频繁进行选举
**原因**: 心跳间隔过长或网络不稳定
**解决方案**:

```rust
let config = ElectionConfig {
    election_timeout_min: Duration::from_millis(150),
    election_timeout_max: Duration::from_millis(300),
    heartbeat_interval: Duration::from_millis(50), // 减少心跳间隔
    max_election_timeout: Duration::from_millis(1000),
};
```

#### 3. 双领导者

**症状**: 系统中同时存在多个领导者
**原因**: 网络分区或时钟不同步
**解决方案**:

- 检查网络分区情况
- 确保时钟同步
- 增加选举超时的随机性

## 📚 进一步阅读

- [Raft 论文](https://raft.github.io/raft.pdf) - Raft 共识算法详细说明
- [共识机制总览](./README.md) - 共识算法概述
- [日志复制](./log_replication.md) - 日志复制机制
- [故障处理](../failure/README.md) - 故障检测和处理

## 🔗 相关文档

- [共识算法](./README.md)
- [日志复制](./log_replication.md)
- [故障处理](../failure/README.md)
- [时间模型](../time/README.md)
- [网络传输](../transport/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

# 日志复制机制

> 分布式系统中的日志复制、同步和冲突解决机制

## 目录

- [日志复制机制](#日志复制机制)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [日志结构](#日志结构)
    - [日志不变量](#日志不变量)
  - [🔧 日志复制实现](#-日志复制实现)
    - [追加条目请求](#追加条目请求)
    - [领导者日志复制](#领导者日志复制)
  - [🔄 冲突解决机制](#-冲突解决机制)
    - [日志冲突检测](#日志冲突检测)
    - [快速回退优化](#快速回退优化)
  - [📊 性能优化](#-性能优化)
    - [批量复制](#批量复制)
    - [流水线复制](#流水线复制)
  - [🧪 测试策略](#-测试策略)
    - [日志复制测试](#日志复制测试)
  - [🔍 故障排查](#-故障排查)
    - [常见问题](#常见问题)
      - [1. 日志复制卡住](#1-日志复制卡住)
      - [2. 日志冲突频繁](#2-日志冲突频繁)
      - [3. 提交索引不推进](#3-提交索引不推进)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

日志复制是分布式共识算法的核心机制，确保所有节点上的日志保持一致，并在出现冲突时能够正确解决。

## 🎯 核心概念

### 日志结构

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Log {
    entries: Vec<LogEntry>,
    commit_index: u64,
    last_applied: u64,
}
```

### 日志不变量

- **前缀匹配**: 如果两个日志在相同索引处有相同任期，则它们在该索引之前的所有条目都相同
- **提交单调**: 一旦某个索引的日志条目被提交，所有更高索引的条目都不会被覆盖
- **应用单调**: 应用索引只能单调递增

## 🔧 日志复制实现

### 追加条目请求

```rust
#[derive(Debug, Clone)]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub leader_id: String,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: u64,
}

#[derive(Debug, Clone)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
    pub next_index: u64,
    pub match_index: u64,
}

impl RaftNode {
    pub fn handle_append_entries(
        &mut self,
        request: AppendEntriesRequest,
    ) -> AppendEntriesResponse {
        // 1. 检查任期
        if request.term > self.current_term {
            self.current_term = request.term;
            self.voted_for = None;
            self.state = RaftState::Follower;
        }
        
        // 2. 检查日志匹配
        let log_match = self.check_log_match(
            request.prev_log_index,
            request.prev_log_term,
        );
        
        if !log_match {
            return AppendEntriesResponse {
                term: self.current_term,
                success: false,
                next_index: self.find_next_index(request.prev_log_index),
                match_index: 0,
            };
        }
        
        // 3. 处理日志条目
        self.process_log_entries(&request.entries, request.prev_log_index);
        
        // 4. 更新提交索引
        if request.leader_commit > self.commit_index {
            self.commit_index = std::cmp::min(
                request.leader_commit,
                self.last_log_index(),
            );
        }
        
        // 5. 应用已提交的条目
        self.apply_committed_entries();
        
        AppendEntriesResponse {
            term: self.current_term,
            success: true,
            next_index: self.last_log_index() + 1,
            match_index: self.last_log_index(),
        }
    }
    
    fn check_log_match(&self, prev_log_index: u64, prev_log_term: u64) -> bool {
        if prev_log_index == 0 {
            return true; // 没有前一个条目
        }
        
        if prev_log_index > self.last_log_index() {
            return false; // 索引超出范围
        }
        
        let entry = &self.log.entries[prev_log_index as usize - 1];
        entry.term == prev_log_term
    }
    
    fn process_log_entries(&mut self, entries: &[LogEntry], prev_log_index: u64) {
        for (i, entry) in entries.iter().enumerate() {
            let index = prev_log_index + 1 + i as u64;
            
            if index <= self.last_log_index() {
                // 检查是否冲突
                let existing_entry = &self.log.entries[index as usize - 1];
                if existing_entry.term != entry.term {
                    // 冲突，截断日志
                    self.log.entries.truncate(index as usize - 1);
                    break;
                }
            } else {
                // 追加新条目
                self.log.entries.push(entry.clone());
            }
        }
    }
}
```

### 领导者日志复制

```rust
impl RaftNode {
    pub async fn replicate_log(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.state != RaftState::Leader {
            return Ok(());
        }
        
        for peer in &self.peers {
            self.replicate_to_peer(peer.id.clone()).await?;
        }
        
        Ok(())
    }
    
    async fn replicate_to_peer(&mut self, peer_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let next_index = *self.next_index.get(&peer_id).unwrap_or(&1);
        let prev_log_index = next_index - 1;
        let prev_log_term = if prev_log_index == 0 {
            0
        } else {
            self.log.entries[prev_log_index as usize - 1].term
        };
        
        let entries = if next_index <= self.last_log_index() {
            self.log.entries[next_index as usize - 1..].to_vec()
        } else {
            Vec::new()
        };
        
        let request = AppendEntriesRequest {
            term: self.current_term,
            leader_id: self.node_id.clone(),
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit: self.commit_index,
        };
        
        match self.send_append_entries(peer_id.clone(), request).await {
            Ok(response) => {
                if response.success {
                    // 更新匹配索引
                    self.match_index.insert(peer_id.clone(), response.match_index);
                    self.next_index.insert(peer_id.clone(), response.next_index);
                    
                    // 尝试推进提交索引
                    self.try_advance_commit_index();
                } else {
                    // 日志不匹配，回退
                    if response.term > self.current_term {
                        self.current_term = response.term;
                        self.state = RaftState::Follower;
                        self.voted_for = None;
                    } else {
                        // 减少 next_index 并重试
                        let new_next_index = std::cmp::max(1, next_index - 1);
                        self.next_index.insert(peer_id.clone(), new_next_index);
                    }
                }
            }
            Err(_) => {
                // 网络错误，稍后重试
            }
        }
        
        Ok(())
    }
    
    fn try_advance_commit_index(&mut self) {
        // 找到多数派已复制的最大索引
        let mut match_indices: Vec<u64> = self.match_index.values().cloned().collect();
        match_indices.push(self.last_log_index()); // 包括领导者自己
        match_indices.sort();
        
        let majority_index = match_indices[match_indices.len() / 2];
        
        // 检查该索引的条目是否在当前任期内
        if majority_index > self.commit_index {
            let entry = &self.log.entries[majority_index as usize - 1];
            if entry.term == self.current_term {
                self.commit_index = majority_index;
                self.apply_committed_entries();
            }
        }
    }
}
```

## 🔄 冲突解决机制

### 日志冲突检测

```rust
impl RaftNode {
    fn detect_log_conflict(&self, entries: &[LogEntry], start_index: u64) -> Option<u64> {
        for (i, entry) in entries.iter().enumerate() {
            let index = start_index + i as u64;
            
            if index <= self.last_log_index() {
                let existing_entry = &self.log.entries[index as usize - 1];
                if existing_entry.term != entry.term {
                    return Some(index);
                }
            }
        }
        
        None
    }
    
    fn resolve_log_conflict(&mut self, conflict_index: u64) {
        // 截断冲突点之后的所有条目
        self.log.entries.truncate(conflict_index as usize - 1);
        
        // 更新相关索引
        if self.commit_index >= conflict_index {
            self.commit_index = conflict_index - 1;
        }
        
        if self.last_applied >= conflict_index {
            self.last_applied = conflict_index - 1;
        }
    }
}
```

### 快速回退优化

```rust
impl RaftNode {
    fn find_next_index(&self, prev_log_index: u64) -> u64 {
        // 快速回退算法
        for i in (1..=prev_log_index).rev() {
            let entry = &self.log.entries[i as usize - 1];
            if entry.term == self.current_term {
                return i + 1;
            }
        }
        
        1
    }
    
    fn binary_search_next_index(&self, prev_log_index: u64, prev_log_term: u64) -> u64 {
        // 二分搜索优化
        let mut left = 1;
        let mut right = prev_log_index;
        
        while left <= right {
            let mid = (left + right) / 2;
            let entry = &self.log.entries[mid as usize - 1];
            
            if entry.term <= prev_log_term {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }
        
        left
    }
}
```

## 📊 性能优化

### 批量复制

```rust
impl RaftNode {
    pub async fn batch_replicate(&mut self, commands: Vec<Vec<u8>>) -> Result<u64, Box<dyn std::error::Error>> {
        if self.state != RaftState::Leader {
            return Err("Not leader".into());
        }
        
        let start_index = self.last_log_index() + 1;
        let term = self.current_term;
        
        // 创建日志条目
        let entries: Vec<LogEntry> = commands
            .into_iter()
            .enumerate()
            .map(|(i, command)| LogEntry {
                term,
                index: start_index + i as u64,
                command,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            })
            .collect();
        
        // 追加到本地日志
        self.log.entries.extend(entries.clone());
        
        // 并行复制到所有对等节点
        let mut tasks = Vec::new();
        for peer in &self.peers {
            let peer_id = peer.id.clone();
            let entries_clone = entries.clone();
            let task = tokio::spawn(async move {
                self.replicate_entries_to_peer(peer_id, entries_clone).await
            });
            tasks.push(task);
        }
        
        // 等待复制完成
        let mut success_count = 1; // 包括领导者自己
        for task in tasks {
            if let Ok(Ok(true)) = task.await {
                success_count += 1;
            }
        }
        
        // 检查是否达到多数派
        if success_count >= self.majority_count() {
            // 推进提交索引
            let new_commit_index = start_index + entries.len() as u64 - 1;
            self.commit_index = new_commit_index;
            self.apply_committed_entries();
            Ok(new_commit_index)
        } else {
            Err("Failed to replicate to majority".into())
        }
    }
}
```

### 流水线复制

```rust
pub struct PipelineReplicator {
    pipeline_size: usize,
    pending_requests: HashMap<String, Vec<PendingRequest>>,
}

#[derive(Debug, Clone)]
struct PendingRequest {
    request: AppendEntriesRequest,
    callback: oneshot::Sender<AppendEntriesResponse>,
}

impl PipelineReplicator {
    pub async fn pipeline_replicate(
        &mut self,
        peer_id: String,
        request: AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse, Box<dyn std::error::Error>> {
        let (tx, rx) = oneshot::channel();
        
        let pending_request = PendingRequest {
            request,
            callback: tx,
        };
        
        // 添加到流水线
        self.pending_requests
            .entry(peer_id.clone())
            .or_insert_with(Vec::new)
            .push(pending_request);
        
        // 检查流水线大小
        if self.pending_requests[&peer_id].len() >= self.pipeline_size {
            self.flush_pipeline(peer_id.clone()).await?;
        }
        
        // 等待响应
        match rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err("Pipeline request failed".into()),
        }
    }
    
    async fn flush_pipeline(&mut self, peer_id: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(requests) = self.pending_requests.remove(&peer_id) {
            for request in requests {
                // 发送请求
                let response = self.send_append_entries(peer_id.clone(), request.request).await?;
                
                // 发送响应
                let _ = request.callback.send(response);
            }
        }
        
        Ok(())
    }
}
```

## 🧪 测试策略

### 日志复制测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_log_replication() {
        let mut cluster = create_raft_cluster(3).await;
        
        // 1. 提交日志条目
        let entry = LogEntry {
            term: 1,
            index: 1,
            command: b"test_command".to_vec(),
            timestamp: 0,
        };
        
        let leader = cluster.get_leader().await.unwrap();
        cluster.propose_entry(leader, entry.clone()).await.unwrap();
        
        // 2. 等待复制
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 3. 验证所有节点都有该条目
        for node in cluster.nodes() {
            let log = node.get_log().await;
            assert!(log.entries.contains(&entry));
        }
    }
    
    #[tokio::test]
    async fn test_log_conflict_resolution() {
        let mut cluster = create_raft_cluster(3).await;
        
        // 1. 创建网络分区
        cluster.partition(vec![0], vec![1, 2]).await;
        
        // 2. 在两个分区中提交不同的条目
        let entry1 = LogEntry {
            term: 2,
            index: 1,
            command: b"command1".to_vec(),
            timestamp: 0,
        };
        
        let entry2 = LogEntry {
            term: 2,
            index: 1,
            command: b"command2".to_vec(),
            timestamp: 0,
        };
        
        cluster.propose_entry(0, entry1).await.unwrap();
        cluster.propose_entry(1, entry2).await.unwrap();
        
        // 3. 恢复网络
        cluster.heal_partition().await;
        
        // 4. 等待冲突解决
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // 5. 验证最终一致性
        let final_entry = cluster.get_committed_entry(1).await.unwrap();
        assert!(final_entry.command == b"command1" || final_entry.command == b"command2");
    }
    
    #[tokio::test]
    async fn test_commit_index_advancement() {
        let mut cluster = create_raft_cluster(5).await;
        
        // 1. 提交多个条目
        for i in 1..=10 {
            let entry = LogEntry {
                term: 1,
                index: i,
                command: format!("command_{}", i).into_bytes(),
                timestamp: 0,
            };
            
            let leader = cluster.get_leader().await.unwrap();
            cluster.propose_entry(leader, entry).await.unwrap();
        }
        
        // 2. 等待复制和提交
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // 3. 验证提交索引
        for node in cluster.nodes() {
            let commit_index = node.get_commit_index().await;
            assert!(commit_index >= 10);
        }
    }
}
```

## 🔍 故障排查

### 常见问题

#### 1. 日志复制卡住

**症状**: 日志条目无法复制到多数派节点
**原因**: 网络分区或节点故障
**解决方案**:

```rust
// 增加重试机制
impl RaftNode {
    async fn replicate_with_retry(&mut self, peer_id: String, max_retries: usize) -> Result<(), Box<dyn std::error::Error>> {
        for attempt in 0..max_retries {
            match self.replicate_to_peer(peer_id.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt == max_retries - 1 {
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                }
            }
        }
        Ok(())
    }
}
```

#### 2. 日志冲突频繁

**症状**: 频繁出现日志冲突和回退
**原因**: 网络不稳定或时钟不同步
**解决方案**:

- 优化网络配置
- 确保时钟同步
- 增加心跳频率

#### 3. 提交索引不推进

**症状**: 日志条目已复制但未提交
**原因**: 无法获得多数派确认
**解决方案**:

```rust
// 检查多数派状态
impl RaftNode {
    fn check_majority_health(&self) -> bool {
        let healthy_peers = self.peers.iter()
            .filter(|peer| self.is_peer_healthy(peer.id.clone()))
            .count();
        
        healthy_peers >= self.majority_count() - 1 // 不包括领导者自己
    }
}
```

## 📚 进一步阅读

- [Raft 论文](https://raft.github.io/raft.pdf) - Raft 共识算法详细说明
- [共识机制总览](./README.md) - 共识算法概述
- [领导者选举](./leader_election.md) - 领导者选举机制
- [故障处理](../failure/README.md) - 故障检测和处理

## 🔗 相关文档

- [共识算法](./README.md)
- [领导者选举](./leader_election.md)
- [故障处理](../failure/README.md)
- [存储抽象](../storage/README.md)
- [网络传输](../transport/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

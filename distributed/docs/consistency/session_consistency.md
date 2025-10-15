# 会话一致性（Session Consistency）

> 分布式系统中的会话保证和一致性实现

## 目录

- [会话一致性（Session Consistency）](#会话一致性session-consistency)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [会话保证（Session Guarantees）](#会话保证session-guarantees)
    - [会话一致性定义](#会话一致性定义)
  - [🔧 实现机制](#-实现机制)
    - [会话向量（Session Vector）](#会话向量session-vector)
    - [会话令牌（Session Token）](#会话令牌session-token)
    - [会话一致性存储](#会话一致性存储)
  - [🚀 高级特性](#-高级特性)
    - [会话合并（Session Merging）](#会话合并session-merging)
    - [会话过期（Session Expiration）](#会话过期session-expiration)
  - [🧪 测试策略](#-测试策略)
    - [会话一致性测试](#会话一致性测试)
  - [🔍 性能优化](#-性能优化)
    - [会话缓存](#会话缓存)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

会话一致性是分布式系统中一种重要的弱一致性模型，它保证在同一个客户端会话内，系统能够提供特定的读写语义保证。
会话一致性是介于因果一致性和最终一致性之间的一致性级别。

## 🎯 核心概念

### 会话保证（Session Guarantees）

会话一致性提供以下四种基本保证：

1. **读己之写（Read Your Writes, RYW）**: 客户端能够读取到自己写入的数据
2. **单调读（Monotonic Reads, MR）**: 客户端不会读到比之前更旧的数据
3. **单调写（Monotonic Writes, MW）**: 客户端的写操作按顺序执行
4. **写后读（Writes Follow Reads, WFR）**: 客户端写入的数据能够反映之前读取到的数据

### 会话一致性定义

**定义 1（会话一致性）**: 对于任意客户端会话 S，如果操作序列 O₁, O₂, ..., Oₙ 在会话 S 中执行，则存在一个全局操作序列 G，使得：

- G 包含所有操作 O₁, O₂, ..., Oₙ
- G 保持会话内的操作顺序
- G 满足会话保证（RYW, MR, MW, WFR）

## 🔧 实现机制

### 会话向量（Session Vector）

```rust
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionVector {
    session_id: String,
    client_id: String,
    last_write_vector: HashMap<String, u64>,
    last_read_vector: HashMap<String, u64>,
    write_sequence: u64,
    read_sequence: u64,
}

impl SessionVector {
    pub fn new(session_id: String, client_id: String) -> Self {
        Self {
            session_id,
            client_id,
            last_write_vector: HashMap::new(),
            last_read_vector: HashMap::new(),
            write_sequence: 0,
            read_sequence: 0,
        }
    }
    
    // 记录写操作
    pub fn record_write(&mut self, node_id: String, sequence: u64) {
        self.last_write_vector.insert(node_id, sequence);
        self.write_sequence += 1;
    }
    
    // 记录读操作
    pub fn record_read(&mut self, node_id: String, sequence: u64) {
        self.last_read_vector.insert(node_id, sequence);
        self.read_sequence += 1;
    }
    
    // 检查读己之写保证
    pub fn can_read_own_writes(&self, node_id: &str) -> bool {
        if let Some(&last_write) = self.last_write_vector.get(node_id) {
            if let Some(&last_read) = self.last_read_vector.get(node_id) {
                return last_read >= last_write;
            }
        }
        true
    }
    
    // 检查单调读保证
    pub fn can_monotonic_read(&self, node_id: &str, current_sequence: u64) -> bool {
        if let Some(&last_read) = self.last_read_vector.get(node_id) {
            return current_sequence >= last_read;
        }
        true
    }
}
```

### 会话令牌（Session Token）

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    session_id: String,
    client_id: String,
    dependencies: HashMap<String, u64>,
    timestamp: u64,
}

impl SessionToken {
    pub fn new(session_id: String, client_id: String) -> Self {
        Self {
            session_id,
            client_id,
            dependencies: HashMap::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
    
    // 添加依赖
    pub fn add_dependency(&mut self, node_id: String, sequence: u64) {
        self.dependencies.insert(node_id, sequence);
    }
    
    // 检查依赖是否满足
    pub fn dependencies_satisfied(&self, node_sequences: &HashMap<String, u64>) -> bool {
        for (node_id, &required_sequence) in &self.dependencies {
            if let Some(&current_sequence) = node_sequences.get(node_id) {
                if current_sequence < required_sequence {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}
```

### 会话一致性存储

```rust
use std::sync::{Arc, RwLock};

pub struct SessionConsistentStore {
    data: Arc<RwLock<HashMap<String, Vec<DataEntry>>>>,
    sessions: Arc<RwLock<HashMap<String, SessionVector>>>,
    node_id: String,
}

#[derive(Debug, Clone)]
pub struct DataEntry {
    pub key: String,
    pub value: String,
    pub sequence: u64,
    pub timestamp: u64,
    pub session_id: Option<String>,
}

impl SessionConsistentStore {
    pub fn new(node_id: String) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            node_id,
        }
    }
    
    // 写入数据（会话一致性）
    pub fn write(&self, key: String, value: String, session_token: &SessionToken) 
        -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write().unwrap();
        let mut sessions = self.sessions.write().unwrap();
        
        // 获取或创建会话向量
        let session_vector = sessions
            .entry(session_token.session_id.clone())
            .or_insert_with(|| SessionVector::new(
                session_token.session_id.clone(),
                session_token.client_id.clone()
            ));
        
        // 生成序列号
        let sequence = self.generate_sequence();
        
        // 创建数据条目
        let entry = DataEntry {
            key: key.clone(),
            value,
            sequence,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis() as u64,
            session_id: Some(session_token.session_id.clone()),
        };
        
        // 存储数据
        data.entry(key).or_insert_with(Vec::new).push(entry);
        
        // 更新会话向量
        session_vector.record_write(self.node_id.clone(), sequence);
        
        Ok(())
    }
    
    // 读取数据（会话一致性）
    pub fn read(&self, key: String, session_token: &SessionToken) 
        -> Result<Option<String>, Box<dyn std::error::Error>> {
        let data = self.data.read().unwrap();
        let mut sessions = self.sessions.write().unwrap();
        
        // 获取或创建会话向量
        let session_vector = sessions
            .entry(session_token.session_id.clone())
            .or_insert_with(|| SessionVector::new(
                session_token.session_id.clone(),
                session_token.client_id.clone()
            ));
        
        if let Some(entries) = data.get(&key) {
            // 找到满足会话保证的最新条目
            for entry in entries.iter().rev() {
                // 检查读己之写保证
                if !session_vector.can_read_own_writes(&self.node_id) {
                    continue;
                }
                
                // 检查单调读保证
                if !session_vector.can_monotonic_read(&self.node_id, entry.sequence) {
                    continue;
                }
                
                // 检查写后读保证
                if !self.satisfies_writes_follow_reads(entry, session_token) {
                    continue;
                }
                
                // 更新会话向量
                session_vector.record_read(self.node_id.clone(), entry.sequence);
                
                return Ok(Some(entry.value.clone()));
            }
        }
        
        Ok(None)
    }
    
    // 检查写后读保证
    fn satisfies_writes_follow_reads(&self, entry: &DataEntry, session_token: &SessionToken) -> bool {
        // 检查依赖是否满足
        session_token.dependencies_satisfied(&HashMap::new()) // 简化实现
    }
    
    // 生成序列号
    fn generate_sequence(&self) -> u64 {
        // 简化实现，实际应该使用更复杂的序列号生成机制
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
```

## 🚀 高级特性

### 会话合并（Session Merging）

```rust
impl SessionVector {
    // 合并两个会话向量
    pub fn merge(&mut self, other: &SessionVector) {
        // 合并写向量
        for (node_id, &sequence) in &other.last_write_vector {
            let current = self.last_write_vector.entry(node_id.clone()).or_insert(0);
            *current = (*current).max(sequence);
        }
        
        // 合并读向量
        for (node_id, &sequence) in &other.last_read_vector {
            let current = self.last_read_vector.entry(node_id.clone()).or_insert(0);
            *current = (*current).max(sequence);
        }
        
        // 更新序列号
        self.write_sequence = self.write_sequence.max(other.write_sequence);
        self.read_sequence = self.read_sequence.max(other.read_sequence);
    }
}
```

### 会话过期（Session Expiration）

```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, (SessionVector, u64)>>>,
    session_timeout: u64,
}

impl SessionManager {
    pub fn new(session_timeout: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout,
        }
    }
    
    // 清理过期会话
    pub fn cleanup_expired_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().unwrap();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        sessions.retain(|_, (_, last_access)| {
            current_time - *last_access < self.session_timeout
        });
        
        Ok(())
    }
    
    // 更新会话访问时间
    pub fn update_session_access(&self, session_id: &str) {
        let mut sessions = self.sessions.write().unwrap();
        if let Some((_, last_access)) = sessions.get_mut(session_id) {
            *last_access = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }
    }
}
```

## 🧪 测试策略

### 会话一致性测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_your_writes() {
        let store = SessionConsistentStore::new("node1".to_string());
        let session_token = SessionToken::new("session1".to_string(), "client1".to_string());
        
        // 写入数据
        store.write("key1".to_string(), "value1".to_string(), &session_token).unwrap();
        
        // 读取数据
        let value = store.read("key1".to_string(), &session_token).unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }
    
    #[test]
    fn test_monotonic_reads() {
        let store = SessionConsistentStore::new("node1".to_string());
        let session_token = SessionToken::new("session1".to_string(), "client1".to_string());
        
        // 写入多个版本
        store.write("key1".to_string(), "value1".to_string(), &session_token).unwrap();
        store.write("key1".to_string(), "value2".to_string(), &session_token).unwrap();
        
        // 第一次读取
        let value1 = store.read("key1".to_string(), &session_token).unwrap();
        
        // 第二次读取应该不会读到更旧的值
        let value2 = store.read("key1".to_string(), &session_token).unwrap();
        assert!(value2 >= value1);
    }
    
    #[test]
    fn test_monotonic_writes() {
        let store = SessionConsistentStore::new("node1".to_string());
        let session_token = SessionToken::new("session1".to_string(), "client1".to_string());
        
        // 连续写入
        store.write("key1".to_string(), "value1".to_string(), &session_token).unwrap();
        store.write("key1".to_string(), "value2".to_string(), &session_token).unwrap();
        store.write("key1".to_string(), "value3".to_string(), &session_token).unwrap();
        
        // 读取应该得到最新值
        let value = store.read("key1".to_string(), &session_token).unwrap();
        assert_eq!(value, Some("value3".to_string()));
    }
    
    #[test]
    fn test_writes_follow_reads() {
        let store = SessionConsistentStore::new("node1".to_string());
        let session_token = SessionToken::new("session1".to_string(), "client1".to_string());
        
        // 先读取
        let _ = store.read("key1".to_string(), &session_token).unwrap();
        
        // 后写入
        store.write("key1".to_string(), "value1".to_string(), &session_token).unwrap();
        
        // 再次读取应该能看到写入的值
        let value = store.read("key1".to_string(), &session_token).unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }
}
```

## 🔍 性能优化

### 会话缓存

```rust
pub struct SessionCache {
    cache: Arc<RwLock<HashMap<String, CachedSession>>>,
    max_size: usize,
}

#[derive(Debug, Clone)]
pub struct CachedSession {
    session_vector: SessionVector,
    last_access: u64,
    access_count: u64,
}

impl SessionCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }
    
    // 获取会话
    pub fn get_session(&self, session_id: &str) -> Option<SessionVector> {
        let mut cache = self.cache.write().unwrap();
        if let Some(cached) = cache.get_mut(session_id) {
            cached.last_access = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            cached.access_count += 1;
            Some(cached.session_vector.clone())
        } else {
            None
        }
    }
    
    // 存储会话
    pub fn put_session(&self, session_id: String, session_vector: SessionVector) {
        let mut cache = self.cache.write().unwrap();
        
        // 检查缓存大小
        if cache.len() >= self.max_size {
            self.evict_least_recently_used(&mut cache);
        }
        
        let cached = CachedSession {
            session_vector,
            last_access: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            access_count: 1,
        };
        
        cache.insert(session_id, cached);
    }
    
    // LRU 淘汰
    fn evict_least_recently_used(&self, cache: &mut HashMap<String, CachedSession>) {
        if let Some((oldest_key, _)) = cache.iter().min_by_key(|(_, cached)| cached.last_access) {
            cache.remove(oldest_key);
        }
    }
}
```

## 📚 进一步阅读

- [一致性模型概述](./README.md) - 一致性模型总览
- [CAP/PACELC 定理](./cap_pacelc.md) - 一致性、可用性、分区容错权衡
- [向量时钟理论](./vector_clocks.md) - 因果依赖跟踪
- [故障处理](../failure/README.md) - 故障检测和处理

## 🔗 相关文档

- [一致性模型](./README.md)
- [CAP/PACELC](./cap_pacelc.md)
- [向量时钟](./vector_clocks.md)
- [故障处理](../failure/README.md)
- [共识机制](../consensus/README.md)
- [复制策略](../replication/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

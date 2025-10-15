# 单调一致性（Monotonic Consistency）

> 分布式系统中的单调读写保证和实现机制

## 目录

- [单调一致性（Monotonic Consistency）](#单调一致性monotonic-consistency)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [单调读（Monotonic Reads）](#单调读monotonic-reads)
    - [单调写（Monotonic Writes）](#单调写monotonic-writes)
    - [单调一致性定义](#单调一致性定义)
  - [🔧 实现机制](#-实现机制)
    - [单调读实现](#单调读实现)
    - [单调写实现](#单调写实现)
    - [单调一致性存储](#单调一致性存储)
  - [🚀 高级特性](#-高级特性)
    - [版本向量（Version Vector）](#版本向量version-vector)
    - [单调性检查器](#单调性检查器)
  - [🧪 测试策略](#-测试策略)
    - [单调性测试](#单调性测试)
  - [🔍 性能优化](#-性能优化)
    - [单调性缓存](#单调性缓存)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

单调一致性是分布式系统中一种重要的弱一致性模型，它保证客户端在访问分布式系统时能够获得单调的读写体验。
单调一致性包括单调读和单调写两种保证，确保客户端不会观察到数据的历史倒退。

## 🎯 核心概念

### 单调读（Monotonic Reads）

**定义 1（单调读）**: 对于任意客户端 C，如果客户端 C 在时间 t₁ 读取到数据版本 v₁，在时间 t₂ 读取到数据版本 v₂，且 t₁ < t₂，则 v₁ ≤ v₂。

单调读保证客户端不会读到比之前更旧的数据版本，确保数据访问的单调性。

### 单调写（Monotonic Writes）

**定义 2（单调写）**: 对于任意客户端 C，如果客户端 C 执行写操作序列 W₁, W₂, ..., Wₙ，则这些写操作在所有副本上的执行顺序必须保持一致。

单调写保证客户端的写操作按照提交顺序在所有副本上执行，避免写操作的乱序执行。

### 单调一致性定义

**定义 3（单调一致性）**: 分布式系统满足单调一致性，当且仅当系统同时满足单调读和单调写保证。

## 🔧 实现机制

### 单调读实现

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonotonicReadTracker {
    client_id: String,
    last_read_versions: HashMap<String, u64>,
    last_read_timestamps: HashMap<String, u64>,
}

impl MonotonicReadTracker {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            last_read_versions: HashMap::new(),
            last_read_timestamps: HashMap::new(),
        }
    }
    
    // 记录读操作
    pub fn record_read(&mut self, key: &str, version: u64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        self.last_read_versions.insert(key.to_string(), version);
        self.last_read_timestamps.insert(key.to_string(), timestamp);
    }
    
    // 检查单调读保证
    pub fn can_read_version(&self, key: &str, version: u64) -> bool {
        if let Some(&last_version) = self.last_read_versions.get(key) {
            return version >= last_version;
        }
        true // 首次读取，允许任何版本
    }
    
    // 获取最后读取的版本
    pub fn get_last_read_version(&self, key: &str) -> Option<u64> {
        self.last_read_versions.get(key).copied()
    }
}
```

### 单调写实现

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonotonicWriteTracker {
    client_id: String,
    write_sequence: u64,
    pending_writes: HashMap<String, u64>,
    committed_writes: HashMap<String, u64>,
}

impl MonotonicWriteTracker {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            write_sequence: 0,
            pending_writes: HashMap::new(),
            committed_writes: HashMap::new(),
        }
    }
    
    // 开始写操作
    pub fn begin_write(&mut self, key: &str) -> u64 {
        self.write_sequence += 1;
        self.pending_writes.insert(key.to_string(), self.write_sequence);
        self.write_sequence
    }
    
    // 提交写操作
    pub fn commit_write(&mut self, key: &str, sequence: u64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(&pending_sequence) = self.pending_writes.get(key) {
            if pending_sequence == sequence {
                self.committed_writes.insert(key.to_string(), sequence);
                self.pending_writes.remove(key);
                Ok(())
            } else {
                Err("Write sequence mismatch".into())
            }
        } else {
            Err("No pending write found".into())
        }
    }
    
    // 检查单调写保证
    pub fn can_write(&self, key: &str, sequence: u64) -> bool {
        if let Some(&last_committed) = self.committed_writes.get(key) {
            return sequence > last_committed;
        }
        true // 首次写入，允许任何序列号
    }
    
    // 获取最后提交的写序列号
    pub fn get_last_committed_sequence(&self, key: &str) -> Option<u64> {
        self.committed_writes.get(key).copied()
    }
}
```

### 单调一致性存储

```rust
pub struct MonotonicConsistentStore {
    data: Arc<RwLock<HashMap<String, Vec<DataVersion>>>>,
    read_trackers: Arc<RwLock<HashMap<String, MonotonicReadTracker>>>,
    write_trackers: Arc<RwLock<HashMap<String, MonotonicWriteTracker>>>,
    node_id: String,
}

#[derive(Debug, Clone)]
pub struct DataVersion {
    pub key: String,
    pub value: String,
    pub version: u64,
    pub timestamp: u64,
    pub client_id: String,
    pub write_sequence: u64,
}

impl MonotonicConsistentStore {
    pub fn new(node_id: String) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            read_trackers: Arc::new(RwLock::new(HashMap::new())),
            write_trackers: Arc::new(RwLock::new(HashMap::new())),
            node_id,
        }
    }
    
    // 写入数据（单调一致性）
    pub fn write(&self, key: String, value: String, client_id: String) 
        -> Result<u64, Box<dyn std::error::Error>> {
        let mut data = self.data.write().unwrap();
        let mut write_trackers = self.write_trackers.write().unwrap();
        
        // 获取或创建写跟踪器
        let write_tracker = write_trackers
            .entry(client_id.clone())
            .or_insert_with(|| MonotonicWriteTracker::new(client_id.clone()));
        
        // 开始写操作
        let sequence = write_tracker.begin_write(&key);
        
        // 检查单调写保证
        if !write_tracker.can_write(&key, sequence) {
            return Err("Monotonic write violation".into());
        }
        
        // 创建数据版本
        let version = self.generate_version();
        let data_version = DataVersion {
            key: key.clone(),
            value,
            version,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis() as u64,
            client_id: client_id.clone(),
            write_sequence: sequence,
        };
        
        // 存储数据版本
        data.entry(key).or_insert_with(Vec::new).push(data_version);
        
        // 提交写操作
        write_tracker.commit_write(&key, sequence)?;
        
        Ok(sequence)
    }
    
    // 读取数据（单调一致性）
    pub fn read(&self, key: String, client_id: String) 
        -> Result<Option<String>, Box<dyn std::error::Error>> {
        let data = self.data.read().unwrap();
        let mut read_trackers = self.read_trackers.write().unwrap();
        
        // 获取或创建读跟踪器
        let read_tracker = read_trackers
            .entry(client_id.clone())
            .or_insert_with(|| MonotonicReadTracker::new(client_id.clone()));
        
        if let Some(versions) = data.get(&key) {
            // 找到满足单调读保证的最新版本
            for version in versions.iter().rev() {
                if read_tracker.can_read_version(&key, version.version) {
                    // 记录读操作
                    read_tracker.record_read(&key, version.version);
                    
                    return Ok(Some(version.value.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    // 生成版本号
    fn generate_version(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
```

## 🚀 高级特性

### 版本向量（Version Vector）

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionVector {
    versions: HashMap<String, u64>,
    node_id: String,
}

impl VersionVector {
    pub fn new(node_id: String) -> Self {
        Self {
            versions: HashMap::new(),
            node_id,
        }
    }
    
    // 递增版本
    pub fn increment(&mut self) -> u64 {
        let current = self.versions.get(&self.node_id).unwrap_or(&0);
        let new_version = current + 1;
        self.versions.insert(self.node_id.clone(), new_version);
        new_version
    }
    
    // 更新版本向量
    pub fn update(&mut self, other: &VersionVector) {
        for (node_id, &version) in &other.versions {
            let current = self.versions.get(node_id).unwrap_or(&0);
            self.versions.insert(node_id.clone(), (*current).max(version));
        }
    }
    
    // 检查版本向量关系
    pub fn happens_before(&self, other: &VersionVector) -> bool {
        let mut strictly_less = false;
        
        for (node_id, &version) in &self.versions {
            let other_version = other.versions.get(node_id).unwrap_or(&0);
            
            if version > *other_version {
                return false;
            }
            
            if version < *other_version {
                strictly_less = true;
            }
        }
        
        // 检查是否有节点在其他版本向量中存在但在当前版本向量中不存在
        for node_id in other.versions.keys() {
            if !self.versions.contains_key(node_id) {
                strictly_less = true;
            }
        }
        
        strictly_less
    }
    
    // 检查并发关系
    pub fn is_concurrent(&self, other: &VersionVector) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}
```

### 单调性检查器

```rust
pub struct MonotonicityChecker {
    read_trackers: Arc<RwLock<HashMap<String, MonotonicReadTracker>>>,
    write_trackers: Arc<RwLock<HashMap<String, MonotonicWriteTracker>>>,
}

impl MonotonicityChecker {
    pub fn new() -> Self {
        Self {
            read_trackers: Arc::new(RwLock::new(HashMap::new())),
            write_trackers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // 检查单调读违规
    pub fn check_monotonic_read_violation(&self, client_id: &str, key: &str, version: u64) -> bool {
        let read_trackers = self.read_trackers.read().unwrap();
        
        if let Some(tracker) = read_trackers.get(client_id) {
            !tracker.can_read_version(key, version)
        } else {
            false // 首次读取，不违规
        }
    }
    
    // 检查单调写违规
    pub fn check_monotonic_write_violation(&self, client_id: &str, key: &str, sequence: u64) -> bool {
        let write_trackers = self.write_trackers.read().unwrap();
        
        if let Some(tracker) = write_trackers.get(client_id) {
            !tracker.can_write(key, sequence)
        } else {
            false // 首次写入，不违规
        }
    }
    
    // 生成单调性报告
    pub fn generate_monotonicity_report(&self) -> MonotonicityReport {
        let read_trackers = self.read_trackers.read().unwrap();
        let write_trackers = self.write_trackers.read().unwrap();
        
        MonotonicityReport {
            total_clients: read_trackers.len().max(write_trackers.len()),
            read_trackers_count: read_trackers.len(),
            write_trackers_count: write_trackers.len(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonotonicityReport {
    pub total_clients: usize,
    pub read_trackers_count: usize,
    pub write_trackers_count: usize,
    pub timestamp: u64,
}
```

## 🧪 测试策略

### 单调性测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monotonic_reads() {
        let store = MonotonicConsistentStore::new("node1".to_string());
        let client_id = "client1".to_string();
        
        // 写入多个版本
        store.write("key1".to_string(), "value1".to_string(), client_id.clone()).unwrap();
        store.write("key1".to_string(), "value2".to_string(), client_id.clone()).unwrap();
        store.write("key1".to_string(), "value3".to_string(), client_id.clone()).unwrap();
        
        // 第一次读取
        let value1 = store.read("key1".to_string(), client_id.clone()).unwrap();
        
        // 第二次读取应该不会读到更旧的值
        let value2 = store.read("key1".to_string(), client_id.clone()).unwrap();
        assert!(value2 >= value1);
    }
    
    #[test]
    fn test_monotonic_writes() {
        let store = MonotonicConsistentStore::new("node1".to_string());
        let client_id = "client1".to_string();
        
        // 连续写入
        let seq1 = store.write("key1".to_string(), "value1".to_string(), client_id.clone()).unwrap();
        let seq2 = store.write("key1".to_string(), "value2".to_string(), client_id.clone()).unwrap();
        let seq3 = store.write("key1".to_string(), "value3".to_string(), client_id.clone()).unwrap();
        
        // 验证写序列号的单调性
        assert!(seq1 < seq2);
        assert!(seq2 < seq3);
        
        // 读取应该得到最新值
        let value = store.read("key1".to_string(), client_id.clone()).unwrap();
        assert_eq!(value, Some("value3".to_string()));
    }
    
    #[test]
    fn test_version_vector_happens_before() {
        let mut vv1 = VersionVector::new("node1".to_string());
        let mut vv2 = VersionVector::new("node2".to_string());
        
        vv1.increment();
        vv2.update(&vv1);
        vv2.increment();
        
        assert!(vv1.happens_before(&vv2));
        assert!(!vv2.happens_before(&vv1));
    }
    
    #[test]
    fn test_version_vector_concurrent() {
        let mut vv1 = VersionVector::new("node1".to_string());
        let mut vv2 = VersionVector::new("node2".to_string());
        
        vv1.increment();
        vv2.increment();
        
        assert!(vv1.is_concurrent(&vv2));
        assert!(vv2.is_concurrent(&vv1));
    }
    
    #[test]
    fn test_monotonicity_checker() {
        let checker = MonotonicityChecker::new();
        
        // 检查首次读取（应该不违规）
        assert!(!checker.check_monotonic_read_violation("client1", "key1", 1));
        
        // 检查首次写入（应该不违规）
        assert!(!checker.check_monotonic_write_violation("client1", "key1", 1));
        
        // 生成报告
        let report = checker.generate_monotonicity_report();
        assert_eq!(report.total_clients, 0);
    }
}
```

## 🔍 性能优化

### 单调性缓存

```rust
pub struct MonotonicityCache {
    cache: Arc<RwLock<HashMap<String, CachedMonotonicity>>>,
    max_size: usize,
    ttl: u64,
}

#[derive(Debug, Clone)]
pub struct CachedMonotonicity {
    read_tracker: MonotonicReadTracker,
    write_tracker: MonotonicWriteTracker,
    last_access: u64,
    access_count: u64,
}

impl MonotonicityCache {
    pub fn new(max_size: usize, ttl: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl,
        }
    }
    
    // 获取单调性跟踪器
    pub fn get_monotonicity(&self, client_id: &str) -> Option<(MonotonicReadTracker, MonotonicWriteTracker)> {
        let mut cache = self.cache.write().unwrap();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        if let Some(cached) = cache.get_mut(client_id) {
            // 检查是否过期
            if current_time - cached.last_access > self.ttl {
                cache.remove(client_id);
                return None;
            }
            
            cached.last_access = current_time;
            cached.access_count += 1;
            
            Some((cached.read_tracker.clone(), cached.write_tracker.clone()))
        } else {
            None
        }
    }
    
    // 存储单调性跟踪器
    pub fn put_monotonicity(&self, client_id: String, read_tracker: MonotonicReadTracker, write_tracker: MonotonicWriteTracker) {
        let mut cache = self.cache.write().unwrap();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // 检查缓存大小
        if cache.len() >= self.max_size {
            self.evict_least_recently_used(&mut cache);
        }
        
        let cached = CachedMonotonicity {
            read_tracker,
            write_tracker,
            last_access: current_time,
            access_count: 1,
        };
        
        cache.insert(client_id, cached);
    }
    
    // LRU 淘汰
    fn evict_least_recently_used(&self, cache: &mut HashMap<String, CachedMonotonicity>) {
        if let Some((oldest_key, _)) = cache.iter().min_by_key(|(_, cached)| cached.last_access) {
            cache.remove(oldest_key);
        }
    }
    
    // 清理过期条目
    pub fn cleanup_expired(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.cache.write().unwrap();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        cache.retain(|_, cached| {
            current_time - cached.last_access <= self.ttl
        });
        
        Ok(())
    }
}
```

## 📚 进一步阅读

- [一致性模型概述](./README.md) - 一致性模型总览
- [会话一致性](./session_consistency.md) - 会话保证和实现
- [CAP/PACELC 定理](./cap_pacelc.md) - 一致性、可用性、分区容错权衡
- [向量时钟理论](./vector_clocks.md) - 因果依赖跟踪
- [故障处理](../failure/README.md) - 故障检测和处理

## 🔗 相关文档

- [一致性模型](./README.md)
- [会话一致性](./session_consistency.md)
- [CAP/PACELC](./cap_pacelc.md)
- [向量时钟](./vector_clocks.md)
- [故障处理](../failure/README.md)
- [共识机制](../consensus/README.md)
- [复制策略](../replication/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

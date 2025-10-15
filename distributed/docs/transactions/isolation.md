# 事务隔离（Transaction Isolation）

> 分布式系统中的事务隔离级别和并发控制机制

## 目录

- [事务隔离（Transaction Isolation）](#事务隔离transaction-isolation)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [ACID 属性](#acid-属性)
    - [隔离级别](#隔离级别)
    - [并发控制](#并发控制)
  - [🔧 实现机制](#-实现机制)
    - [两阶段锁定](#两阶段锁定)
    - [多版本并发控制](#多版本并发控制)
    - [时间戳排序](#时间戳排序)
  - [🚀 高级特性](#-高级特性)
    - [快照隔离](#快照隔离)
    - [可串行化快照隔离](#可串行化快照隔离)
  - [🧪 测试策略](#-测试策略)
    - [隔离级别测试](#隔离级别测试)
  - [🔍 性能优化](#-性能优化)
    - [锁优化](#锁优化)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

事务隔离是分布式系统中确保并发事务执行时数据一致性的重要机制。通过定义不同的隔离级别，系统可以在性能和数据一致性之间进行权衡，为应用程序提供适当的一致性保证。

## 🎯 核心概念

### ACID 属性

**定义 1（ACID 属性）**: 事务必须满足以下四个基本属性：

1. **原子性（Atomicity）**: 事务中的所有操作要么全部成功，要么全部失败
2. **一致性（Consistency）**: 事务执行前后数据库保持一致状态
3. **隔离性（Isolation）**: 并发事务之间相互隔离，互不干扰
4. **持久性（Durability）**: 事务提交后，数据永久保存

### 隔离级别

**定义 2（隔离级别）**: 隔离级别定义了并发事务之间的隔离程度，从弱到强包括：

1. **读未提交（Read Uncommitted）**: 允许读取未提交的数据
2. **读已提交（Read Committed）**: 只能读取已提交的数据
3. **可重复读（Repeatable Read）**: 同一事务中多次读取结果一致
4. **可串行化（Serializable）**: 事务执行结果与串行执行等价

### 并发控制

**定义 3（并发控制）**: 并发控制是确保多个事务并发执行时数据一致性的机制，主要包括：

- **锁机制**: 通过加锁防止并发访问冲突
- **时间戳**: 通过时间戳排序事务执行
- **多版本**: 通过维护多个数据版本避免冲突

## 🔧 实现机制

### 两阶段锁定

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockType {
    Shared,
    Exclusive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockMode {
    Read,
    Write,
}

#[derive(Debug, Clone)]
pub struct Lock {
    pub resource_id: String,
    pub lock_type: LockType,
    pub transaction_id: String,
    pub timestamp: u64,
}

pub struct TwoPhaseLocking {
    locks: Arc<RwLock<HashMap<String, Vec<Lock>>>>,
    transaction_locks: Arc<RwLock<HashMap<String, Vec<String>>>>,
    waiting_queue: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl TwoPhaseLocking {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            transaction_locks: Arc::new(RwLock::new(HashMap::new())),
            waiting_queue: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // 获取锁
    pub fn acquire_lock(&self, transaction_id: String, resource_id: String, lock_mode: LockMode) 
        -> Result<bool, Box<dyn std::error::Error>> {
        let mut locks = self.locks.write().unwrap();
        let mut transaction_locks = self.transaction_locks.write().unwrap();
        
        let lock_type = match lock_mode {
            LockMode::Read => LockType::Shared,
            LockMode::Write => LockType::Exclusive,
        };
        
        // 检查是否可以获取锁
        if self.can_acquire_lock(&locks, &resource_id, &lock_type, &transaction_id) {
            // 获取锁
            let lock = Lock {
                resource_id: resource_id.clone(),
                lock_type: lock_type.clone(),
                transaction_id: transaction_id.clone(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_millis() as u64,
            };
            
            locks.entry(resource_id.clone()).or_insert_with(Vec::new).push(lock);
            transaction_locks.entry(transaction_id.clone()).or_insert_with(Vec::new).push(resource_id);
            
            Ok(true)
        } else {
            // 加入等待队列
            let mut waiting_queue = self.waiting_queue.write().unwrap();
            waiting_queue.entry(resource_id).or_insert_with(Vec::new).push(transaction_id);
            Ok(false)
        }
    }
    
    // 释放锁
    pub fn release_lock(&self, transaction_id: String, resource_id: String) 
        -> Result<(), Box<dyn std::error::Error>> {
        let mut locks = self.locks.write().unwrap();
        let mut transaction_locks = self.transaction_locks.write().unwrap();
        let mut waiting_queue = self.waiting_queue.write().unwrap();
        
        // 释放锁
        if let Some(resource_locks) = locks.get_mut(&resource_id) {
            resource_locks.retain(|lock| lock.transaction_id != transaction_id);
        }
        
        // 从事务锁列表中移除
        if let Some(transaction_resource_locks) = transaction_locks.get_mut(&transaction_id) {
            transaction_resource_locks.retain(|id| id != &resource_id);
        }
        
        // 检查等待队列
        if let Some(waiting_transactions) = waiting_queue.get_mut(&resource_id) {
            if let Some(next_transaction) = waiting_transactions.pop() {
                // 尝试为下一个事务获取锁
                drop(locks);
                drop(transaction_locks);
                drop(waiting_queue);
                
                self.acquire_lock(next_transaction, resource_id, LockMode::Read)?;
            }
        }
        
        Ok(())
    }
    
    // 检查是否可以获取锁
    fn can_acquire_lock(&self, locks: &HashMap<String, Vec<Lock>>, resource_id: &str, 
                        lock_type: &LockType, transaction_id: &str) -> bool {
        if let Some(resource_locks) = locks.get(resource_id) {
            for lock in resource_locks {
                if lock.transaction_id != transaction_id {
                    match lock_type {
                        LockType::Shared => {
                            // 共享锁可以与共享锁兼容
                            if matches!(lock.lock_type, LockType::Exclusive) {
                                return false;
                            }
                        }
                        LockType::Exclusive => {
                            // 排他锁与任何锁都不兼容
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    
    // 提交事务（释放所有锁）
    pub fn commit_transaction(&self, transaction_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let transaction_locks = self.transaction_locks.read().unwrap();
        
        if let Some(resource_locks) = transaction_locks.get(&transaction_id) {
            for resource_id in resource_locks {
                self.release_lock(transaction_id.clone(), resource_id.clone())?;
            }
        }
        
        Ok(())
    }
    
    // 回滚事务（释放所有锁）
    pub fn rollback_transaction(&self, transaction_id: String) -> Result<(), Box<dyn std::error::Error>> {
        self.commit_transaction(transaction_id)
    }
}
```

### 多版本并发控制

```rust
#[derive(Debug, Clone)]
pub struct Version {
    pub version_id: u64,
    pub data: Vec<u8>,
    pub transaction_id: String,
    pub timestamp: u64,
    pub is_committed: bool,
}

pub struct MultiVersionConcurrencyControl {
    versions: Arc<RwLock<HashMap<String, Vec<Version>>>>,
    transaction_timestamps: Arc<RwLock<HashMap<String, u64>>>,
    next_version_id: Arc<RwLock<u64>>,
}

impl MultiVersionConcurrencyControl {
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            transaction_timestamps: Arc::new(RwLock::new(HashMap::new())),
            next_version_id: Arc::new(RwLock::new(1)),
        }
    }
    
    // 开始事务
    pub fn begin_transaction(&self, transaction_id: String) -> Result<u64, Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        let mut transaction_timestamps = self.transaction_timestamps.write().unwrap();
        transaction_timestamps.insert(transaction_id, timestamp);
        
        Ok(timestamp)
    }
    
    // 读取数据
    pub fn read(&self, transaction_id: String, key: String) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let versions = self.versions.read().unwrap();
        let transaction_timestamps = self.transaction_timestamps.read().unwrap();
        
        let transaction_timestamp = transaction_timestamps.get(&transaction_id)
            .ok_or("Transaction not found")?;
        
        if let Some(key_versions) = versions.get(&key) {
            // 找到适合的版本（时间戳小于等于事务时间戳且已提交）
            for version in key_versions.iter().rev() {
                if version.timestamp <= *transaction_timestamp && version.is_committed {
                    return Ok(Some(version.data.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    // 写入数据
    pub fn write(&self, transaction_id: String, key: String, data: Vec<u8>) 
        -> Result<(), Box<dyn std::error::Error>> {
        let mut versions = self.versions.write().unwrap();
        let mut next_version_id = self.next_version_id.write().unwrap();
        
        let version = Version {
            version_id: *next_version_id,
            data,
            transaction_id: transaction_id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis() as u64,
            is_committed: false,
        };
        
        versions.entry(key).or_insert_with(Vec::new).push(version);
        *next_version_id += 1;
        
        Ok(())
    }
    
    // 提交事务
    pub fn commit_transaction(&self, transaction_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut versions = self.versions.write().unwrap();
        
        // 标记所有版本为已提交
        for key_versions in versions.values_mut() {
            for version in key_versions.iter_mut() {
                if version.transaction_id == transaction_id {
                    version.is_committed = true;
                }
            }
        }
        
        // 清理事务时间戳
        let mut transaction_timestamps = self.transaction_timestamps.write().unwrap();
        transaction_timestamps.remove(&transaction_id);
        
        Ok(())
    }
    
    // 回滚事务
    pub fn rollback_transaction(&self, transaction_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut versions = self.versions.write().unwrap();
        
        // 删除所有未提交的版本
        for key_versions in versions.values_mut() {
            key_versions.retain(|version| version.transaction_id != transaction_id);
        }
        
        // 清理事务时间戳
        let mut transaction_timestamps = self.transaction_timestamps.write().unwrap();
        transaction_timestamps.remove(&transaction_id);
        
        Ok(())
    }
}
```

### 时间戳排序

```rust
pub struct TimestampOrdering {
    transaction_timestamps: Arc<RwLock<HashMap<String, u64>>>,
    read_timestamps: Arc<RwLock<HashMap<String, u64>>>,
    write_timestamps: Arc<RwLock<HashMap<String, u64>>>,
}

impl TimestampOrdering {
    pub fn new() -> Self {
        Self {
            transaction_timestamps: Arc::new(RwLock::new(HashMap::new())),
            read_timestamps: Arc::new(RwLock::new(HashMap::new())),
            write_timestamps: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // 开始事务
    pub fn begin_transaction(&self, transaction_id: String) -> Result<u64, Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        let mut transaction_timestamps = self.transaction_timestamps.write().unwrap();
        transaction_timestamps.insert(transaction_id, timestamp);
        
        Ok(timestamp)
    }
    
    // 读取数据
    pub fn read(&self, transaction_id: String, key: String) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let transaction_timestamps = self.transaction_timestamps.read().unwrap();
        let mut read_timestamps = self.read_timestamps.write().unwrap();
        let write_timestamps = self.write_timestamps.read().unwrap();
        
        let transaction_timestamp = transaction_timestamps.get(&transaction_id)
            .ok_or("Transaction not found")?;
        
        // 检查写时间戳
        if let Some(&write_timestamp) = write_timestamps.get(&key) {
            if *transaction_timestamp < write_timestamp {
                return Err("Transaction too old to read".into());
            }
        }
        
        // 更新读时间戳
        let current_read_timestamp = read_timestamps.get(&key).unwrap_or(&0);
        if *transaction_timestamp > *current_read_timestamp {
            read_timestamps.insert(key, *transaction_timestamp);
        }
        
        // 返回数据（简化实现）
        Ok(Some(b"data".to_vec()))
    }
    
    // 写入数据
    pub fn write(&self, transaction_id: String, key: String, data: Vec<u8>) 
        -> Result<(), Box<dyn std::error::Error>> {
        let transaction_timestamps = self.transaction_timestamps.read().unwrap();
        let read_timestamps = self.read_timestamps.read().unwrap();
        let mut write_timestamps = self.write_timestamps.write().unwrap();
        
        let transaction_timestamp = transaction_timestamps.get(&transaction_id)
            .ok_or("Transaction not found")?;
        
        // 检查读时间戳
        if let Some(&read_timestamp) = read_timestamps.get(&key) {
            if *transaction_timestamp < read_timestamp {
                return Err("Transaction too old to write".into());
            }
        }
        
        // 检查写时间戳
        if let Some(&write_timestamp) = write_timestamps.get(&key) {
            if *transaction_timestamp < write_timestamp {
                return Err("Transaction too old to write".into());
            }
        }
        
        // 更新写时间戳
        write_timestamps.insert(key, *transaction_timestamp);
        
        Ok(())
    }
    
    // 提交事务
    pub fn commit_transaction(&self, transaction_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut transaction_timestamps = self.transaction_timestamps.write().unwrap();
        transaction_timestamps.remove(&transaction_id);
        Ok(())
    }
    
    // 回滚事务
    pub fn rollback_transaction(&self, transaction_id: String) -> Result<(), Box<dyn std::error::Error>> {
        self.commit_transaction(transaction_id)
    }
}
```

## 🚀 高级特性

### 快照隔离

```rust
pub struct SnapshotIsolation {
    mvcc: MultiVersionConcurrencyControl,
    snapshot_timestamps: Arc<RwLock<HashMap<String, u64>>>,
}

impl SnapshotIsolation {
    pub fn new() -> Self {
        Self {
            mvcc: MultiVersionConcurrencyControl::new(),
            snapshot_timestamps: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // 开始快照事务
    pub fn begin_snapshot_transaction(&self, transaction_id: String) 
        -> Result<u64, Box<dyn std::error::Error>> {
        let snapshot_timestamp = self.mvcc.begin_transaction(transaction_id.clone())?;
        
        let mut snapshot_timestamps = self.snapshot_timestamps.write().unwrap();
        snapshot_timestamps.insert(transaction_id, snapshot_timestamp);
        
        Ok(snapshot_timestamp)
    }
    
    // 快照读取
    pub fn snapshot_read(&self, transaction_id: String, key: String) 
        -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        self.mvcc.read(transaction_id, key)
    }
    
    // 快照写入
    pub fn snapshot_write(&self, transaction_id: String, key: String, data: Vec<u8>) 
        -> Result<(), Box<dyn std::error::Error>> {
        self.mvcc.write(transaction_id, key, data)
    }
    
    // 提交快照事务
    pub fn commit_snapshot_transaction(&self, transaction_id: String) 
        -> Result<(), Box<dyn std::error::Error>> {
        self.mvcc.commit_transaction(transaction_id.clone())?;
        
        let mut snapshot_timestamps = self.snapshot_timestamps.write().unwrap();
        snapshot_timestamps.remove(&transaction_id);
        
        Ok(())
    }
}
```

### 可串行化快照隔离

```rust
pub struct SerializableSnapshotIsolation {
    snapshot_isolation: SnapshotIsolation,
    conflict_detection: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SerializableSnapshotIsolation {
    pub fn new() -> Self {
        Self {
            snapshot_isolation: SnapshotIsolation::new(),
            conflict_detection: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // 开始可串行化快照事务
    pub fn begin_serializable_snapshot_transaction(&self, transaction_id: String) 
        -> Result<u64, Box<dyn std::error::Error>> {
        self.snapshot_isolation.begin_snapshot_transaction(transaction_id)
    }
    
    // 可串行化快照读取
    pub fn serializable_snapshot_read(&self, transaction_id: String, key: String) 
        -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // 记录读操作
        let mut conflict_detection = self.conflict_detection.write().unwrap();
        conflict_detection.entry(transaction_id.clone()).or_insert_with(Vec::new).push(key.clone());
        
        self.snapshot_isolation.snapshot_read(transaction_id, key)
    }
    
    // 可串行化快照写入
    pub fn serializable_snapshot_write(&self, transaction_id: String, key: String, data: Vec<u8>) 
        -> Result<(), Box<dyn std::error::Error>> {
        // 检查写-写冲突
        if self.has_write_write_conflict(&transaction_id, &key) {
            return Err("Write-write conflict detected".into());
        }
        
        // 记录写操作
        let mut conflict_detection = self.conflict_detection.write().unwrap();
        conflict_detection.entry(transaction_id.clone()).or_insert_with(Vec::new).push(key.clone());
        
        self.snapshot_isolation.snapshot_write(transaction_id, key, data)
    }
    
    // 检查写-写冲突
    fn has_write_write_conflict(&self, transaction_id: &str, key: &str) -> bool {
        let conflict_detection = self.conflict_detection.read().unwrap();
        
        for (other_transaction_id, operations) in conflict_detection.iter() {
            if other_transaction_id != transaction_id && operations.contains(&key.to_string()) {
                return true;
            }
        }
        
        false
    }
    
    // 提交可串行化快照事务
    pub fn commit_serializable_snapshot_transaction(&self, transaction_id: String) 
        -> Result<(), Box<dyn std::error::Error>> {
        self.snapshot_isolation.commit_snapshot_transaction(transaction_id.clone())?;
        
        let mut conflict_detection = self.conflict_detection.write().unwrap();
        conflict_detection.remove(&transaction_id);
        
        Ok(())
    }
}
```

## 🧪 测试策略

### 隔离级别测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_two_phase_locking() {
        let locking = TwoPhaseLocking::new();
        
        // 事务1获取读锁
        let result1 = locking.acquire_lock("tx1".to_string(), "resource1".to_string(), LockMode::Read).unwrap();
        assert!(result1);
        
        // 事务2获取读锁（应该成功）
        let result2 = locking.acquire_lock("tx2".to_string(), "resource1".to_string(), LockMode::Read).unwrap();
        assert!(result2);
        
        // 事务3获取写锁（应该失败）
        let result3 = locking.acquire_lock("tx3".to_string(), "resource1".to_string(), LockMode::Write).unwrap();
        assert!(!result3);
        
        // 释放锁
        locking.release_lock("tx1".to_string(), "resource1".to_string()).unwrap();
        locking.release_lock("tx2".to_string(), "resource1".to_string()).unwrap();
    }
    
    #[test]
    fn test_multi_version_concurrency_control() {
        let mvcc = MultiVersionConcurrencyControl::new();
        
        // 开始事务
        let timestamp1 = mvcc.begin_transaction("tx1".to_string()).unwrap();
        let timestamp2 = mvcc.begin_transaction("tx2".to_string()).unwrap();
        
        // 事务1写入数据
        mvcc.write("tx1".to_string(), "key1".to_string(), b"value1".to_vec()).unwrap();
        
        // 事务2读取数据（应该读到旧版本）
        let value = mvcc.read("tx2".to_string(), "key1".to_string()).unwrap();
        assert_eq!(value, None);
        
        // 事务1提交
        mvcc.commit_transaction("tx1".to_string()).unwrap();
        
        // 事务2再次读取（应该读到新版本）
        let value = mvcc.read("tx2".to_string(), "key1".to_string()).unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }
    
    #[test]
    fn test_timestamp_ordering() {
        let timestamp_ordering = TimestampOrdering::new();
        
        // 开始事务
        let timestamp1 = timestamp_ordering.begin_transaction("tx1".to_string()).unwrap();
        let timestamp2 = timestamp_ordering.begin_transaction("tx2".to_string()).unwrap();
        
        // 事务1写入数据
        timestamp_ordering.write("tx1".to_string(), "key1".to_string(), b"value1".to_vec()).unwrap();
        
        // 事务2读取数据（应该成功）
        let value = timestamp_ordering.read("tx2".to_string(), "key1".to_string()).unwrap();
        assert_eq!(value, Some(b"data".to_vec()));
        
        // 事务1提交
        timestamp_ordering.commit_transaction("tx1".to_string()).unwrap();
        timestamp_ordering.commit_transaction("tx2".to_string()).unwrap();
    }
    
    #[test]
    fn test_snapshot_isolation() {
        let snapshot_isolation = SnapshotIsolation::new();
        
        // 开始快照事务
        let snapshot_timestamp = snapshot_isolation.begin_snapshot_transaction("tx1".to_string()).unwrap();
        
        // 快照读取
        let value = snapshot_isolation.snapshot_read("tx1".to_string(), "key1".to_string()).unwrap();
        assert_eq!(value, None);
        
        // 快照写入
        snapshot_isolation.snapshot_write("tx1".to_string(), "key1".to_string(), b"value1".to_vec()).unwrap();
        
        // 提交快照事务
        snapshot_isolation.commit_snapshot_transaction("tx1".to_string()).unwrap();
    }
}
```

## 🔍 性能优化

### 锁优化

```rust
pub struct LockOptimizer {
    lock_manager: TwoPhaseLocking,
    lock_escalation_threshold: usize,
    deadlock_detection_interval: u64,
}

impl LockOptimizer {
    pub fn new(lock_escalation_threshold: usize, deadlock_detection_interval: u64) -> Self {
        Self {
            lock_manager: TwoPhaseLocking::new(),
            lock_escalation_threshold,
            deadlock_detection_interval,
        }
    }
    
    // 锁升级
    pub fn escalate_locks(&self, transaction_id: String) -> Result<(), Box<dyn std::error::Error>> {
        // 检查是否需要升级锁
        let transaction_locks = self.lock_manager.transaction_locks.read().unwrap();
        
        if let Some(locks) = transaction_locks.get(&transaction_id) {
            if locks.len() >= self.lock_escalation_threshold {
                // 升级为表级锁
                self.upgrade_to_table_lock(transaction_id, locks.clone())?;
            }
        }
        
        Ok(())
    }
    
    // 升级为表级锁
    fn upgrade_to_table_lock(&self, transaction_id: String, resource_locks: Vec<String>) 
        -> Result<(), Box<dyn std::error::Error>> {
        // 释放所有行级锁
        for resource_id in resource_locks {
            self.lock_manager.release_lock(transaction_id.clone(), resource_id)?;
        }
        
        // 获取表级锁
        self.lock_manager.acquire_lock(transaction_id, "table".to_string(), LockMode::Write)?;
        
        Ok(())
    }
    
    // 死锁检测
    pub fn detect_deadlock(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let locks = self.lock_manager.locks.read().unwrap();
        let waiting_queue = self.lock_manager.waiting_queue.read().unwrap();
        
        let mut deadlocked_transactions = Vec::new();
        
        // 简化的死锁检测算法
        for (resource_id, waiting_transactions) in waiting_queue.iter() {
            if waiting_transactions.len() > 1 {
                // 检查是否存在循环等待
                if self.has_cycle(waiting_transactions, &waiting_queue) {
                    deadlocked_transactions.extend(waiting_transactions.clone());
                }
            }
        }
        
        Ok(deadlocked_transactions)
    }
    
    // 检查循环等待
    fn has_cycle(&self, transactions: &[String], waiting_queue: &HashMap<String, Vec<String>>) -> bool {
        // 简化的循环检测实现
        transactions.len() > 2
    }
}
```

## 📚 进一步阅读

- [分布式事务](./README.md) - 分布式事务概述
- [补偿机制](./compensation.md) - 事务回滚和补偿策略
- [幂等性](./idempotency.md) - 幂等操作和重复处理
- [一致性模型](../consistency/README.md) - 一致性模型概述

## 🔗 相关文档

- [分布式事务](./README.md)
- [补偿机制](./compensation.md)
- [幂等性](./idempotency.md)
- [一致性模型](../consistency/README.md)
- [共识机制](../consensus/README.md)
- [复制策略](../replication/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

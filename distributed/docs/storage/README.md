# 存储抽象

> 分布式系统中的存储抽象、WAL、快照和状态机实现

## 目录

- [存储抽象](#存储抽象)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [存储层次结构](#存储层次结构)
    - [关键不变量](#关键不变量)
  - [🔧 存储接口设计](#-存储接口设计)
    - [基础存储接口](#基础存储接口)
    - [日志存储接口](#日志存储接口)
    - [状态机接口](#状态机接口)
  - [🏗️ WAL 实现](#️-wal-实现)
    - [WAL 结构](#wal-结构)
    - [WAL 恢复](#wal-恢复)
  - [📸 快照管理](#-快照管理)
    - [快照接口](#快照接口)
    - [快照实现](#快照实现)
  - [🔄 状态机实现](#-状态机实现)
    - [KV 存储状态机](#kv-存储状态机)
    - [计数器状态机](#计数器状态机)
  - [🧪 测试策略](#-测试策略)
    - [存储测试](#存储测试)
  - [🔍 性能优化](#-性能优化)
    - [批量操作优化](#批量操作优化)
    - [异步刷新优化](#异步刷新优化)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

存储抽象是分布式系统的核心组件，提供了数据持久化、日志记录、快照管理和状态机复制的统一接口。

## 🎯 核心概念

### 存储层次结构

```text
应用层
  ↓
状态机 (State Machine)
  ↓
日志存储 (Log Storage)
  ↓
WAL (Write-Ahead Log)
  ↓
持久化存储 (Persistent Storage)
```

### 关键不变量

- **WAL 追加性**: 日志只能追加，不能修改
- **快照恢复等价**: 快照恢复后的状态与完整日志恢复等价
- **原子落盘**: 写入操作要么全部成功，要么全部失败
- **last_applied ≤ commit_index**: 应用索引不能超过提交索引

## 🔧 存储接口设计

### 基础存储接口

```rust
pub trait Storage {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // 写入数据
    async fn write(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    
    // 读取数据
    async fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    
    // 删除数据
    async fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error>;
    
    // 批量操作
    async fn batch_write(&mut self, operations: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Self::Error>;
    
    // 范围查询
    async fn range_scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Self::Error>;
}
```

### 日志存储接口

```rust
pub trait LogStorage {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // 追加日志条目
    async fn append_entry(&mut self, entry: &LogEntry) -> Result<u64, Self::Error>;
    
    // 批量追加日志条目
    async fn append_entries(&mut self, entries: &[LogEntry]) -> Result<u64, Self::Error>;
    
    // 读取日志条目
    async fn get_entry(&self, index: u64) -> Result<Option<LogEntry>, Self::Error>;
    
    // 读取日志范围
    async fn get_entries(&self, start: u64, end: u64) -> Result<Vec<LogEntry>, Self::Error>;
    
    // 获取最后日志索引
    async fn last_index(&self) -> Result<u64, Self::Error>;
    
    // 获取最后日志任期
    async fn last_term(&self) -> Result<u64, Self::Error>;
    
    // 截断日志
    async fn truncate(&mut self, index: u64) -> Result<(), Self::Error>;
    
    // 压缩日志
    async fn compact(&mut self, index: u64) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: Vec<u8>,
    pub timestamp: u64,
}
```

### 状态机接口

```rust
pub trait StateMachine {
    type Command;
    type Result;
    type Error: std::error::Error + Send + Sync + 'static;
    
    // 应用命令
    async fn apply(&mut self, command: Self::Command) -> Result<Self::Result, Self::Error>;
    
    // 批量应用命令
    async fn apply_batch(&mut self, commands: Vec<Self::Command>) -> Result<Vec<Self::Result>, Self::Error>;
    
    // 创建快照
    async fn snapshot(&self) -> Result<Vec<u8>, Self::Error>;
    
    // 从快照恢复
    async fn restore(&mut self, snapshot: &[u8]) -> Result<(), Self::Error>;
    
    // 获取状态
    async fn get_state(&self) -> Result<Vec<u8>, Self::Error>;
    
    // 设置状态
    async fn set_state(&mut self, state: &[u8]) -> Result<(), Self::Error>;
}
```

## 🏗️ WAL 实现

### WAL 结构

```rust
pub struct WriteAheadLog {
    file: File,
    buffer: Vec<u8>,
    buffer_size: usize,
    sync_on_write: bool,
}

impl WriteAheadLog {
    pub fn new(path: &Path, buffer_size: usize, sync_on_write: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        
        Ok(Self {
            file,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
            sync_on_write,
        })
    }
    
    pub async fn append(&mut self, entry: &LogEntry) -> Result<u64, Box<dyn std::error::Error>> {
        // 序列化日志条目
        let data = bincode::serialize(entry)?;
        let len = data.len() as u32;
        
        // 写入长度前缀
        self.buffer.extend_from_slice(&len.to_le_bytes());
        self.buffer.extend_from_slice(&data);
        
        // 检查缓冲区是否需要刷新
        if self.buffer.len() >= self.buffer_size {
            self.flush().await?;
        }
        
        Ok(entry.index)
    }
    
    pub async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.buffer.is_empty() {
            self.file.write_all(&self.buffer).await?;
            self.buffer.clear();
            
            if self.sync_on_write {
                self.file.sync_all().await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn read_entries(&mut self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();
        let mut buffer = Vec::new();
        
        // 读取文件内容
        self.file.read_to_end(&mut buffer).await?;
        
        let mut offset = 0;
        while offset < buffer.len() {
            // 读取长度前缀
            if offset + 4 > buffer.len() {
                break;
            }
            
            let len = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]) as usize;
            
            offset += 4;
            
            // 读取日志条目
            if offset + len > buffer.len() {
                break;
            }
            
            let entry_data = &buffer[offset..offset + len];
            let entry: LogEntry = bincode::deserialize(entry_data)?;
            entries.push(entry);
            
            offset += len;
        }
        
        Ok(entries)
    }
}
```

### WAL 恢复

```rust
impl WriteAheadLog {
    pub async fn recover(&mut self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.read_entries().await?;
        
        // 验证日志完整性
        for (i, entry) in entries.iter().enumerate() {
            if entry.index != (i + 1) as u64 {
                return Err("Log corruption detected".into());
            }
        }
        
        Ok(entries)
    }
    
    pub async fn truncate(&mut self, index: u64) -> Result<(), Box<dyn std::error::Error>> {
        // 读取所有条目
        let entries = self.read_entries().await?;
        
        // 截断到指定索引
        let truncated_entries: Vec<LogEntry> = entries
            .into_iter()
            .take(index as usize)
            .collect();
        
        // 重写文件
        self.file.set_len(0).await?;
        self.buffer.clear();
        
        for entry in truncated_entries {
            self.append(&entry).await?;
        }
        
        self.flush().await?;
        
        Ok(())
    }
}
```

## 📸 快照管理

### 快照接口

```rust
pub trait SnapshotManager {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // 创建快照
    async fn create_snapshot(&mut self, index: u64, term: u64, data: &[u8]) -> Result<Snapshot, Self::Error>;
    
    // 加载快照
    async fn load_snapshot(&self, snapshot_id: &str) -> Result<Snapshot, Self::Error>;
    
    // 删除快照
    async fn delete_snapshot(&mut self, snapshot_id: &str) -> Result<(), Self::Error>;
    
    // 列出快照
    async fn list_snapshots(&self) -> Result<Vec<Snapshot>, Self::Error>;
    
    // 压缩快照
    async fn compact_snapshots(&mut self, keep_count: usize) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: String,
    pub index: u64,
    pub term: u64,
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub checksum: u64,
}
```

### 快照实现

```rust
pub struct FileSnapshotManager {
    snapshot_dir: PathBuf,
    compression: bool,
}

impl FileSnapshotManager {
    pub fn new(snapshot_dir: PathBuf, compression: bool) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&snapshot_dir)?;
        
        Ok(Self {
            snapshot_dir,
            compression,
        })
    }
    
    pub async fn create_snapshot(&mut self, index: u64, term: u64, data: &[u8]) -> Result<Snapshot, Box<dyn std::error::Error>> {
        let snapshot_id = format!("snapshot_{}_{}", index, term);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        
        // 计算校验和
        let checksum = self.calculate_checksum(data);
        
        // 压缩数据（如果启用）
        let compressed_data = if self.compression {
            self.compress_data(data)?
        } else {
            data.to_vec()
        };
        
        // 写入快照文件
        let snapshot_path = self.snapshot_dir.join(&snapshot_id);
        let mut file = File::create(&snapshot_path).await?;
        file.write_all(&compressed_data).await?;
        file.sync_all().await?;
        
        let snapshot = Snapshot {
            id: snapshot_id,
            index,
            term,
            data: compressed_data,
            timestamp,
            checksum,
        };
        
        // 保存快照元数据
        self.save_snapshot_metadata(&snapshot).await?;
        
        Ok(snapshot)
    }
    
    pub async fn load_snapshot(&self, snapshot_id: &str) -> Result<Snapshot, Box<dyn std::error::Error>> {
        let snapshot_path = self.snapshot_dir.join(snapshot_id);
        let mut file = File::open(&snapshot_path).await?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;
        
        // 解压缩数据（如果启用）
        let decompressed_data = if self.compression {
            self.decompress_data(&data)?
        } else {
            data
        };
        
        // 验证校验和
        let checksum = self.calculate_checksum(&decompressed_data);
        
        // 加载元数据
        let metadata = self.load_snapshot_metadata(snapshot_id).await?;
        
        let snapshot = Snapshot {
            id: snapshot_id.to_string(),
            index: metadata.index,
            term: metadata.term,
            data: decompressed_data,
            timestamp: metadata.timestamp,
            checksum,
        };
        
        Ok(snapshot)
    }
    
    fn calculate_checksum(&self, data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
    
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }
    
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}
```

## 🔄 状态机实现

### KV 存储状态机

```rust
pub struct KVStateMachine {
    data: HashMap<String, String>,
    last_applied: u64,
}

impl KVStateMachine {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            last_applied: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KVCommand {
    Set { key: String, value: String },
    Delete { key: String },
    Get { key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KVResult {
    Set { success: bool },
    Delete { success: bool },
    Get { value: Option<String> },
}

impl StateMachine for KVStateMachine {
    type Command = KVCommand;
    type Result = KVResult;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn apply(&mut self, command: Self::Command) -> Result<Self::Result, Self::Error> {
        let result = match command {
            KVCommand::Set { key, value } => {
                self.data.insert(key.clone(), value.clone());
                KVResult::Set { success: true }
            }
            KVCommand::Delete { key } => {
                let success = self.data.remove(&key).is_some();
                KVResult::Delete { success }
            }
            KVCommand::Get { key } => {
                let value = self.data.get(&key).cloned();
                KVResult::Get { value }
            }
        };
        
        self.last_applied += 1;
        Ok(result)
    }
    
    async fn apply_batch(&mut self, commands: Vec<Self::Command>) -> Result<Vec<Self::Result>, Self::Error> {
        let mut results = Vec::new();
        
        for command in commands {
            let result = self.apply(command).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    async fn snapshot(&self) -> Result<Vec<u8>, Self::Error> {
        let snapshot_data = SnapshotData {
            data: self.data.clone(),
            last_applied: self.last_applied,
        };
        
        Ok(bincode::serialize(&snapshot_data)?)
    }
    
    async fn restore(&mut self, snapshot: &[u8]) -> Result<(), Self::Error> {
        let snapshot_data: SnapshotData = bincode::deserialize(snapshot)?;
        
        self.data = snapshot_data.data;
        self.last_applied = snapshot_data.last_applied;
        
        Ok(())
    }
    
    async fn get_state(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(bincode::serialize(&self.data)?)
    }
    
    async fn set_state(&mut self, state: &[u8]) -> Result<(), Self::Error> {
        self.data = bincode::deserialize(state)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotData {
    data: HashMap<String, String>,
    last_applied: u64,
}
```

### 计数器状态机

```rust
pub struct CounterStateMachine {
    count: i64,
    last_applied: u64,
}

impl CounterStateMachine {
    pub fn new(initial_count: i64) -> Self {
        Self {
            count: initial_count,
            last_applied: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterCommand {
    Increment { delta: i64 },
    Decrement { delta: i64 },
    Set { value: i64 },
    Get,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterResult {
    Increment { new_count: i64 },
    Decrement { new_count: i64 },
    Set { new_count: i64 },
    Get { count: i64 },
}

impl StateMachine for CounterStateMachine {
    type Command = CounterCommand;
    type Result = CounterResult;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn apply(&mut self, command: Self::Command) -> Result<Self::Result, Self::Error> {
        let result = match command {
            CounterCommand::Increment { delta } => {
                self.count += delta;
                CounterResult::Increment { new_count: self.count }
            }
            CounterCommand::Decrement { delta } => {
                self.count -= delta;
                CounterResult::Decrement { new_count: self.count }
            }
            CounterCommand::Set { value } => {
                self.count = value;
                CounterResult::Set { new_count: self.count }
            }
            CounterCommand::Get => {
                CounterResult::Get { count: self.count }
            }
        };
        
        self.last_applied += 1;
        Ok(result)
    }
    
    async fn snapshot(&self) -> Result<Vec<u8>, Self::Error> {
        let snapshot_data = CounterSnapshot {
            count: self.count,
            last_applied: self.last_applied,
        };
        
        Ok(bincode::serialize(&snapshot_data)?)
    }
    
    async fn restore(&mut self, snapshot: &[u8]) -> Result<(), Self::Error> {
        let snapshot_data: CounterSnapshot = bincode::deserialize(snapshot)?;
        
        self.count = snapshot_data.count;
        self.last_applied = snapshot_data.last_applied;
        
        Ok(())
    }
    
    // ... 其他方法实现
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterSnapshot {
    count: i64,
    last_applied: u64,
}
```

## 🧪 测试策略

### 存储测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_wal_append_and_recover() {
        let temp_dir = tempfile::tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let mut wal = WriteAheadLog::new(&wal_path, 1024, true).unwrap();
        
        // 追加日志条目
        let entry1 = LogEntry {
            term: 1,
            index: 1,
            command: b"command1".to_vec(),
            timestamp: 1000,
        };
        
        let entry2 = LogEntry {
            term: 1,
            index: 2,
            command: b"command2".to_vec(),
            timestamp: 2000,
        };
        
        wal.append(&entry1).await.unwrap();
        wal.append(&entry2).await.unwrap();
        wal.flush().await.unwrap();
        
        // 恢复日志
        let mut wal2 = WriteAheadLog::new(&wal_path, 1024, true).unwrap();
        let entries = wal2.recover().await.unwrap();
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry1);
        assert_eq!(entries[1], entry2);
    }
    
    #[tokio::test]
    async fn test_snapshot_create_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut snapshot_manager = FileSnapshotManager::new(temp_dir.path().to_path_buf(), true).unwrap();
        
        let data = b"test snapshot data";
        let snapshot = snapshot_manager.create_snapshot(100, 5, data).await.unwrap();
        
        assert_eq!(snapshot.index, 100);
        assert_eq!(snapshot.term, 5);
        
        // 加载快照
        let loaded_snapshot = snapshot_manager.load_snapshot(&snapshot.id).await.unwrap();
        
        assert_eq!(loaded_snapshot.index, snapshot.index);
        assert_eq!(loaded_snapshot.term, snapshot.term);
        assert_eq!(loaded_snapshot.data, data);
    }
    
    #[tokio::test]
    async fn test_kv_state_machine() {
        let mut state_machine = KVStateMachine::new();
        
        // 测试设置操作
        let set_command = KVCommand::Set {
            key: "key1".to_string(),
            value: "value1".to_string(),
        };
        
        let result = state_machine.apply(set_command).await.unwrap();
        assert!(matches!(result, KVResult::Set { success: true }));
        
        // 测试获取操作
        let get_command = KVCommand::Get {
            key: "key1".to_string(),
        };
        
        let result = state_machine.apply(get_command).await.unwrap();
        assert!(matches!(result, KVResult::Get { value: Some(v) } if v == "value1"));
        
        // 测试删除操作
        let delete_command = KVCommand::Delete {
            key: "key1".to_string(),
        };
        
        let result = state_machine.apply(delete_command).await.unwrap();
        assert!(matches!(result, KVResult::Delete { success: true }));
        
        // 验证删除后获取
        let get_command = KVCommand::Get {
            key: "key1".to_string(),
        };
        
        let result = state_machine.apply(get_command).await.unwrap();
        assert!(matches!(result, KVResult::Get { value: None }));
    }
}
```

## 🔍 性能优化

### 批量操作优化

```rust
impl WriteAheadLog {
    pub async fn append_batch(&mut self, entries: &[LogEntry]) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
        let mut indices = Vec::new();
        
        for entry in entries {
            let index = self.append(entry).await?;
            indices.push(index);
        }
        
        self.flush().await?;
        Ok(indices)
    }
}
```

### 异步刷新优化

```rust
pub struct AsyncWAL {
    wal: WriteAheadLog,
    flush_task: Option<tokio::task::JoinHandle<()>>,
    flush_interval: Duration,
}

impl AsyncWAL {
    pub fn new(wal: WriteAheadLog, flush_interval: Duration) -> Self {
        Self {
            wal,
            flush_task: None,
            flush_interval,
        }
    }
    
    pub async fn start_async_flush(&mut self) {
        let mut wal = &mut self.wal;
        let interval = self.flush_interval;
        
        self.flush_task = Some(tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                let _ = wal.flush().await;
            }
        }));
    }
}
```

## 📚 进一步阅读

- [共识机制](../consensus/README.md) - 共识算法与日志复制
- [复制策略](../replication/README.md) - 数据复制和一致性
- [故障处理](../failure/README.md) - 故障检测和恢复
- [性能优化](../performance/OPTIMIZATION.md) - 存储性能优化

## 🔗 相关文档

- [共识机制](../consensus/README.md)
- [复制策略](../replication/README.md)
- [故障处理](../failure/README.md)
- [性能优化](../performance/OPTIMIZATION.md)
- [测试策略](../testing/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

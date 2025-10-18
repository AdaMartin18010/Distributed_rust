# RocksDB 集成技术设计文档

**版本**: v1.0  
**日期**: 2025年10月17日  
**状态**: 设计阶段  
**优先级**: 🔴 P0 - 最高

---

## 📋 文档概览

### 目标

集成RocksDB作为生产级持久化存储引擎，实现WAL（Write-Ahead Log）和快照机制。

### 预期收益

- **数据持久性**: 提供ACID保证的数据持久化
- **性能提升**: 利用RocksDB的LSM-Tree优化读写性能
- **生产就绪**: 支持生产环境部署
- **可靠性**: 故障恢复和数据一致性保证

### 资源估算

- **开发时间**: 3周
- **测试时间**: 1周
- **开发人员**: 2名存储专家
- **审查人员**: 1名架构师

---

## 🎯 背景和动机

### 当前问题

1. **缺少持久化**: 数据仅在内存中，重启后丢失
2. **无法生产使用**: 不能用于需要数据持久化的场景
3. **恢复能力弱**: 故障后无法恢复数据
4. **扩展性差**: 数据量受内存限制

### 为什么选择RocksDB

#### RocksDB的优势

1. **高性能**: LSM-Tree设计，优化写入和范围查询
2. **成熟稳定**: Facebook出品，TiKV、CockroachDB等广泛使用
3. **Rust生态**: 有良好的Rust绑定（rust-rocksdb）
4. **功能丰富**: 支持事务、快照、压缩、备份等

#### 行业标准

| 项目 | 存储引擎 | 使用场景 |
|------|---------|---------|
| TiKV | RocksDB | 分布式KV存储 |
| CockroachDB | RocksDB | 分布式数据库 |
| Databend | 对象存储 + RocksDB | 数据仓库 |
| etcd | bbolt (BoltDB) | 配置存储 |

### 设计目标

1. **数据持久化**: 所有数据安全存储到磁盘
2. **快速恢复**: 支持从磁盘快速恢复状态
3. **高性能**: 不显著影响读写性能
4. **易于运维**: 支持备份、恢复、压缩等操作

---

## 🏗️ 技术设计

### 整体架构

```text
┌─────────────────────────────────────────┐
│         Raft State Machine              │
├─────────────────────────────────────────┤
│         Storage Abstraction             │
│  ┌──────────┐  ┌──────────┐  ┌────────┐│
│  │ Log Store│  │State Store│  │Metadata││
│  └─────┬────┘  └─────┬─────┘  └───┬────┘│
├────────┴─────────────┴─────────────┴─────┤
│          RocksDB Storage Engine          │
│  ┌──────────────────────────────────┐   │
│  │  Column Families:                │   │
│  │  - raft_log: Raft日志            │   │
│  │  - state_machine: 状态机数据     │   │
│  │  - metadata: 元数据              │   │
│  │  - snapshots: 快照索引           │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### Column Family设计

#### 1. raft_log (Raft日志)

```rust
/// Key: LogIndex (8 bytes)
/// Value: LogEntry (变长)
/// 
/// LogEntry 结构:
/// - term: u64
/// - index: u64
/// - entry_type: EntryType
/// - data: Vec<u8>
```

**特点**:

- 顺序写入为主
- 很少随机读取
- 定期压缩旧日志

**优化配置**:

```rust
let mut opts = Options::default();
opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
opts.set_max_write_buffer_number(3);
opts.set_target_file_size_base(64 * 1024 * 1024);
opts.set_compression_type(DBCompressionType::Lz4);
```

#### 2. state_machine (状态机数据)

```rust
/// Key: Application定义的Key (变长)
/// Value: Application定义的Value (变长)
```

**特点**:

- 读写混合
- 支持范围查询
- 数据量可能很大

**优化配置**:

```rust
let mut opts = Options::default();
opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB
opts.set_max_write_buffer_number(4);
opts.set_level_zero_file_num_compaction_trigger(4);
opts.set_compression_type(DBCompressionType::Zstd);
opts.optimize_level_style_compaction(512 * 1024 * 1024);
```

#### 3. metadata (元数据)

```rust
/// 存储的元数据包括:
/// - "current_term": 当前任期
/// - "voted_for": 投票给谁
/// - "commit_index": 提交索引
/// - "last_applied": 最后应用索引
/// - "hard_state": Raft硬状态
```

**特点**:

- 数据量很小
- 读多写少
- 需要高可靠性

**优化配置**:

```rust
let mut opts = Options::default();
opts.set_write_buffer_size(4 * 1024 * 1024); // 4MB
opts.set_max_write_buffer_number(2);
opts.disable_auto_compactions();
```

#### 4. snapshots (快照索引)

```rust
/// Key: SnapshotId (8 bytes)
/// Value: SnapshotMetadata
/// 
/// SnapshotMetadata:
/// - snapshot_id: u64
/// - last_included_index: u64
/// - last_included_term: u64
/// - size: u64
/// - checksum: u64
/// - created_at: Timestamp
```

**特点**:

- 数据量小
- 写入不频繁
- 需要支持删除旧快照

### 核心API设计

#### Storage Trait

```rust
/// 统一的存储抽象
pub trait Storage: Send + Sync {
    // ====== 日志存储 ======
    
    /// 追加日志条目
    fn append_entries(&mut self, entries: &[LogEntry]) -> Result<(), StorageError>;
    
    /// 读取日志条目
    fn get_entry(&self, index: LogIndex) -> Result<Option<LogEntry>, StorageError>;
    
    /// 读取范围日志
    fn get_entries(&self, start: LogIndex, end: LogIndex) -> Result<Vec<LogEntry>, StorageError>;
    
    /// 获取最后一条日志
    fn last_entry(&self) -> Result<Option<LogEntry>, StorageError>;
    
    /// 删除日志（快照后压缩）
    fn delete_entries_before(&mut self, index: LogIndex) -> Result<(), StorageError>;
    
    // ====== 状态存储 ======
    
    /// 保存硬状态（term, voted_for）
    fn save_hard_state(&mut self, state: &HardState) -> Result<(), StorageError>;
    
    /// 读取硬状态
    fn load_hard_state(&self) -> Result<Option<HardState>, StorageError>;
    
    /// 更新提交索引
    fn save_commit_index(&mut self, index: LogIndex) -> Result<(), StorageError>;
    
    /// 读取提交索引
    fn load_commit_index(&self) -> Result<LogIndex, StorageError>;
    
    // ====== 快照管理 ======
    
    /// 创建快照
    fn create_snapshot(&mut self, data: Vec<u8>, metadata: SnapshotMetadata) 
        -> Result<SnapshotId, StorageError>;
    
    /// 读取快照
    fn load_snapshot(&self, id: SnapshotId) -> Result<Snapshot, StorageError>;
    
    /// 应用快照
    fn apply_snapshot(&mut self, snapshot: &Snapshot) -> Result<(), StorageError>;
    
    /// 删除旧快照
    fn delete_old_snapshots(&mut self, keep_count: usize) -> Result<(), StorageError>;
    
    // ====== 状态机存储 ======
    
    /// 写入KV
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    
    /// 读取KV
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    
    /// 删除KV
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError>;
    
    /// 范围查询
    fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StorageError>;
    
    // ====== 事务支持 ======
    
    /// 批量写入
    fn write_batch(&mut self, batch: WriteBatch) -> Result<(), StorageError>;
    
    /// 同步到磁盘
    fn sync(&mut self) -> Result<(), StorageError>;
}
```

#### RocksDB Implementation

```rust
/// RocksDB存储实现
pub struct RocksDBStorage {
    /// RocksDB实例
    db: Arc<DB>,
    
    /// Column Family句柄
    cf_log: ColumnFamily,
    cf_state: ColumnFamily,
    cf_metadata: ColumnFamily,
    cf_snapshots: ColumnFamily,
    
    /// 写选项
    write_opts: WriteOptions,
    
    /// 读选项
    read_opts: ReadOptions,
    
    /// 配置
    config: RocksDBConfig,
}

impl RocksDBStorage {
    /// 创建或打开RocksDB
    pub fn open(path: impl AsRef<Path>, config: RocksDBConfig) 
        -> Result<Self, StorageError> {
        // 1. 设置全局选项
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        
        // 2. 定义Column Families
        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new("raft_log", Self::log_cf_options()),
            ColumnFamilyDescriptor::new("state_machine", Self::state_cf_options()),
            ColumnFamilyDescriptor::new("metadata", Self::metadata_cf_options()),
            ColumnFamilyDescriptor::new("snapshots", Self::snapshot_cf_options()),
        ];
        
        // 3. 打开数据库
        let db = DB::open_cf_descriptors(&db_opts, path, cf_descriptors)?;
        
        // 4. 获取Column Family句柄
        let cf_log = db.cf_handle("raft_log").unwrap();
        let cf_state = db.cf_handle("state_machine").unwrap();
        let cf_metadata = db.cf_handle("metadata").unwrap();
        let cf_snapshots = db.cf_handle("snapshots").unwrap();
        
        // 5. 设置写选项
        let mut write_opts = WriteOptions::default();
        write_opts.set_sync(config.sync_write);
        write_opts.disable_wal(false);
        
        // 6. 设置读选项
        let read_opts = ReadOptions::default();
        
        Ok(Self {
            db: Arc::new(db),
            cf_log,
            cf_state,
            cf_metadata,
            cf_snapshots,
            write_opts,
            read_opts,
            config,
        })
    }
    
    /// 日志CF配置
    fn log_cf_options() -> Options {
        let mut opts = Options::default();
        opts.set_write_buffer_size(64 * 1024 * 1024);
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024);
        opts.set_compression_type(DBCompressionType::Lz4);
        opts
    }
    
    /// 状态CF配置
    fn state_cf_options() -> Options {
        let mut opts = Options::default();
        opts.set_write_buffer_size(128 * 1024 * 1024);
        opts.set_max_write_buffer_number(4);
        opts.set_compression_type(DBCompressionType::Zstd);
        opts.optimize_level_style_compaction(512 * 1024 * 1024);
        opts
    }
    
    /// 元数据CF配置
    fn metadata_cf_options() -> Options {
        let mut opts = Options::default();
        opts.set_write_buffer_size(4 * 1024 * 1024);
        opts.set_max_write_buffer_number(2);
        opts
    }
    
    /// 快照CF配置
    fn snapshot_cf_options() -> Options {
        let mut opts = Options::default();
        opts.set_write_buffer_size(8 * 1024 * 1024);
        opts
    }
}

impl Storage for RocksDBStorage {
    fn append_entries(&mut self, entries: &[LogEntry]) -> Result<(), StorageError> {
        let mut batch = WriteBatch::default();
        
        for entry in entries {
            let key = entry.index.to_be_bytes();
            let value = bincode::serialize(entry)?;
            batch.put_cf(&self.cf_log, key, value);
        }
        
        self.db.write_opt(batch, &self.write_opts)?;
        Ok(())
    }
    
    fn get_entry(&self, index: LogIndex) -> Result<Option<LogEntry>, StorageError> {
        let key = index.0.to_be_bytes();
        let value = self.db.get_cf_opt(&self.cf_log, key, &self.read_opts)?;
        
        match value {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }
    
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.db.put_cf_opt(&self.cf_state, key, value, &self.write_opts)?;
        Ok(())
    }
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.db.get_cf_opt(&self.cf_state, key, &self.read_opts)?)
    }
    
    // ... 其他方法实现
}
```

### WAL (Write-Ahead Log)

RocksDB内置WAL支持，我们需要正确配置：

```rust
pub struct WALConfig {
    /// 是否启用WAL
    pub enabled: bool,
    
    /// WAL目录（可以放在单独的磁盘）
    pub wal_dir: Option<PathBuf>,
    
    /// WAL大小限制（MB）
    pub wal_size_limit_mb: u64,
    
    /// WAL过期时间（秒）
    pub wal_ttl_seconds: u64,
    
    /// 是否在写入时同步WAL
    pub sync_wal: bool,
}

impl RocksDBStorage {
    fn configure_wal(opts: &mut Options, config: &WALConfig) {
        if config.enabled {
            // 设置WAL目录
            if let Some(wal_dir) = &config.wal_dir {
                opts.set_wal_dir(wal_dir);
            }
            
            // 设置WAL大小限制
            opts.set_max_total_wal_size(config.wal_size_limit_mb * 1024 * 1024);
            
            // 设置WAL过期时间
            opts.set_wal_ttl_seconds(config.wal_ttl_seconds);
            
            // WAL恢复模式
            opts.set_wal_recovery_mode(WALRecoveryMode::PointInTime);
        } else {
            // 禁用WAL（不推荐）
            opts.set_manual_wal_flush(true);
        }
    }
}
```

### 快照机制

```rust
/// 快照元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// 快照ID
    pub snapshot_id: u64,
    
    /// 包含的最后日志索引
    pub last_included_index: LogIndex,
    
    /// 包含的最后日志任期
    pub last_included_term: Term,
    
    /// 快照大小（字节）
    pub size: u64,
    
    /// 校验和
    pub checksum: u64,
    
    /// 创建时间
    pub created_at: SystemTime,
}

/// 快照数据
pub struct Snapshot {
    /// 元数据
    pub metadata: SnapshotMetadata,
    
    /// 快照数据
    pub data: Vec<u8>,
}

impl RocksDBStorage {
    /// 创建快照
    pub fn create_snapshot(&mut self, 
        last_included_index: LogIndex,
        last_included_term: Term,
    ) -> Result<Snapshot, StorageError> {
        // 1. 创建RocksDB快照
        let db_snapshot = self.db.snapshot();
        
        // 2. 读取状态机数据
        let mut data = Vec::new();
        let iter = db_snapshot.iterator_cf(&self.cf_state, IteratorMode::Start);
        
        for item in iter {
            let (key, value) = item?;
            // 序列化KV对
            bincode::serialize_into(&mut data, &(key.to_vec(), value.to_vec()))?;
        }
        
        // 3. 计算校验和
        let checksum = crc64::crc64(0, &data);
        
        // 4. 创建元数据
        let snapshot_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        let metadata = SnapshotMetadata {
            snapshot_id,
            last_included_index,
            last_included_term,
            size: data.len() as u64,
            checksum,
            created_at: SystemTime::now(),
        };
        
        // 5. 保存快照元数据
        let key = snapshot_id.to_be_bytes();
        let value = bincode::serialize(&metadata)?;
        self.db.put_cf(&self.cf_snapshots, key, value)?;
        
        // 6. 保存快照数据到文件
        let snapshot_path = self.snapshot_file_path(snapshot_id);
        std::fs::write(snapshot_path, &data)?;
        
        Ok(Snapshot { metadata, data })
    }
    
    /// 应用快照
    pub fn apply_snapshot(&mut self, snapshot: &Snapshot) -> Result<(), StorageError> {
        // 1. 验证校验和
        let checksum = crc64::crc64(0, &snapshot.data);
        if checksum != snapshot.metadata.checksum {
            return Err(StorageError::CorruptedSnapshot);
        }
        
        // 2. 清空状态机数据
        let mut batch = WriteBatch::default();
        let iter = self.db.iterator_cf(&self.cf_state, IteratorMode::Start);
        for item in iter {
            let (key, _) = item?;
            batch.delete_cf(&self.cf_state, key);
        }
        
        // 3. 恢复快照数据
        let mut cursor = Cursor::new(&snapshot.data);
        while cursor.position() < snapshot.data.len() as u64 {
            let (key, value): (Vec<u8>, Vec<u8>) = bincode::deserialize_from(&mut cursor)?;
            batch.put_cf(&self.cf_state, key, value);
        }
        
        // 4. 应用批量写入
        self.db.write(batch)?;
        
        // 5. 更新元数据
        self.save_commit_index(snapshot.metadata.last_included_index)?;
        
        // 6. 删除旧日志
        self.delete_entries_before(snapshot.metadata.last_included_index)?;
        
        Ok(())
    }
    
    /// 快照文件路径
    fn snapshot_file_path(&self, snapshot_id: u64) -> PathBuf {
        self.config.data_dir.join(format!("snapshot-{}.dat", snapshot_id))
    }
}
```

---

## 🧪 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_storage() -> (RocksDBStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = RocksDBConfig::default();
        let storage = RocksDBStorage::open(temp_dir.path(), config).unwrap();
        (storage, temp_dir)
    }
    
    #[test]
    fn test_append_and_get_entries() {
        let (mut storage, _temp) = create_test_storage();
        
        // 追加日志
        let entries = vec![
            LogEntry::new(Term(1), LogIndex(1), vec![1, 2, 3]),
            LogEntry::new(Term(1), LogIndex(2), vec![4, 5, 6]),
        ];
        storage.append_entries(&entries).unwrap();
        
        // 读取日志
        let entry = storage.get_entry(LogIndex(1)).unwrap().unwrap();
        assert_eq!(entry.term, Term(1));
        assert_eq!(entry.data, vec![1, 2, 3]);
    }
    
    #[test]
    fn test_snapshot_create_and_apply() {
        let (mut storage, _temp) = create_test_storage();
        
        // 写入数据
        storage.put(b"key1", b"value1").unwrap();
        storage.put(b"key2", b"value2").unwrap();
        
        // 创建快照
        let snapshot = storage.create_snapshot(LogIndex(10), Term(1)).unwrap();
        
        // 清空数据
        storage.delete(b"key1").unwrap();
        storage.delete(b"key2").unwrap();
        
        // 应用快照
        storage.apply_snapshot(&snapshot).unwrap();
        
        // 验证数据恢复
        assert_eq!(storage.get(b"key1").unwrap().unwrap(), b"value1");
        assert_eq!(storage.get(b"key2").unwrap().unwrap(), b"value2");
    }
    
    #[test]
    fn test_crash_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        
        // 第一次打开，写入数据
        {
            let config = RocksDBConfig::default();
            let mut storage = RocksDBStorage::open(&path, config).unwrap();
            storage.put(b"key1", b"value1").unwrap();
            storage.sync().unwrap();
            // storage被drop，关闭数据库
        }
        
        // 第二次打开，验证数据持久化
        {
            let config = RocksDBConfig::default();
            let storage = RocksDBStorage::open(&path, config).unwrap();
            assert_eq!(storage.get(b"key1").unwrap().unwrap(), b"value1");
        }
    }
}
```

### 性能测试

```rust
#[bench]
fn bench_sequential_write(b: &mut Bencher) {
    let (mut storage, _temp) = create_test_storage();
    
    b.iter(|| {
        let entry = LogEntry::new(Term(1), LogIndex(1), vec![0; 1024]);
        black_box(storage.append_entries(&[entry]).unwrap());
    });
}

#[bench]
fn bench_random_read(b: &mut Bencher) {
    let (mut storage, _temp) = create_test_storage();
    
    // 预写入数据
    for i in 0..1000 {
        storage.put(&i.to_be_bytes(), &vec![0; 1024]).unwrap();
    }
    
    let mut rng = thread_rng();
    b.iter(|| {
        let key = rng.gen_range(0..1000).to_be_bytes();
        black_box(storage.get(&key).unwrap());
    });
}
```

---

## 📈 性能目标

| 操作 | 目标性能 | 说明 |
|------|---------|------|
| 顺序写入 | >= 50K writes/sec | 日志追加 |
| 随机读取 | >= 100K reads/sec | 状态查询 |
| 批量写入 | >= 200K writes/sec | 批量提交 |
| 快照创建 | < 5s for 1GB | 1GB数据 |
| 快照恢复 | < 10s for 1GB | 1GB数据 |
| 磁盘空间 | < 2x 数据大小 | 包括压缩 |

---

## 📝 实施计划

### Week 1: 基础实现

- [ ] Day 1-2: RocksDB配置和初始化
- [ ] Day 3-4: 日志存储实现
- [ ] Day 5: 状态存储实现
- [ ] Day 6-7: 单元测试

### Week 2: 快照和WAL

- [ ] Day 8-10: 快照机制实现
- [ ] Day 11-12: WAL配置和测试
- [ ] Day 13-14: 恢复机制测试

### Week 3: 优化和集成

- [ ] Day 15-17: 性能优化
- [ ] Day 18-19: 集成到Raft
- [ ] Day 20-21: 端到端测试

---

**文档维护者**: Storage Team  
**最后更新**: 2025年10月17日

# 时间模型

> 分布式系统中的时间概念、时钟同步和时序保证

## 目录

- [时间模型](#时间模型)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [时间类型](#时间类型)
    - [物理时钟](#物理时钟)
    - [逻辑时钟 (Lamport Clock)](#逻辑时钟-lamport-clock)
    - [向量时钟](#向量时钟)
  - [⏰ 时钟同步](#-时钟同步)
    - [NTP 同步](#ntp-同步)
    - [PTP 同步 (Precision Time Protocol)](#ptp-同步-precision-time-protocol)
  - [🕐 TrueTime 实现](#-truetime-实现)
    - [TrueTime 时钟](#truetime-时钟)
    - [外部一致性保证](#外部一致性保证)
  - [🔄 时间戳排序](#-时间戳排序)
    - [时间戳生成器](#时间戳生成器)
    - [时间戳比较](#时间戳比较)
  - [🧪 测试策略](#-测试策略)
    - [时间模型测试](#时间模型测试)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

时间在分布式系统中是一个复杂而重要的概念，涉及物理时钟、逻辑时钟、时钟同步和时序保证等多个方面。

## 🎯 核心概念

### 时间类型

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TimeType {
    Physical,  // 物理时间
    Logical,   // 逻辑时间
    Vector,    // 向量时间
    Hybrid,    // 混合时间
}

#[derive(Debug, Clone)]
pub struct TimeStamp {
    pub time_type: TimeType,
    pub value: u64,
    pub node_id: Option<String>,
    pub uncertainty: Option<Duration>,
}
```

### 物理时钟

```rust
pub struct PhysicalClock {
    node_id: String,
    clock_offset: Duration,
    clock_drift: f64,
    last_sync: Instant,
    sync_interval: Duration,
}

impl PhysicalClock {
    pub fn new(node_id: String, sync_interval: Duration) -> Self {
        Self {
            node_id,
            clock_offset: Duration::from_secs(0),
            clock_drift: 0.0,
            last_sync: Instant::now(),
            sync_interval,
        }
    }
    
    pub fn now(&self) -> SystemTime {
        let elapsed = self.last_sync.elapsed();
        let adjusted_elapsed = Duration::from_secs_f64(
            elapsed.as_secs_f64() * (1.0 + self.clock_drift)
        );
        
        SystemTime::UNIX_EPOCH + self.clock_offset + adjusted_elapsed
    }
    
    pub async fn sync_with_ntp(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 实现 NTP 同步逻辑
        let ntp_time = self.query_ntp_server().await?;
        let local_time = SystemTime::now();
        
        // 计算时钟偏移
        self.clock_offset = ntp_time.duration_since(local_time)?;
        self.last_sync = Instant::now();
        
        Ok(())
    }
    
    async fn query_ntp_server(&self) -> Result<SystemTime, Box<dyn std::error::Error>> {
        // 实现 NTP 查询逻辑
        Ok(SystemTime::now())
    }
    
    pub fn get_clock_uncertainty(&self) -> Duration {
        let elapsed = self.last_sync.elapsed();
        let drift_uncertainty = Duration::from_secs_f64(
            elapsed.as_secs_f64() * self.clock_drift.abs()
        );
        
        drift_uncertainty + Duration::from_millis(10) // 基础不确定性
    }
}
```

### 逻辑时钟 (Lamport Clock)

```rust
pub struct LamportClock {
    node_id: String,
    logical_time: u64,
    last_known_times: HashMap<String, u64>,
}

impl LamportClock {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            logical_time: 0,
            last_known_times: HashMap::new(),
        }
    }
    
    pub fn tick(&mut self) -> u64 {
        self.logical_time += 1;
        self.logical_time
    }
    
    pub fn receive_message(&mut self, sender_id: &str, message_time: u64) -> u64 {
        // 更新已知的最大时间
        let current_known = self.last_known_times.get(sender_id).unwrap_or(&0);
        if message_time > *current_known {
            self.last_known_times.insert(sender_id.to_string(), message_time);
        }
        
        // 更新本地逻辑时间
        self.logical_time = std::cmp::max(self.logical_time, message_time) + 1;
        self.logical_time
    }
    
    pub fn get_time(&self) -> u64 {
        self.logical_time
    }
    
    pub fn happens_before(&self, other: &LamportClock) -> bool {
        self.logical_time < other.logical_time
    }
    
    pub fn is_concurrent(&self, other: &LamportClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}
```

### 向量时钟

```rust
pub struct VectorClock {
    node_id: String,
    clock: HashMap<String, u64>,
    node_count: usize,
}

impl VectorClock {
    pub fn new(node_id: String, node_count: usize) -> Self {
        let mut clock = HashMap::new();
        for i in 0..node_count {
            clock.insert(format!("node_{}", i), 0);
        }
        
        Self {
            node_id,
            clock,
            node_count,
        }
    }
    
    pub fn tick(&mut self) -> u64 {
        let current = self.clock.get(&self.node_id).unwrap_or(&0);
        let new_time = current + 1;
        self.clock.insert(self.node_id.clone(), new_time);
        new_time
    }
    
    pub fn receive_message(&mut self, other_clock: &VectorClock) {
        // 更新所有节点的时间
        for (node_id, time) in &other_clock.clock {
            let current_time = self.clock.get(node_id).unwrap_or(&0);
            let new_time = std::cmp::max(*current_time, *time);
            self.clock.insert(node_id.clone(), new_time);
        }
        
        // 递增本地时钟
        self.tick();
    }
    
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;
        
        for (node_id, time) in &self.clock {
            let other_time = other.clock.get(node_id).unwrap_or(&0);
            
            if time > other_time {
                return false;
            }
            
            if time < other_time {
                strictly_less = true;
            }
        }
        
        strictly_less
    }
    
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
    
    pub fn get_clock(&self) -> &HashMap<String, u64> {
        &self.clock
    }
}
```

## ⏰ 时钟同步

### NTP 同步

```rust
pub struct NTPSynchronizer {
    ntp_servers: Vec<String>,
    sync_interval: Duration,
    max_offset: Duration,
    sync_task: Option<tokio::task::JoinHandle<()>>,
}

impl NTPSynchronizer {
    pub fn new(ntp_servers: Vec<String>, sync_interval: Duration, max_offset: Duration) -> Self {
        Self {
            ntp_servers,
            sync_interval,
            max_offset,
            sync_task: None,
        }
    }
    
    pub async fn start_sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let ntp_servers = self.ntp_servers.clone();
        let sync_interval = self.sync_interval;
        let max_offset = self.max_offset;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(sync_interval);
            
            loop {
                interval.tick().await;
                
                // 尝试与 NTP 服务器同步
                for server in &ntp_servers {
                    if let Ok(offset) = Self::sync_with_server(server).await {
                        if offset.abs() > max_offset {
                            eprintln!("Clock offset too large: {:?}", offset);
                        }
                    }
                }
            }
        });
        
        self.sync_task = Some(handle);
        Ok(())
    }
    
    async fn sync_with_server(server: &str) -> Result<Duration, Box<dyn std::error::Error>> {
        // 实现 NTP 同步逻辑
        let start_time = Instant::now();
        
        // 发送 NTP 请求
        let ntp_time = Self::query_ntp(server).await?;
        let end_time = Instant::now();
        
        // 计算网络延迟
        let network_delay = end_time.duration_since(start_time) / 2;
        
        // 计算时钟偏移
        let local_time = SystemTime::now();
        let offset = ntp_time.duration_since(local_time)? - network_delay;
        
        Ok(offset)
    }
    
    async fn query_ntp(server: &str) -> Result<SystemTime, Box<dyn std::error::Error>> {
        // 实现 NTP 查询逻辑
        Ok(SystemTime::now())
    }
}
```

### PTP 同步 (Precision Time Protocol)

```rust
pub struct PTP synchronizer {
    ptp_domain: u8,
    sync_interval: Duration,
    announce_interval: Duration,
    sync_task: Option<tokio::task::JoinHandle<()>>,
}

impl PTP synchronizer {
    pub fn new(ptp_domain: u8, sync_interval: Duration, announce_interval: Duration) -> Self {
        Self {
            ptp_domain,
            sync_interval,
            announce_interval,
            sync_task: None,
        }
    }
    
    pub async fn start_sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let ptp_domain = self.ptp_domain;
        let sync_interval = self.sync_interval;
        let announce_interval = self.announce_interval;
        
        let handle = tokio::spawn(async move {
            let mut sync_interval_timer = tokio::time::interval(sync_interval);
            let mut announce_interval_timer = tokio::time::interval(announce_interval);
            
            loop {
                tokio::select! {
                    _ = sync_interval_timer.tick() => {
                        Self::send_sync_message(ptp_domain).await;
                    }
                    _ = announce_interval_timer.tick() => {
                        Self::send_announce_message(ptp_domain).await;
                    }
                }
            }
        });
        
        self.sync_task = Some(handle);
        Ok(())
    }
    
    async fn send_sync_message(ptp_domain: u8) {
        // 实现 PTP Sync 消息发送
    }
    
    async fn send_announce_message(ptp_domain: u8) {
        // 实现 PTP Announce 消息发送
    }
}
```

## 🕐 TrueTime 实现

### TrueTime 时钟

```rust
pub struct TrueTimeClock {
    node_id: String,
    epsilon: Duration,
    last_sync: Instant,
    sync_interval: Duration,
}

impl TrueTimeClock {
    pub fn new(node_id: String, epsilon: Duration, sync_interval: Duration) -> Self {
        Self {
            node_id,
            epsilon,
            last_sync: Instant::now(),
            sync_interval,
        }
    }
    
    pub fn now(&self) -> TrueTimeInterval {
        let current_time = SystemTime::now();
        let elapsed = self.last_sync.elapsed();
        
        // 计算时间不确定性
        let uncertainty = self.epsilon + elapsed;
        
        TrueTimeInterval {
            earliest: current_time - uncertainty,
            latest: current_time + uncertainty,
        }
    }
    
    pub async fn sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 实现 TrueTime 同步逻辑
        self.last_sync = Instant::now();
        Ok(())
    }
    
    pub fn get_epsilon(&self) -> Duration {
        self.epsilon
    }
}

#[derive(Debug, Clone)]
pub struct TrueTimeInterval {
    pub earliest: SystemTime,
    pub latest: SystemTime,
}

impl TrueTimeInterval {
    pub fn contains(&self, time: SystemTime) -> bool {
        time >= self.earliest && time <= self.latest
    }
    
    pub fn is_before(&self, other: &TrueTimeInterval) -> bool {
        self.latest < other.earliest
    }
    
    pub fn is_after(&self, other: &TrueTimeInterval) -> bool {
        self.earliest > other.latest
    }
    
    pub fn overlaps(&self, other: &TrueTimeInterval) -> bool {
        !self.is_before(other) && !self.is_after(other)
    }
}
```

### 外部一致性保证

```rust
pub struct ExternalConsistencyManager {
    true_time_clock: TrueTimeClock,
    commit_wait_time: Duration,
}

impl ExternalConsistencyManager {
    pub fn new(true_time_clock: TrueTimeClock, commit_wait_time: Duration) -> Self {
        Self {
            true_time_clock,
            commit_wait_time,
        }
    }
    
    pub async fn commit_with_external_consistency(
        &mut self,
        transaction: &Transaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 获取提交时间
        let commit_time = self.true_time_clock.now();
        
        // 2. 等待确保外部一致性
        let wait_time = self.calculate_wait_time(&commit_time);
        tokio::time::sleep(wait_time).await;
        
        // 3. 提交事务
        transaction.commit().await?;
        
        Ok(())
    }
    
    fn calculate_wait_time(&self, commit_time: &TrueTimeInterval) -> Duration {
        let now = SystemTime::now();
        let latest_commit_time = commit_time.latest;
        
        if now < latest_commit_time {
            latest_commit_time.duration_since(now).unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        }
    }
    
    pub async fn read_with_external_consistency(
        &self,
        key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // 1. 获取读取时间
        let read_time = self.true_time_clock.now();
        
        // 2. 等待确保外部一致性
        let wait_time = self.calculate_wait_time(&read_time);
        tokio::time::sleep(wait_time).await;
        
        // 3. 执行读取
        Ok(self.read_from_storage(key).await?)
    }
    
    async fn read_from_storage(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // 实现存储读取逻辑
        Ok(None)
    }
}
```

## 🔄 时间戳排序

### 时间戳生成器

```rust
pub struct TimestampGenerator {
    node_id: String,
    sequence: u64,
    last_timestamp: u64,
    clock: Box<dyn Clock>,
}

pub trait Clock {
    fn now(&self) -> u64;
}

impl TimestampGenerator {
    pub fn new(node_id: String, clock: Box<dyn Clock>) -> Self {
        Self {
            node_id,
            sequence: 0,
            last_timestamp: 0,
            clock,
        }
    }
    
    pub fn generate_timestamp(&mut self) -> Timestamp {
        let current_time = self.clock.now();
        
        if current_time > self.last_timestamp {
            self.last_timestamp = current_time;
            self.sequence = 0;
        } else {
            self.sequence += 1;
        }
        
        Timestamp {
            time: self.last_timestamp,
            sequence: self.sequence,
            node_id: self.node_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    pub time: u64,
    pub sequence: u64,
    pub node_id: String,
}
```

### 时间戳比较

```rust
impl Timestamp {
    pub fn happens_before(&self, other: &Timestamp) -> bool {
        if self.time < other.time {
            true
        } else if self.time == other.time {
            if self.node_id < other.node_id {
                true
            } else if self.node_id == other.node_id {
                self.sequence < other.sequence
            } else {
                false
            }
        } else {
            false
        }
    }
    
    pub fn is_concurrent(&self, other: &Timestamp) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}
```

## 🧪 测试策略

### 时间模型测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lamport_clock() {
        let mut clock1 = LamportClock::new("node1".to_string());
        let mut clock2 = LamportClock::new("node2".to_string());
        
        // 本地事件
        let time1 = clock1.tick();
        let time2 = clock2.tick();
        
        assert_eq!(time1, 1);
        assert_eq!(time2, 1);
        
        // 接收消息
        let time3 = clock2.receive_message("node1", time1);
        assert_eq!(time3, 2);
        
        // 验证因果关系
        assert!(clock1.happens_before(&clock2));
    }
    
    #[test]
    fn test_vector_clock() {
        let mut vc1 = VectorClock::new("node1".to_string(), 3);
        let mut vc2 = VectorClock::new("node2".to_string(), 3);
        
        // 本地事件
        vc1.tick();
        vc2.tick();
        
        // 发送消息
        vc2.receive_message(&vc1);
        
        // 验证因果关系
        assert!(vc1.happens_before(&vc2));
        assert!(!vc2.happens_before(&vc1));
    }
    
    #[test]
    fn test_true_time_interval() {
        let now = SystemTime::now();
        let epsilon = Duration::from_millis(10);
        
        let interval = TrueTimeInterval {
            earliest: now - epsilon,
            latest: now + epsilon,
        };
        
        assert!(interval.contains(now));
        assert!(!interval.contains(now + Duration::from_millis(20)));
    }
    
    #[tokio::test]
    async fn test_timestamp_generator() {
        struct MockClock {
            time: u64,
        }
        
        impl Clock for MockClock {
            fn now(&self) -> u64 {
                self.time
            }
        }
        
        let mut generator = TimestampGenerator::new(
            "node1".to_string(),
            Box::new(MockClock { time: 1000 }),
        );
        
        let timestamp1 = generator.generate_timestamp();
        let timestamp2 = generator.generate_timestamp();
        
        assert!(timestamp1.happens_before(&timestamp2));
    }
}
```

## 📚 进一步阅读

- [一致性模型](../consistency/README.md) - 时间与一致性关系
- [共识机制](../consensus/README.md) - 时间在共识中的作用
- [故障处理](../failure/README.md) - 时间相关的故障
- [性能优化](../performance/OPTIMIZATION.md) - 时间性能优化

## 🔗 相关文档

- [一致性模型](../consistency/README.md)
- [共识机制](../consensus/README.md)
- [故障处理](../failure/README.md)
- [性能优化](../performance/OPTIMIZATION.md)
- [测试策略](../testing/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// 分布式系统一致性级别
///
/// 基于CAP定理和PACELC定理，提供不同的一致性保证
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum ConsistencyLevel {
    /// 强一致性 - 所有节点立即看到相同的数据
    Strong,
    /// 线性一致性 - 所有操作都有全局顺序
    Linearizable,
    /// 顺序一致性 - 每个节点的操作保持顺序
    Sequential,
    /// 因果一致性 - 保持因果关系
    Causal,
    /// 最终一致性 - 最终所有节点会收敛到相同状态
    #[default]
    Eventual,
    /// 会话一致性 - 在同一会话内保持一致性
    Session,
    /// 单调读一致性 - 读操作不会返回比之前更旧的数据
    MonotonicRead,
    /// 单调写一致性 - 写操作按顺序执行
    MonotonicWrite,
    /// 法定人数一致性 - 基于多数节点的确认
    Quorum,
    /// 读己写一致性 - 读操作能看到自己之前的写操作
    ReadYourWrites,
    /// 单调读一致性 - 读操作不会返回比之前更旧的数据
    MonotonicReads,
    /// 单调写一致性 - 写操作按顺序执行
    MonotonicWrites,
    /// 写后读一致性 - 写操作后立即的读操作能看到该写操作
    WritesFollowReads,
    /// 因果一致性 - 保持因果关系
    CausalConsistency,
    /// 强最终一致性 - 在最终一致性基础上提供更强保证
    StrongEventual,
}


impl ConsistencyLevel {
    /// 获取一致性级别的描述
    pub fn description(&self) -> &'static str {
        match self {
            ConsistencyLevel::Strong => "强一致性：所有节点立即看到相同的数据",
            ConsistencyLevel::Linearizable => "线性一致性：所有操作都有全局顺序",
            ConsistencyLevel::Sequential => "顺序一致性：每个节点的操作保持顺序",
            ConsistencyLevel::Causal => "因果一致性：保持因果关系",
            ConsistencyLevel::Eventual => "最终一致性：最终所有节点会收敛到相同状态",
            ConsistencyLevel::Session => "会话一致性：在同一会话内保持一致性",
            ConsistencyLevel::MonotonicRead => "单调读一致性：读操作不会返回比之前更旧的数据",
            ConsistencyLevel::MonotonicWrite => "单调写一致性：写操作按顺序执行",
            ConsistencyLevel::Quorum => "法定人数一致性：基于多数节点的确认",
            ConsistencyLevel::ReadYourWrites => "读己写一致性：读操作能看到自己之前的写操作",
            ConsistencyLevel::MonotonicReads => "单调读一致性：读操作不会返回比之前更旧的数据",
            ConsistencyLevel::MonotonicWrites => "单调写一致性：写操作按顺序执行",
            ConsistencyLevel::WritesFollowReads => "写后读一致性：写操作后立即的读操作能看到该写操作",
            ConsistencyLevel::CausalConsistency => "因果一致性：保持因果关系",
            ConsistencyLevel::StrongEventual => "强最终一致性：在最终一致性基础上提供更强保证",
        }
    }

    /// 检查是否支持分区容忍性
    pub fn supports_partition_tolerance(&self) -> bool {
        match self {
            ConsistencyLevel::Strong | ConsistencyLevel::Linearizable => false,
            _ => true,
        }
    }

    /// 检查是否支持高可用性
    pub fn supports_high_availability(&self) -> bool {
        match self {
            ConsistencyLevel::Strong | ConsistencyLevel::Linearizable => false,
            _ => true,
        }
    }

    /// 获取一致性级别的强度（数值越大越强）
    pub fn strength(&self) -> u8 {
        match self {
            ConsistencyLevel::Strong => 10,
            ConsistencyLevel::Linearizable => 9,
            ConsistencyLevel::Sequential => 8,
            ConsistencyLevel::CausalConsistency => 7,
            ConsistencyLevel::Causal => 6,
            ConsistencyLevel::Session => 5,
            ConsistencyLevel::ReadYourWrites => 4,
            ConsistencyLevel::MonotonicReads | ConsistencyLevel::MonotonicRead => 3,
            ConsistencyLevel::MonotonicWrites | ConsistencyLevel::MonotonicWrite => 3,
            ConsistencyLevel::WritesFollowReads => 2,
            ConsistencyLevel::Quorum => 2,
            ConsistencyLevel::StrongEventual => 1,
            ConsistencyLevel::Eventual => 0,
        }
    }

    /// 检查两个一致性级别是否兼容
    pub fn is_compatible_with(&self, other: &ConsistencyLevel) -> bool {
        // 强一致性级别不能与弱一致性级别混合
        match (self, other) {
            (ConsistencyLevel::Strong, ConsistencyLevel::Eventual) => false,
            (ConsistencyLevel::Eventual, ConsistencyLevel::Strong) => false,
            (ConsistencyLevel::Linearizable, ConsistencyLevel::Eventual) => false,
            (ConsistencyLevel::Eventual, ConsistencyLevel::Linearizable) => false,
            _ => true,
        }
    }
}

/// CAP定理权衡策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum CAPStrategy {
    /// 一致性优先 (CP) - 牺牲可用性保证一致性
    ConsistencyPartition,
    /// 可用性优先 (AP) - 牺牲一致性保证可用性
    AvailabilityPartition,
    /// 平衡策略 - 在一致性和可用性之间动态权衡
    #[default]
    Balanced,
}


impl CAPStrategy {
    /// 根据网络分区状态选择一致性级别
    pub fn select_consistency_level(&self, is_partitioned: bool) -> ConsistencyLevel {
        match (self, is_partitioned) {
            (CAPStrategy::ConsistencyPartition, _) => ConsistencyLevel::Strong,
            (CAPStrategy::AvailabilityPartition, _) => ConsistencyLevel::Eventual,
            (CAPStrategy::Balanced, true) => ConsistencyLevel::Causal,
            (CAPStrategy::Balanced, false) => ConsistencyLevel::Linearizable,
        }
    }

    /// 获取策略描述
    pub fn description(&self) -> &'static str {
        match self {
            CAPStrategy::ConsistencyPartition => "一致性优先 (CP) - 牺牲可用性保证一致性",
            CAPStrategy::AvailabilityPartition => "可用性优先 (AP) - 牺牲一致性保证可用性",
            CAPStrategy::Balanced => "平衡策略 - 在一致性和可用性之间动态权衡",
        }
    }

    /// 获取适用场景
    pub fn use_case(&self) -> &'static str {
        match self {
            CAPStrategy::ConsistencyPartition => "金融系统、关键业务数据存储",
            CAPStrategy::AvailabilityPartition => "社交媒体、内容分发网络",
            CAPStrategy::Balanced => "一般业务系统、混合工作负载",
        }
    }

    /// 获取权衡说明
    pub fn trade_off(&self) -> &'static str {
        match self {
            CAPStrategy::ConsistencyPartition => "强一致性但可能牺牲可用性",
            CAPStrategy::AvailabilityPartition => "高可用性但可能牺牲一致性",
            CAPStrategy::Balanced => "根据网络状态动态调整一致性和可用性",
        }
    }
}

/// 因果一致性向量时钟
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    clocks: HashMap<String, u64>,
}

impl VectorClock {
    /// 创建新的向量时钟
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// 为指定节点增加时钟值
    pub fn increment(&mut self, node_id: &str) {
        let entry = self.clocks.entry(node_id.to_string()).or_insert(0);
        *entry += 1;
    }

    /// 更新向量时钟（取最大值）
    pub fn update(&mut self, other: &VectorClock) {
        for (node_id, clock_value) in &other.clocks {
            let entry = self.clocks.entry(node_id.clone()).or_insert(0);
            *entry = (*entry).max(*clock_value);
        }
    }

    /// 检查是否在因果关系上先于另一个向量时钟
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;

        for (node_id, &self_clock) in &self.clocks {
            let other_clock = other.clocks.get(node_id).unwrap_or(&0);
            if self_clock > *other_clock {
                return false;
            }
            if self_clock < *other_clock {
                strictly_less = true;
            }
        }

        // 检查是否有其他节点在other中存在但在self中不存在
        for node_id in other.clocks.keys() {
            if !self.clocks.contains_key(node_id) {
                strictly_less = true;
            }
        }

        strictly_less
    }

    /// 检查是否并发（既不在前也不在后，且不相等）
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self) && !self.is_equal(other)
    }

    /// 检查两个向量时钟是否相等
    pub fn is_equal(&self, other: &VectorClock) -> bool {
        if self.clocks.len() != other.clocks.len() {
            return false;
        }

        for (node_id, &self_clock) in &self.clocks {
            let other_clock = other.clocks.get(node_id).unwrap_or(&0);
            if self_clock != *other_clock {
                return false;
            }
        }

        true
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// 会话一致性管理器
#[derive(Debug, Clone)]
pub struct SessionConsistencyManager {
    sessions: HashMap<String, SessionInfo>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    last_read_version: Option<VectorClock>,
    last_write_version: Option<VectorClock>,
    created_at: SystemTime,
}

impl SessionConsistencyManager {
    /// 创建新的会话管理器
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// 创建新会话
    pub fn create_session(&mut self, session_id: String) {
        self.sessions.insert(
            session_id,
            SessionInfo {
                last_read_version: None,
                last_write_version: None,
                created_at: SystemTime::now(),
            },
        );
    }

    /// 更新会话的读版本
    pub fn update_read_version(&mut self, session_id: &str, version: VectorClock) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.last_read_version = Some(version);
        }
    }

    /// 更新会话的写版本
    pub fn update_write_version(&mut self, session_id: &str, version: VectorClock) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.last_write_version = Some(version);
        }
    }

    /// 检查读操作是否满足会话一致性
    pub fn can_read(&self, session_id: &str, current_version: &VectorClock) -> bool {
        if let Some(session) = self.sessions.get(session_id)
            && let Some(ref last_read) = session.last_read_version {
                return !current_version.happens_before(last_read);
            }
        true
    }

    /// 检查写操作是否满足会话一致性
    pub fn can_write(&self, session_id: &str, current_version: &VectorClock) -> bool {
        if let Some(session) = self.sessions.get(session_id)
            && let Some(ref last_write) = session.last_write_version {
                return !current_version.happens_before(last_write);
            }
        true
    }

    /// 清理过期会话
    pub fn cleanup_expired_sessions(&mut self, max_age: Duration) {
        let now = SystemTime::now();
        self.sessions.retain(|_, session| {
            now.duration_since(session.created_at)
                .unwrap_or(Duration::from_secs(0))
                < max_age
        });
    }
}

impl Default for SessionConsistencyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 单调一致性管理器
#[derive(Debug, Clone)]
pub struct MonotonicConsistencyManager {
    read_versions: HashMap<String, VectorClock>,
    write_versions: HashMap<String, VectorClock>,
}

impl MonotonicConsistencyManager {
    /// 创建新的单调一致性管理器
    pub fn new() -> Self {
        Self {
            read_versions: HashMap::new(),
            write_versions: HashMap::new(),
        }
    }

    /// 检查单调读一致性
    pub fn check_monotonic_read(&mut self, client_id: &str, version: &VectorClock) -> bool {
        if let Some(last_read) = self.read_versions.get(client_id)
            && version.happens_before(last_read) {
                return false; // 违反了单调读一致性
            }
        self.read_versions
            .insert(client_id.to_string(), version.clone());
        true
    }

    /// 检查单调写一致性
    pub fn check_monotonic_write(&mut self, client_id: &str, version: &VectorClock) -> bool {
        if let Some(last_write) = self.write_versions.get(client_id)
            && version.happens_before(last_write) {
                return false; // 违反了单调写一致性
            }
        self.write_versions
            .insert(client_id.to_string(), version.clone());
        true
    }
}

impl Default for MonotonicConsistencyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 高级一致性管理器
/// 支持多种一致性模型的组合和动态切换
#[derive(Debug, Clone)]
pub struct AdvancedConsistencyManager {
    session_manager: SessionConsistencyManager,
    monotonic_manager: MonotonicConsistencyManager,
    current_level: ConsistencyLevel,
    client_sessions: HashMap<String, String>, // client_id -> session_id
    read_barriers: HashMap<String, VectorClock>, // 读屏障
    write_barriers: HashMap<String, VectorClock>, // 写屏障
}

impl AdvancedConsistencyManager {
    /// 创建新的高级一致性管理器
    pub fn new(initial_level: ConsistencyLevel) -> Self {
        Self {
            session_manager: SessionConsistencyManager::new(),
            monotonic_manager: MonotonicConsistencyManager::new(),
            current_level: initial_level,
            client_sessions: HashMap::new(),
            read_barriers: HashMap::new(),
            write_barriers: HashMap::new(),
        }
    }

    /// 设置一致性级别
    pub fn set_consistency_level(&mut self, level: ConsistencyLevel) {
        self.current_level = level;
    }

    /// 获取当前一致性级别
    pub fn get_consistency_level(&self) -> ConsistencyLevel {
        self.current_level
    }

    /// 为客户端创建会话
    pub fn create_client_session(&mut self, client_id: String) -> String {
        let session_id = format!("session_{}_{}", client_id, 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_millis());
        self.client_sessions.insert(client_id.clone(), session_id.clone());
        self.session_manager.create_session(session_id.clone());
        session_id
    }

    /// 检查读操作的一致性
    pub fn check_read_consistency(&mut self, client_id: &str, version: &VectorClock) -> bool {
        match self.current_level {
            ConsistencyLevel::Strong | ConsistencyLevel::Linearizable => {
                // 强一致性：检查读屏障
                if let Some(barrier) = self.read_barriers.get(client_id) {
                    return !version.happens_before(barrier);
                }
                true
            },
            ConsistencyLevel::Session => {
                if let Some(session_id) = self.client_sessions.get(client_id) {
                    self.session_manager.can_read(session_id, version)
                } else {
                    true
                }
            },
            ConsistencyLevel::MonotonicRead | ConsistencyLevel::MonotonicReads => {
                self.monotonic_manager.check_monotonic_read(client_id, version)
            },
            ConsistencyLevel::ReadYourWrites => {
                // 读己写一致性：检查写屏障
                if let Some(barrier) = self.write_barriers.get(client_id) {
                    return !version.happens_before(barrier);
                }
                true
            },
            _ => true, // 其他级别暂时返回true
        }
    }

    /// 检查写操作的一致性
    pub fn check_write_consistency(&mut self, client_id: &str, version: &VectorClock) -> bool {
        match self.current_level {
            ConsistencyLevel::Strong | ConsistencyLevel::Linearizable => {
                // 强一致性：检查写屏障
                if let Some(barrier) = self.write_barriers.get(client_id) {
                    return !version.happens_before(barrier);
                }
                true
            },
            ConsistencyLevel::Session => {
                if let Some(session_id) = self.client_sessions.get(client_id) {
                    self.session_manager.can_write(session_id, version)
                } else {
                    true
                }
            },
            ConsistencyLevel::MonotonicWrite | ConsistencyLevel::MonotonicWrites => {
                self.monotonic_manager.check_monotonic_write(client_id, version)
            },
            _ => true, // 其他级别暂时返回true
        }
    }

    /// 更新读屏障
    pub fn update_read_barrier(&mut self, client_id: &str, version: VectorClock) {
        self.read_barriers.insert(client_id.to_string(), version);
    }

    /// 更新写屏障
    pub fn update_write_barrier(&mut self, client_id: &str, version: VectorClock) {
        self.write_barriers.insert(client_id.to_string(), version);
    }

    /// 清理过期数据
    pub fn cleanup(&mut self, max_age: Duration) {
        self.session_manager.cleanup_expired_sessions(max_age);
        // 清理过期的屏障（简化实现）
        self.read_barriers.clear();
        self.write_barriers.clear();
    }

    /// 获取一致性统计信息
    pub fn get_stats(&self) -> ConsistencyStats {
        ConsistencyStats {
            active_sessions: self.client_sessions.len(),
            read_barriers: self.read_barriers.len(),
            write_barriers: self.write_barriers.len(),
            current_level: self.current_level,
        }
    }
}

/// 一致性统计信息
#[derive(Debug, Clone)]
pub struct ConsistencyStats {
    pub active_sessions: usize,
    pub read_barriers: usize,
    pub write_barriers: usize,
    pub current_level: ConsistencyLevel,
}

impl Default for AdvancedConsistencyManager {
    fn default() -> Self {
        Self::new(ConsistencyLevel::Eventual)
    }
}

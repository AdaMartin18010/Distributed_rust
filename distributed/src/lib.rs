//! c20_distributed - 分布式系统库
//!
//! 基于 Rust 1.89 的现代分布式系统基础设施，提供完整的分布式系统能力。
//!
//! ## 模块结构
//!
//! - **core**: 核心模块 - 配置、错误处理、成员管理、拓扑等基础组件
//! - **consensus**: 共识算法 - Raft、Paxos、拜占庭容错等
//! - **consistency**: 一致性模型 - 各种一致性级别和管理器
//! - **network**: 网络通信 - RPC、连接池、分布式锁等
//! - **storage**: 存储抽象 - 日志存储、复制、分区等
//! - **monitoring**: 监控系统 - 指标收集、健康检查、性能监控
//! - **security**: 安全模块 - 访问控制、限流、熔断器等
//!
//! ## 快速开始
//!
//! ```rust
//! use c20_distributed::core::DistributedConfig;
//! use c20_distributed::network::DistributedMutex;
//! 
//! let config = DistributedConfig::default();
//! // 使用分布式系统组件...
//! ```

// 核心模块
pub mod core;

// 共识算法模块
pub mod consensus;

// 一致性模块
pub mod consistency;

// 网络通信模块
pub mod network;

// 存储模块
pub mod storage;

// 监控模块
pub mod monitoring;

// 安全模块
pub mod security;

// 示例和基准测试模块
pub mod examples;
pub mod benchmarks;

// 其他实用模块
pub mod cap_theorem;
pub mod chaos;
pub mod codec;
pub mod config_management;
pub mod load_balancing;
pub mod partitioning;
pub mod service_discovery;
pub mod swim;
pub mod transactions;

// 重新导出核心类型以保持向后兼容
pub use core::{DistributedConfig, DistributedError, ClusterMembership, ClusterNodeId, ClusterTopology, ShardId, LogicalClock, TimerService};

// 重新导出共识相关类型（保持向后兼容的模块名）
pub use consensus::raft as consensus_raft;
pub use consensus::raft::*;
pub use consensus::paxos::*;
pub use consensus::byzantine::*;

// 为向后兼容重新导出
pub use core::topology;
pub use storage::replication;
pub use network as transport;

// 重新导出一致性相关类型
pub use consistency::{
    AdvancedConsistencyManager, CAPStrategy, ConsistencyLevel, ConsistencyStats,
    MonotonicConsistencyManager, SessionConsistencyManager, VectorClock,
};

// 重新导出网络相关类型
pub use network::{
    BatchRpcRequest, BatchRpcResponse, ConnectionInfo, ConnectionPool, ConnectionPoolConfig,
    InMemoryRpcClient, InMemoryRpcServer, RetryClient, RetryPolicy, RpcClient,
    RpcRequest, RpcResponse, RpcServer,
};

#[cfg(feature = "runtime-tokio")]
pub use network::RequestBatcher;

// 重新导出分布式锁相关类型
pub use network::distributed_lock::{
    DistributedLockManager, DistributedMutex, DistributedRwLock, DistributedSemaphore,
    LockInfo, LockMode, LockRequest, LockResponse, LockState, LockType,
};

// 重新导出存储相关类型
pub use storage::{LogStorage, StateMachineStorage};
pub use storage::replication::{MajorityQuorum, QuorumPolicy, Replicator};

// 重新导出监控相关类型
pub use monitoring::{
    Counter, Gauge, HealthCheck, HealthStatus, Histogram, Metric, MetricCollector,
    MetricImpl, MetricLabels, MetricRegistry, MetricType, MetricValue, PerformanceMonitor,
    SystemHealthChecker,
};

// 重新导出安全相关类型
pub use security::{
    AclManager, AclRule, Action, AuditEvent, Auditor, CircuitBreaker, CircuitConfig, CircuitState,
    Governance, Principal, RateLimitConfig, Resource, TokenBucket,
};

// 重新导出其他实用类型
pub use cap_theorem::{
    CAPAnalysisReport, CAPAnalyzer, CAPManager, ConsistencyDecision, PartitionDetector,
    PartitionStats, PerformanceMetrics,
};
pub use chaos::{ChaosConfig, ChaosInjector};
pub use codec::{BinaryCodec, BytesCodec, StringUtf8Codec};
pub use config_management::{
    ConfigManager, ConfigSnapshot, ConfigSource, ConfigValue, EnvSource, FileSource, InMemorySource,
};
pub use load_balancing::{
    ConsistentHashBalancer, GeographicBalancer, LeastConnectionsBalancer,
    LeastResponseTimeBalancer, LoadBalancerManager, LoadBalancingStrategy, RandomBalancer,
    RoundRobinBalancer, ServerStats, WeightedRandomBalancer, WeightedRoundRobinBalancer,
};
pub use partitioning::{HashPartitioner, Partitioner};
pub use service_discovery::{
    ConfigServiceDiscovery, DiscoveryStrategy, DnsServiceDiscovery,
    RegistryServiceDiscovery, ServiceDiscoveryConfig, ServiceDiscoveryManager, ServiceInstance,
};
pub use swim::{
    EnhancedSwimTransport, MembershipView, SwimEvent, SwimMemberState, SwimNode, SwimTransport,
};
pub use transactions::{Saga, SagaStep};
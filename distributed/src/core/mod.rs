//! 核心模块
//!
//! 范围与目标：
//! - 汇聚公共抽象（配置、错误、成员关系、拓扑、调度），供其余子系统复用。
//! - 确保类型稳定性与向后兼容，作为外部集成的稳定入口。

pub mod config;
pub mod errors;
pub mod membership;
pub mod topology;
pub mod scheduling;

pub use config::DistributedConfig;
pub use errors::DistributedError;
pub use membership::{ClusterMembership, ClusterNodeId};
pub use topology::{ClusterTopology, ShardId};
pub use scheduling::{LogicalClock, TimerService};

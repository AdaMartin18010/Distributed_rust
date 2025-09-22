//! 核心模块
//! 
//! 提供分布式系统的核心抽象和基础组件

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

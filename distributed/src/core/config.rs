//! 分布式系统配置模块

/// 分布式系统配置
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    pub nodes: Vec<String>,
    pub replication_factor: usize,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            replication_factor: 3,
        }
    }
}

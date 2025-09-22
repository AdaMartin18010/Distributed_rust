//! 混沌注入模块
//!
//! 提供可配置的延迟、丢包、抖动、网络分区等模拟能力，便于在本地与测试环境复现故障场景。

use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChaosConfig {
    /// 固定延迟（毫秒）
    pub latency_ms: u64,
    /// 随机丢弃概率（0.0~1.0）
    pub drop_rate: f64,
    /// 抖动（±毫秒）
    pub jitter_ms: u64,
    /// 是否启用网络分区模拟
    pub partition_enabled: bool,
    /// 分区集合（例如：与这些节点之间视为断联）
    pub partition_peers: HashSet<String>,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            drop_rate: 0.0,
            jitter_ms: 0,
            partition_enabled: false,
            partition_peers: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChaosInjector {
    cfg: ChaosConfig,
}

impl ChaosInjector {
    pub fn new(cfg: ChaosConfig) -> Self {
        Self { cfg }
    }

    pub fn update(&mut self, cfg: ChaosConfig) {
        self.cfg = cfg;
    }

    /// 根据配置注入延迟（含抖动）
    pub fn inject_latency(&self) {
        if self.cfg.latency_ms == 0 && self.cfg.jitter_ms == 0 {
            return;
        }
        let base = self.cfg.latency_ms as i64;
        let jitter = if self.cfg.jitter_ms == 0 {
            0
        } else {
            Instant::now().elapsed().as_nanos() as i64 % (self.cfg.jitter_ms as i64 + 1)
        };
        let dur_ms = (base + jitter).max(0) as u64;
        thread::sleep(Duration::from_millis(dur_ms));
    }

    /// 根据概率决定是否丢弃
    pub fn should_drop(&self) -> bool {
        if self.cfg.drop_rate <= 0.0 {
            return false;
        }
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        Instant::now().hash(&mut h);
        let r = (h.finish() % 10_000) as f64 / 10_000.0;
        r < self.cfg.drop_rate
    }

    /// 判断是否与目标节点处于“分区”关系
    pub fn is_partitioned_with(&self, peer: &str) -> bool {
        self.cfg.partition_enabled && self.cfg.partition_peers.contains(peer)
    }
}

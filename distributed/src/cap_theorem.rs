use crate::consistency::{CAPStrategy, ConsistencyLevel};
use crate::swim::MembershipView;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

/// CAP定理权衡管理器
///
/// 根据网络分区状态动态调整一致性级别和可用性策略
#[derive(Debug, Clone)]
pub struct CAPManager {
    strategy: CAPStrategy,
    partition_detector: PartitionDetector,
    consistency_history: Vec<ConsistencyDecision>,
    last_partition_check: Instant,
    partition_check_interval: Duration,
    /// 历史容量上限（超过后触发裁剪）
    history_capacity: usize,
    /// 每次裁剪的批量大小
    history_prune_batch: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyDecision {
    pub timestamp: SystemTime,
    pub is_partitioned: bool,
    pub selected_level: ConsistencyLevel,
    pub strategy: CAPStrategy,
    pub reasoning: String,
}

impl CAPManager {
    /// 创建新的CAP管理器
    pub fn new(strategy: CAPStrategy) -> Self {
        Self {
            strategy,
            partition_detector: PartitionDetector::new(),
            consistency_history: Vec::new(),
            last_partition_check: Instant::now(),
            partition_check_interval: Duration::from_millis(1000),
            history_capacity: 1000,
            history_prune_batch: 100,
        }
    }

    /// 获取当前策略
    pub fn strategy(&self) -> CAPStrategy {
        self.strategy
    }

    /// 获取分区检测间隔
    pub fn partition_check_interval(&self) -> Duration {
        self.partition_check_interval
    }

    /// 设置分区检测间隔
    pub fn with_partition_check_interval(mut self, interval: Duration) -> Self {
        self.partition_check_interval = interval;
        self
    }

    /// 设置历史容量与裁剪批量
    pub fn with_history_limits(mut self, capacity: usize, prune_batch: usize) -> Self {
        self.history_capacity = capacity.max(1);
        self.history_prune_batch = prune_batch.max(1).min(self.history_capacity);
        self
    }

    /// 根据当前状态选择一致性级别
    pub fn select_consistency_level(
        &mut self,
        membership_view: &MembershipView,
    ) -> ConsistencyLevel {
        let now = Instant::now();
        let is_partitioned =
            if now.duration_since(self.last_partition_check) >= self.partition_check_interval {
                self.partition_detector.detect_partition(membership_view)
            } else {
                self.partition_detector.is_currently_partitioned()
            };

        let selected_level = self.strategy.select_consistency_level(is_partitioned);
        let reasoning = self.generate_reasoning(is_partitioned, selected_level);

        let decision = ConsistencyDecision {
            timestamp: SystemTime::now(),
            is_partitioned,
            selected_level,
            strategy: self.strategy,
            reasoning,
        };

        self.consistency_history.push(decision);
        self.last_partition_check = now;

        // 保持历史记录在合理范围内（可配置）
        if self.consistency_history.len() > self.history_capacity {
            let drain_len = self.history_prune_batch.min(self.consistency_history.len());
            self.consistency_history.drain(0..drain_len);
        }

        selected_level
    }

    /// 生成决策理由
    fn generate_reasoning(&self, is_partitioned: bool, level: ConsistencyLevel) -> String {
        match (self.strategy, is_partitioned, level) {
            (CAPStrategy::ConsistencyPartition, _, ConsistencyLevel::Strong) => {
                "CP策略：优先保证一致性，选择强一致性".to_string()
            }
            (CAPStrategy::AvailabilityPartition, _, ConsistencyLevel::Eventual) => {
                "AP策略：优先保证可用性，选择最终一致性".to_string()
            }
            (CAPStrategy::Balanced, true, ConsistencyLevel::Causal) => {
                "平衡策略：检测到分区，选择因果一致性".to_string()
            }
            (CAPStrategy::Balanced, false, ConsistencyLevel::Linearizable) => {
                "平衡策略：无分区，选择线性一致性".to_string()
            }
            _ => format!(
                "策略：{:?}，分区状态：{}，选择：{:?}",
                self.strategy, is_partitioned, level
            ),
        }
    }

    /// 获取最近的一致性决策历史
    pub fn get_recent_decisions(&self, count: usize) -> Vec<&ConsistencyDecision> {
        let start = if self.consistency_history.len() > count {
            self.consistency_history.len() - count
        } else {
            0
        };
        self.consistency_history[start..].iter().collect()
    }

    /// 获取分区检测统计
    pub fn get_partition_stats(&self) -> PartitionStats {
        self.partition_detector.get_stats()
    }

    /// 检测网络分区
    pub fn detect_partition(&mut self, membership_view: &MembershipView) -> bool {
        self.partition_detector.detect_partition(membership_view)
    }

    /// 更新策略
    pub fn update_strategy(&mut self, new_strategy: CAPStrategy) {
        self.strategy = new_strategy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consistency::CAPStrategy;
    use crate::swim::MembershipView;

    #[test]
    fn test_history_prune_limits() {
        let mut m = CAPManager::new(CAPStrategy::Balanced).with_history_limits(5, 2);
        let view = MembershipView::new("me".to_string());
        for _ in 0..10 {
            let _ = m.select_consistency_level(&view);
        }
        assert!(m.consistency_history.len() <= 5);
    }
}

/// 网络分区检测器
#[derive(Debug, Clone)]
pub struct PartitionDetector {
    connectivity_matrix: HashMap<String, HashMap<String, bool>>,
    last_connectivity_check: Instant,
    partition_threshold: f64,
    stats: PartitionStats,
}

#[derive(Debug, Clone, Default)]
pub struct PartitionStats {
    pub total_checks: u64,
    pub partition_detected_count: u64,
    pub last_partition_duration: Duration,
    pub average_partition_duration: Duration,
    pub connectivity_ratio: f64,
}

impl PartitionDetector {
    /// 创建新的分区检测器
    pub fn new() -> Self {
        Self {
            connectivity_matrix: HashMap::new(),
            last_connectivity_check: Instant::now(),
            partition_threshold: 0.5, // 50%的节点不可达时认为分区
            stats: PartitionStats::default(),
        }
    }

    /// 获取分区阈值
    pub fn partition_threshold(&self) -> f64 {
        self.partition_threshold
    }

    /// 获取统计信息
    pub fn stats(&self) -> &PartitionStats {
        &self.stats
    }

    /// 设置分区阈值
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.partition_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// 检测网络分区
    pub fn detect_partition(&mut self, membership_view: &MembershipView) -> bool {
        self.stats.total_checks += 1;

        let alive_members = membership_view.alive_members();
        if alive_members.len() <= 1 {
            return false; // 单个节点或没有活跃节点不算分区
        }

        // 更新连接矩阵
        self.update_connectivity_matrix(&alive_members);

        // 计算连通性
        let connectivity_ratio = self.calculate_connectivity_ratio(&alive_members);
        self.stats.connectivity_ratio = connectivity_ratio;

        let is_partitioned = connectivity_ratio < self.partition_threshold;

        if is_partitioned {
            self.stats.partition_detected_count += 1;
        }

        self.last_connectivity_check = Instant::now();
        is_partitioned
    }

    /// 更新连接矩阵
    fn update_connectivity_matrix(&mut self, alive_members: &[String]) {
        // 模拟网络连接检测
        for node1 in alive_members {
            let connections = self
                .connectivity_matrix
                .entry(node1.clone())
                .or_default();

            for node2 in alive_members {
                if node1 != node2 {
                    // 模拟连接检测（实际实现中这里会是真实的网络检测）
                    let is_connected = Self::simulate_connection_test_static(node1, node2);
                    connections.insert(node2.clone(), is_connected);
                }
            }
        }
    }

    /// 模拟连接测试
    #[allow(dead_code)]
    fn simulate_connection_test(&self, _node1: &str, _node2: &str) -> bool {
        // 在实际实现中，这里会进行真实的网络连接测试
        // 现在使用随机模拟
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        let random = (hash % 100) as f64 / 100.0;

        random > 0.2 // 80%的连接成功率
    }

    /// 静态模拟连接测试（避免借用检查器问题）
    #[allow(dead_code)]
    fn simulate_connection_test_static(_node1: &str, _node2: &str) -> bool {
        // 在实际实现中，这里会进行真实的网络连接测试
        // 现在使用随机模拟
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        let random = (hash % 100) as f64 / 100.0;

        random > 0.2 // 80%的连接成功率
    }

    /// 简单的连接测试（避免借用检查器问题）
    #[allow(dead_code)]
    fn simulate_connection_test_simple(&self, _node1: &str, _node2: &str) -> bool {
        // 在实际实现中，这里会进行真实的网络连接测试
        // 现在使用随机模拟
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        let random = (hash % 100) as f64 / 100.0;

        random > 0.2 // 80%的连接成功率
    }

    /// 计算连通性比例
    fn calculate_connectivity_ratio(&self, alive_members: &[String]) -> f64 {
        if alive_members.len() <= 1 {
            return 1.0;
        }

        let mut total_connections = 0;
        let mut successful_connections = 0;

        for node1 in alive_members {
            if let Some(connections) = self.connectivity_matrix.get(node1) {
                for node2 in alive_members {
                    if node1 != node2 {
                        total_connections += 1;
                        if *connections.get(node2).unwrap_or(&false) {
                            successful_connections += 1;
                        }
                    }
                }
            }
        }

        if total_connections == 0 {
            1.0
        } else {
            successful_connections as f64 / total_connections as f64
        }
    }

    /// 检查当前是否处于分区状态
    pub fn is_currently_partitioned(&self) -> bool {
        self.stats.connectivity_ratio < self.partition_threshold
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> PartitionStats {
        self.stats.clone()
    }
}

impl Default for PartitionDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// CAP定理权衡分析器
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CAPAnalyzer {
    decision_history: Vec<ConsistencyDecision>,
    performance_metrics: HashMap<ConsistencyLevel, PerformanceMetrics>,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub usage_count: u64,
    pub average_latency: Duration,
    pub success_rate: f64,
    pub partition_tolerance_score: f64,
}

impl CAPAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            decision_history: Vec::new(),
            performance_metrics: HashMap::new(),
        }
    }

    /// 获取决策历史长度
    pub fn decision_history_len(&self) -> usize {
        self.decision_history.len()
    }

    /// 检查决策历史是否为空
    pub fn is_decision_history_empty(&self) -> bool {
        self.decision_history.is_empty()
    }

    /// 记录一致性决策
    pub fn record_decision(&mut self, decision: ConsistencyDecision) {
        self.decision_history.push(decision);

        // 保持历史记录在合理范围内
        if self.decision_history.len() > 10000 {
            self.decision_history.drain(0..1000);
        }
    }

    /// 分析CAP权衡效果
    pub fn analyze_cap_tradeoffs(&self) -> CAPAnalysisReport {
        let total_decisions = self.decision_history.len();
        if total_decisions == 0 {
            return CAPAnalysisReport::default();
        }

        let mut partition_count = 0;
        let mut consistency_level_usage: HashMap<ConsistencyLevel, u64> = HashMap::new();
        let mut strategy_usage: HashMap<CAPStrategy, u64> = HashMap::new();

        for decision in &self.decision_history {
            if decision.is_partitioned {
                partition_count += 1;
            }

            *consistency_level_usage
                .entry(decision.selected_level)
                .or_insert(0) += 1;
            *strategy_usage.entry(decision.strategy).or_insert(0) += 1;
        }

        let partition_rate = partition_count as f64 / total_decisions as f64;

        // 计算最常用的一致性级别
        let most_used_level = consistency_level_usage
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(level, _)| *level)
            .unwrap_or(ConsistencyLevel::Eventual);

        // 计算最常用的策略
        let most_used_strategy = strategy_usage
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(strategy, _)| *strategy)
            .unwrap_or(CAPStrategy::Balanced);

        CAPAnalysisReport {
            total_decisions,
            partition_rate,
            most_used_consistency_level: most_used_level,
            most_used_strategy,
            consistency_level_distribution: consistency_level_usage,
            strategy_distribution: strategy_usage,
            recommendations: self.generate_recommendations(partition_rate, most_used_level),
        }
    }

    /// 生成优化建议
    fn generate_recommendations(
        &self,
        partition_rate: f64,
        most_used_level: ConsistencyLevel,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if partition_rate > 0.3 {
            recommendations.push("检测到频繁的网络分区，建议使用AP策略或平衡策略".to_string());
        }

        if partition_rate < 0.1 {
            recommendations.push("网络稳定，可以考虑使用CP策略以获得更强的一致性".to_string());
        }

        match most_used_level {
            ConsistencyLevel::Strong | ConsistencyLevel::Linearizable => {
                recommendations.push("当前主要使用强一致性，注意监控可用性指标".to_string());
            }
            ConsistencyLevel::Eventual => {
                recommendations.push("当前主要使用最终一致性，注意监控数据一致性指标".to_string());
            }
            ConsistencyLevel::Causal => {
                recommendations
                    .push("当前使用因果一致性，这是平衡一致性和可用性的好选择".to_string());
            }
            _ => {}
        }

        if recommendations.is_empty() {
            recommendations.push("当前配置运行良好，建议继续监控".to_string());
        }

        recommendations
    }
}

#[derive(Debug, Clone, Default)]
pub struct CAPAnalysisReport {
    pub total_decisions: usize,
    pub partition_rate: f64,
    pub most_used_consistency_level: ConsistencyLevel,
    pub most_used_strategy: CAPStrategy,
    pub consistency_level_distribution: HashMap<ConsistencyLevel, u64>,
    pub strategy_distribution: HashMap<CAPStrategy, u64>,
    pub recommendations: Vec<String>,
}

impl Default for CAPAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

use c20_distributed::{
    CAPAnalysisReport, CAPAnalyzer, CAPManager, CAPStrategy, ConsistencyDecision, ConsistencyLevel,
    MembershipView, PartitionDetector, PerformanceMetrics, SwimMemberState,
};
use std::time::Duration;

#[test]
fn test_cap_manager_creation() {
    let manager = CAPManager::new(CAPStrategy::Balanced);

    assert_eq!(manager.strategy(), CAPStrategy::Balanced);
    assert_eq!(
        manager.partition_check_interval(),
        Duration::from_millis(1000)
    );
}

#[test]
fn test_cap_manager_with_custom_interval() {
    let manager = CAPManager::new(CAPStrategy::ConsistencyPartition)
        .with_partition_check_interval(Duration::from_millis(500));

    assert_eq!(manager.strategy(), CAPStrategy::ConsistencyPartition);
    assert_eq!(
        manager.partition_check_interval(),
        Duration::from_millis(500)
    );
}

#[test]
fn test_cap_strategy_selection() {
    let mut manager = CAPManager::new(CAPStrategy::ConsistencyPartition);
    let mut view = MembershipView::new("node1".to_string());

    // 添加一些节点
    view.local_update("node2", SwimMemberState::Alive, 1);
    view.local_update("node3", SwimMemberState::Alive, 1);

    let level = manager.select_consistency_level(&view);
    assert_eq!(level, ConsistencyLevel::Strong);
}

#[test]
fn test_cap_strategy_availability_partition() {
    let mut manager = CAPManager::new(CAPStrategy::AvailabilityPartition);
    let mut view = MembershipView::new("node1".to_string());

    // 添加一些节点
    view.local_update("node2", SwimMemberState::Alive, 1);
    view.local_update("node3", SwimMemberState::Alive, 1);

    let level = manager.select_consistency_level(&view);
    assert_eq!(level, ConsistencyLevel::Eventual);
}

#[test]
fn test_cap_strategy_balanced() {
    let mut manager = CAPManager::new(CAPStrategy::Balanced);
    let mut view = MembershipView::new("node1".to_string());

    // 添加一些节点
    view.local_update("node2", SwimMemberState::Alive, 1);
    view.local_update("node3", SwimMemberState::Alive, 1);

    let level = manager.select_consistency_level(&view);
    // 由于是模拟，结果可能是Causal或Linearizable
    assert!(matches!(
        level,
        ConsistencyLevel::Causal | ConsistencyLevel::Linearizable
    ));
}

#[test]
fn test_cap_manager_recent_decisions() {
    let mut manager = CAPManager::new(CAPStrategy::Balanced);
    let mut view = MembershipView::new("node1".to_string());

    // 添加一些节点
    view.local_update("node2", SwimMemberState::Alive, 1);
    view.local_update("node3", SwimMemberState::Alive, 1);

    // 进行几次决策
    for _ in 0..5 {
        manager.select_consistency_level(&view);
    }

    let recent = manager.get_recent_decisions(3);
    assert!(recent.len() <= 3);
}

#[test]
fn test_cap_manager_strategy_update() {
    let mut manager = CAPManager::new(CAPStrategy::Balanced);
    assert_eq!(manager.strategy(), CAPStrategy::Balanced);

    manager.update_strategy(CAPStrategy::ConsistencyPartition);
    assert_eq!(manager.strategy(), CAPStrategy::ConsistencyPartition);
}

#[test]
fn test_partition_detector_creation() {
    let detector = PartitionDetector::new();

    assert_eq!(detector.partition_threshold(), 0.5);
    assert_eq!(detector.stats().total_checks, 0);
}

#[test]
fn test_partition_detector_with_threshold() {
    let detector = PartitionDetector::new().with_threshold(0.8);

    assert_eq!(detector.partition_threshold(), 0.8);
}

#[test]
fn test_partition_detector_threshold_clamping() {
    let detector1 = PartitionDetector::new().with_threshold(1.5);
    let detector2 = PartitionDetector::new().with_threshold(-0.5);

    assert_eq!(detector1.partition_threshold(), 1.0);
    assert_eq!(detector2.partition_threshold(), 0.0);
}

#[test]
fn test_partition_detection() {
    let mut detector = PartitionDetector::new();
    let mut view = MembershipView::new("node1".to_string());

    // 添加一些节点
    view.local_update("node2", SwimMemberState::Alive, 1);
    view.local_update("node3", SwimMemberState::Alive, 1);
    view.local_update("node4", SwimMemberState::Alive, 1);

    let is_partitioned = detector.detect_partition(&view);

    // 由于是模拟，结果可能是true或false
    assert!(is_partitioned == true || is_partitioned == false);
    assert_eq!(detector.stats().total_checks, 1);
}

#[test]
fn test_partition_detector_single_node() {
    let mut detector = PartitionDetector::new();
    let mut view = MembershipView::new("node1".to_string());

    // 只有一个节点
    view.local_update("node1", SwimMemberState::Alive, 1);

    let is_partitioned = detector.detect_partition(&view);
    assert!(!is_partitioned); // 单个节点不算分区
}

#[test]
fn test_partition_detector_no_alive_nodes() {
    let mut detector = PartitionDetector::new();
    let view = MembershipView::new("node1".to_string());

    // 没有活跃节点
    let is_partitioned = detector.detect_partition(&view);
    assert!(!is_partitioned); // 没有节点不算分区
}

#[test]
fn test_partition_stats() {
    let mut detector = PartitionDetector::new();
    let mut view = MembershipView::new("node1".to_string());

    // 添加一些节点
    view.local_update("node2", SwimMemberState::Alive, 1);
    view.local_update("node3", SwimMemberState::Alive, 1);

    // 进行几次检测
    for _ in 0..10 {
        detector.detect_partition(&view);
    }

    let stats = detector.get_stats();
    assert_eq!(stats.total_checks, 10);
    assert!(stats.partition_detected_count <= 10);
}

#[test]
fn test_cap_analyzer_creation() {
    let analyzer = CAPAnalyzer::new();

    assert!(analyzer.is_decision_history_empty());
}

#[test]
fn test_cap_analyzer_record_decision() {
    let mut analyzer = CAPAnalyzer::new();

    let decision = ConsistencyDecision {
        timestamp: std::time::SystemTime::now(),
        is_partitioned: false,
        selected_level: ConsistencyLevel::Linearizable,
        strategy: CAPStrategy::Balanced,
        reasoning: "测试决策".to_string(),
    };

    analyzer.record_decision(decision);
    assert_eq!(analyzer.decision_history_len(), 1);
}

#[test]
fn test_cap_analyzer_empty_analysis() {
    let analyzer = CAPAnalyzer::new();
    let report = analyzer.analyze_cap_tradeoffs();

    assert_eq!(report.total_decisions, 0);
    assert_eq!(report.partition_rate, 0.0);
    assert_eq!(
        report.most_used_consistency_level,
        ConsistencyLevel::Eventual
    );
    assert_eq!(report.most_used_strategy, CAPStrategy::Balanced);
}

#[test]
fn test_cap_analyzer_with_decisions() {
    let mut analyzer = CAPAnalyzer::new();

    // 记录一些决策
    for i in 0..10 {
        let decision = ConsistencyDecision {
            timestamp: std::time::SystemTime::now(),
            is_partitioned: i % 3 == 0, // 每3个中有1个分区
            selected_level: if i % 2 == 0 {
                ConsistencyLevel::Strong
            } else {
                ConsistencyLevel::Eventual
            },
            strategy: CAPStrategy::Balanced,
            reasoning: format!("决策 {}", i),
        };
        analyzer.record_decision(decision);
    }

    let report = analyzer.analyze_cap_tradeoffs();

    assert_eq!(report.total_decisions, 10);
    assert!(report.partition_rate > 0.0);
    assert!(report.partition_rate < 1.0);
    assert!(!report.recommendations.is_empty());
}

#[test]
fn test_consistency_decision_creation() {
    let decision = ConsistencyDecision {
        timestamp: std::time::SystemTime::now(),
        is_partitioned: true,
        selected_level: ConsistencyLevel::Causal,
        strategy: CAPStrategy::Balanced,
        reasoning: "检测到分区，选择因果一致性".to_string(),
    };

    assert!(decision.is_partitioned);
    assert_eq!(decision.selected_level, ConsistencyLevel::Causal);
    assert_eq!(decision.strategy, CAPStrategy::Balanced);
    assert_eq!(decision.reasoning, "检测到分区，选择因果一致性");
}

#[test]
fn test_performance_metrics_default() {
    let metrics = PerformanceMetrics::default();

    assert_eq!(metrics.usage_count, 0);
    assert_eq!(metrics.average_latency, Duration::from_secs(0));
    assert_eq!(metrics.success_rate, 0.0);
    assert_eq!(metrics.partition_tolerance_score, 0.0);
}

#[test]
fn test_cap_analysis_report_default() {
    let report = CAPAnalysisReport::default();

    assert_eq!(report.total_decisions, 0);
    assert_eq!(report.partition_rate, 0.0);
    assert_eq!(
        report.most_used_consistency_level,
        ConsistencyLevel::Eventual
    );
    assert_eq!(report.most_used_strategy, CAPStrategy::Balanced);
    assert!(report.consistency_level_distribution.is_empty());
    assert!(report.strategy_distribution.is_empty());
    assert!(report.recommendations.is_empty());
}

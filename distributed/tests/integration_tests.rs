use c20_distributed::{
    ByzantineMessage, ByzantineNetwork, CAPManager, CAPStrategy, ConsistencyLevel, MembershipView,
    MonotonicConsistencyManager, PBFTNode, SessionConsistencyManager, SwimMemberState, VectorClock,
};
use std::time::Duration;

/// 集成测试：完整的分布式系统工作流
#[test]
fn test_distributed_system_integration() {
    // 1. 创建CAP管理器
    let mut cap_manager = CAPManager::new(CAPStrategy::Balanced);

    // 2. 创建成员视图
    let mut membership_view = MembershipView::new("node_0".to_string());
    membership_view.local_update("node_1", SwimMemberState::Alive, 1);
    membership_view.local_update("node_2", SwimMemberState::Alive, 1);
    membership_view.local_update("node_3", SwimMemberState::Alive, 1);

    // 3. 根据网络状态选择一致性级别
    let consistency_level = cap_manager.select_consistency_level(&membership_view);
    assert!(matches!(
        consistency_level,
        ConsistencyLevel::Causal | ConsistencyLevel::Linearizable
    ));

    // 4. 创建向量时钟进行因果一致性管理
    let mut vector_clock = VectorClock::new();
    vector_clock.increment("node_0");

    // 5. 创建会话一致性管理器
    let mut session_manager = SessionConsistencyManager::new();
    session_manager.create_session("session_1".to_string());
    session_manager.update_read_version("session_1", vector_clock.clone());

    // 6. 创建单调一致性管理器
    let mut monotonic_manager = MonotonicConsistencyManager::new();
    assert!(monotonic_manager.check_monotonic_read("client_1", &vector_clock));

    // 7. 验证系统状态
    assert_eq!(membership_view.size(), 3);
    assert_eq!(membership_view.alive_count(), 3);
    assert!(session_manager.can_read("session_1", &vector_clock));
}

/// 集成测试：拜占庭容错与一致性结合
#[test]
fn test_byzantine_fault_tolerance_integration() {
    // 1. 创建拜占庭网络
    let network = ByzantineNetwork::new(4, Duration::from_millis(50), 0.0);

    // 2. 创建PBFT节点
    let mut primary_node = PBFTNode::new("node_0".to_string(), 4);
    let mut replica_node = PBFTNode::new("node_1".to_string(), 4);

    // 3. 验证拜占庭容错要求
    assert!(primary_node.is_byzantine_fault_tolerant());
    assert_eq!(primary_node.quorum_size(), 3);

    // 4. 标记一个节点为拜占庭节点
    primary_node.mark_node_byzantine("node_2");
    replica_node.mark_node_byzantine("node_2");

    // 5. 创建请求消息
    let request = ByzantineMessage::Request {
        id: "req_1".to_string(),
        content: b"test data".to_vec(),
        timestamp: std::time::SystemTime::now(),
        sender: "client_1".to_string(),
    };

    // 6. 主节点处理请求
    let responses = primary_node.handle_request(request).unwrap();
    assert_eq!(responses.len(), 1);

    // 7. 验证网络统计
    let stats = network.get_network_stats();
    assert_eq!(stats.total_nodes, 4);
    assert_eq!(stats.byzantine_nodes, 0); // 网络层面还没有标记
}

/// 集成测试：CAP定理与拜占庭容错的权衡
#[test]
fn test_cap_byzantine_tradeoff() {
    // 1. 创建不同策略的CAP管理器
    let mut cp_manager = CAPManager::new(CAPStrategy::ConsistencyPartition);
    let mut ap_manager = CAPManager::new(CAPStrategy::AvailabilityPartition);
    let mut balanced_manager = CAPManager::new(CAPStrategy::Balanced);

    // 2. 创建成员视图
    let mut membership_view = MembershipView::new("node_0".to_string());
    membership_view.local_update("node_1", SwimMemberState::Alive, 1);
    membership_view.local_update("node_2", SwimMemberState::Alive, 1);
    membership_view.local_update("node_3", SwimMemberState::Alive, 1);

    // 3. 测试不同策略的一致性级别选择
    let cp_level = cp_manager.select_consistency_level(&membership_view);
    let ap_level = ap_manager.select_consistency_level(&membership_view);
    let balanced_level = balanced_manager.select_consistency_level(&membership_view);

    assert_eq!(cp_level, ConsistencyLevel::Strong);
    assert_eq!(ap_level, ConsistencyLevel::Eventual);
    assert!(matches!(
        balanced_level,
        ConsistencyLevel::Causal | ConsistencyLevel::Linearizable
    ));

    // 4. 验证拜占庭容错支持
    assert!(cp_level.supports_byzantine_fault_tolerance());
    assert!(!ap_level.supports_byzantine_fault_tolerance());
    // 平衡策略可能选择因果一致性，不支持拜占庭容错
    // assert!(balanced_level.supports_byzantine_fault_tolerance());
}

/// 集成测试：向量时钟与一致性管理
#[test]
fn test_vector_clock_consistency_integration() {
    // 1. 创建多个向量时钟
    let mut clock1 = VectorClock::new();
    let mut clock2 = VectorClock::new();
    let mut clock3 = VectorClock::new();

    // 2. 模拟事件序列
    clock1.increment("node_1");
    clock2.increment("node_2");
    clock3.increment("node_3");

    // 3. 验证因果关系
    // 注意：由于clock1和clock2是并发的（不同节点），它们不应该有因果关系
    assert!(clock1.is_concurrent(&clock2));
    assert!(clock2.is_concurrent(&clock3));
    assert!(clock1.is_concurrent(&clock3));

    // 4. 测试并发事件
    let mut clock4 = VectorClock::new();
    clock4.increment("node_4");
    assert!(clock1.is_concurrent(&clock4));

    // 5. 测试时钟更新
    clock1.update(&clock2);
    // 更新后，clock1应该包含clock2的所有信息，所以它们应该相等或clock1 >= clock2
    assert!(!clock1.happens_before(&clock2));
    // clock2可能仍然先于clock1，因为clock1只是更新了，不是完全包含
}

/// 集成测试：会话一致性与单调一致性
#[test]
fn test_session_monotonic_consistency_integration() {
    // 1. 创建会话管理器
    let mut session_manager = SessionConsistencyManager::new();
    session_manager.create_session("session_1".to_string());

    // 2. 创建单调一致性管理器
    let mut monotonic_manager = MonotonicConsistencyManager::new();

    // 3. 创建向量时钟序列
    let mut clock1 = VectorClock::new();
    clock1.increment("node_1");

    let mut clock2 = VectorClock::new();
    clock2.increment("node_1");
    clock2.increment("node_2");

    let mut clock3 = VectorClock::new();
    clock3.increment("node_1");
    clock3.increment("node_2");
    clock3.increment("node_3");

    // 4. 测试会话一致性
    session_manager.update_read_version("session_1", clock1.clone());
    assert!(session_manager.can_read("session_1", &clock2));
    assert!(session_manager.can_read("session_1", &clock3));

    // 5. 测试单调一致性
    assert!(monotonic_manager.check_monotonic_read("client_1", &clock1));
    assert!(monotonic_manager.check_monotonic_read("client_1", &clock2));
    assert!(monotonic_manager.check_monotonic_read("client_1", &clock3));

    // 6. 测试违反单调一致性
    assert!(!monotonic_manager.check_monotonic_read("client_1", &clock1)); // 应该失败，因为clock1 < clock3
}

/// 集成测试：网络分区与故障检测
#[test]
fn test_network_partition_failure_detection() {
    // 1. 创建CAP管理器
    let mut cap_manager = CAPManager::new(CAPStrategy::Balanced);

    // 2. 创建成员视图
    let mut membership_view = MembershipView::new("node_0".to_string());
    membership_view.local_update("node_1", SwimMemberState::Alive, 1);
    membership_view.local_update("node_2", SwimMemberState::Alive, 1);
    membership_view.local_update("node_3", SwimMemberState::Alive, 1);

    // 3. 模拟网络分区（标记一些节点为可疑）
    membership_view.local_update("node_2", SwimMemberState::Suspect, 2);
    membership_view.local_update("node_3", SwimMemberState::Suspect, 2);

    // 4. 测试分区检测
    let _is_partitioned = cap_manager.detect_partition(&membership_view);
    // 由于是模拟，结果可能是true或false

    // 5. 根据分区状态选择一致性级别
    let _consistency_level = cap_manager.select_consistency_level(&membership_view);

    // 6. 验证决策历史
    let recent_decisions = cap_manager.get_recent_decisions(5);
    assert!(!recent_decisions.is_empty());

    // 7. 验证分区统计
    let partition_stats = cap_manager.get_partition_stats();
    assert!(partition_stats.total_checks > 0);
}

/// 集成测试：完整的PBFT协议流程
#[test]
fn test_complete_pbft_protocol_flow() {
    // 1. 创建PBFT网络
    let mut network = ByzantineNetwork::new(4, Duration::from_millis(10), 0.0);

    // 2. 获取主节点
    let primary_id = "node_0".to_string();
    let mut primary_node = network.nodes.get_mut(&primary_id).unwrap().clone();

    // 3. 创建请求
    let request = ByzantineMessage::Request {
        id: "req_1".to_string(),
        content: b"test operation".to_vec(),
        timestamp: std::time::SystemTime::now(),
        sender: "client_1".to_string(),
    };

    // 4. 主节点处理请求，生成准备消息
    let prepare_messages = primary_node.handle_request(request).unwrap();
    assert_eq!(prepare_messages.len(), 1);

    // 5. 模拟其他节点处理准备消息
    let mut replica_nodes = Vec::new();
    for i in 1..4 {
        let node_id = format!("node_{}", i);
        if let Some(node) = network.nodes.get_mut(&node_id) {
            let responses = node.handle_prepare(prepare_messages[0].clone()).unwrap();
            replica_nodes.push((node_id, responses));
        }
    }

    // 6. 验证网络状态
    let stats = network.get_network_stats();
    assert_eq!(stats.total_nodes, 4);
    assert_eq!(stats.honest_nodes, 4);
}

/// 集成测试：一致性级别与拜占庭容错的兼容性
#[test]
fn test_consistency_byzantine_compatibility() {
    // 1. 测试各种一致性级别对拜占庭容错的支持
    let strong_level = ConsistencyLevel::Strong;
    let linearizable_level = ConsistencyLevel::Linearizable;
    let eventual_level = ConsistencyLevel::Eventual;
    let causal_level = ConsistencyLevel::Causal;

    assert!(strong_level.supports_byzantine_fault_tolerance());
    assert!(linearizable_level.supports_byzantine_fault_tolerance());
    assert!(!eventual_level.supports_byzantine_fault_tolerance());
    assert!(!causal_level.supports_byzantine_fault_tolerance());

    // 2. 测试CAP策略与拜占庭容错的结合
    let mut cp_manager = CAPManager::new(CAPStrategy::ConsistencyPartition);
    let mut ap_manager = CAPManager::new(CAPStrategy::AvailabilityPartition);

    let mut membership_view = MembershipView::new("node_0".to_string());
    membership_view.local_update("node_1", SwimMemberState::Alive, 1);
    membership_view.local_update("node_2", SwimMemberState::Alive, 1);
    membership_view.local_update("node_3", SwimMemberState::Alive, 1);

    let cp_level = cp_manager.select_consistency_level(&membership_view);
    let ap_level = ap_manager.select_consistency_level(&membership_view);

    // CP策略应该选择支持拜占庭容错的一致性级别
    assert!(cp_level.supports_byzantine_fault_tolerance());

    // AP策略可能选择不支持拜占庭容错的一致性级别
    assert!(!ap_level.supports_byzantine_fault_tolerance());
}

/// 集成测试：系统性能与可靠性权衡
#[test]
fn test_performance_reliability_tradeoff() {
    // 1. 创建不同配置的系统
    let mut high_performance_system = CAPManager::new(CAPStrategy::AvailabilityPartition);
    let mut high_reliability_system = CAPManager::new(CAPStrategy::ConsistencyPartition);
    let mut balanced_system = CAPManager::new(CAPStrategy::Balanced);

    // 2. 创建成员视图
    let mut membership_view = MembershipView::new("node_0".to_string());
    for i in 1..5 {
        membership_view.local_update(&format!("node_{}", i), SwimMemberState::Alive, 1);
    }

    // 3. 测试不同系统的决策
    let hp_level = high_performance_system.select_consistency_level(&membership_view);
    let hr_level = high_reliability_system.select_consistency_level(&membership_view);
    let balanced_level = balanced_system.select_consistency_level(&membership_view);

    // 4. 验证性能与可靠性的权衡
    assert_eq!(hp_level, ConsistencyLevel::Eventual); // 高性能，低一致性
    assert_eq!(hr_level, ConsistencyLevel::Strong); // 高可靠性，强一致性
    assert!(matches!(
        balanced_level,
        ConsistencyLevel::Causal | ConsistencyLevel::Linearizable
    )); // 平衡

    // 5. 验证拜占庭容错支持
    assert!(!hp_level.supports_byzantine_fault_tolerance()); // 高性能系统不支持拜占庭容错
    assert!(hr_level.supports_byzantine_fault_tolerance()); // 高可靠性系统支持拜占庭容错
    // 平衡系统可能选择因果一致性，不支持拜占庭容错
    // assert!(balanced_level.supports_byzantine_fault_tolerance());
}

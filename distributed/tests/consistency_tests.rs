use c20_distributed::{
    CAPStrategy, ConsistencyLevel, MonotonicConsistencyManager, SessionConsistencyManager,
    VectorClock,
};

#[test]
fn test_consistency_level_descriptions() {
    assert_eq!(
        ConsistencyLevel::Strong.description(),
        "强一致性：所有节点立即看到相同的数据"
    );
    assert_eq!(
        ConsistencyLevel::Linearizable.description(),
        "线性一致性：所有操作都有全局顺序"
    );
    assert_eq!(
        ConsistencyLevel::Causal.description(),
        "因果一致性：保持因果关系"
    );
    assert_eq!(
        ConsistencyLevel::Eventual.description(),
        "最终一致性：最终所有节点会收敛到相同状态"
    );
}

#[test]
fn test_consistency_level_cap_properties() {
    // 强一致性和线性一致性不支持分区容忍性
    assert!(!ConsistencyLevel::Strong.supports_partition_tolerance());
    assert!(!ConsistencyLevel::Linearizable.supports_partition_tolerance());

    // 其他级别支持分区容忍性
    assert!(ConsistencyLevel::Eventual.supports_partition_tolerance());
    assert!(ConsistencyLevel::Causal.supports_partition_tolerance());
    assert!(ConsistencyLevel::Session.supports_partition_tolerance());
}

#[test]
fn test_consistency_level_availability() {
    // 强一致性和线性一致性不支持高可用性
    assert!(!ConsistencyLevel::Strong.supports_high_availability());
    assert!(!ConsistencyLevel::Linearizable.supports_high_availability());

    // 其他级别支持高可用性
    assert!(ConsistencyLevel::Eventual.supports_high_availability());
    assert!(ConsistencyLevel::Causal.supports_high_availability());
}

#[test]
fn test_cap_strategy_selection() {
    // CP策略总是选择强一致性
    assert_eq!(
        CAPStrategy::ConsistencyPartition.select_consistency_level(false),
        ConsistencyLevel::Strong
    );
    assert_eq!(
        CAPStrategy::ConsistencyPartition.select_consistency_level(true),
        ConsistencyLevel::Strong
    );

    // AP策略总是选择最终一致性
    assert_eq!(
        CAPStrategy::AvailabilityPartition.select_consistency_level(false),
        ConsistencyLevel::Eventual
    );
    assert_eq!(
        CAPStrategy::AvailabilityPartition.select_consistency_level(true),
        ConsistencyLevel::Eventual
    );

    // 平衡策略根据分区状态选择
    assert_eq!(
        CAPStrategy::Balanced.select_consistency_level(false),
        ConsistencyLevel::Linearizable
    );
    assert_eq!(
        CAPStrategy::Balanced.select_consistency_level(true),
        ConsistencyLevel::Causal
    );
}

#[test]
fn test_vector_clock_basic_operations() {
    let mut clock1 = VectorClock::new();
    let mut clock2 = VectorClock::new();

    // 初始状态应该相等（两个空时钟）
    assert!(!clock1.happens_before(&clock2));
    assert!(!clock2.happens_before(&clock1));
    assert!(!clock1.is_concurrent(&clock2)); // 相等的情况下不是并发的

    // 增加clock1的节点A
    clock1.increment("A");
    assert!(!clock1.happens_before(&clock2)); // clock1不能先于clock2，因为clock2没有节点A
    assert!(clock2.happens_before(&clock1)); // clock2先于clock1，因为clock2是空的
    assert!(!clock1.is_concurrent(&clock2)); // 它们不是并发的，clock2先于clock1

    // 增加clock2的节点B
    clock2.increment("B");
    assert!(!clock1.happens_before(&clock2));
    assert!(!clock2.happens_before(&clock1));
    assert!(clock1.is_concurrent(&clock2));
}

#[test]
fn test_vector_clock_update() {
    let mut clock1 = VectorClock::new();
    let mut clock2 = VectorClock::new();

    clock1.increment("A");
    clock1.increment("A");
    clock1.increment("B");

    clock2.increment("A");
    clock2.increment("C");

    // 更新前clock1和clock2是并发的
    assert!(clock1.is_concurrent(&clock2));

    // 更新后应该取最大值
    clock1.update(&clock2);

    // clock1现在应该包含所有节点的最大时钟值
    // A: max(2, 1) = 2, B: 1, C: 1
    assert!(!clock1.happens_before(&clock2));
    assert!(clock2.happens_before(&clock1));
}

#[test]
fn test_session_consistency_manager() {
    let mut manager = SessionConsistencyManager::new();

    // 创建会话
    manager.create_session("session1".to_string());

    let mut version1 = VectorClock::new();
    version1.increment("A");

    let mut version2 = VectorClock::new();
    version2.increment("A");
    version2.increment("A");

    // 初始状态可以读写
    assert!(manager.can_read("session1", &version1));
    assert!(manager.can_write("session1", &version1));

    // 更新读版本
    manager.update_read_version("session1", version1.clone());

    // 更新的版本可以读
    assert!(manager.can_read("session1", &version2));

    // 更新的版本可以写
    assert!(manager.can_write("session1", &version2));
}

#[test]
fn test_monotonic_consistency_manager() {
    let mut manager = MonotonicConsistencyManager::new();

    let mut version1 = VectorClock::new();
    version1.increment("A");

    let mut version2 = VectorClock::new();
    version2.increment("A");
    version2.increment("A");

    let mut version3 = VectorClock::new();
    version3.increment("A");
    version3.increment("A");
    version3.increment("A");

    // 第一次读应该成功
    assert!(manager.check_monotonic_read("client1", &version2));

    // 更新的版本可以读
    assert!(manager.check_monotonic_read("client1", &version3));

    // 更旧的版本不能读（违反单调读一致性）
    assert!(!manager.check_monotonic_read("client1", &version1));

    // 第一次写应该成功
    assert!(manager.check_monotonic_write("client1", &version2));

    // 更新的版本可以写
    assert!(manager.check_monotonic_write("client1", &version3));

    // 更旧的版本不能写（违反单调写一致性）
    assert!(!manager.check_monotonic_write("client1", &version1));
}

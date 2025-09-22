//! 一致性模型使用示例

use crate::consistency::{
    ConsistencyLevel, AdvancedConsistencyManager, SessionConsistencyManager,
    MonotonicConsistencyManager, VectorClock, CAPStrategy
};

/// 一致性级别演示
pub fn consistency_levels_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 一致性级别演示 ===");
    
    // 展示所有一致性级别
    let levels = vec![
        ConsistencyLevel::Strong,
        ConsistencyLevel::Linearizable,
        ConsistencyLevel::Sequential,
        ConsistencyLevel::Causal,
        ConsistencyLevel::Session,
        ConsistencyLevel::MonotonicRead,
        ConsistencyLevel::MonotonicWrite,
        ConsistencyLevel::ReadYourWrites,
        ConsistencyLevel::WritesFollowReads,
        ConsistencyLevel::Eventual,
        ConsistencyLevel::StrongEventual,
    ];
    
    println!("一致性级别及其描述:");
    for level in &levels {
        println!("• {:?}: {}", level, level.description());
        println!("  强度: {}", level.strength());
    }
    
    // 演示一致性级别兼容性
    println!("\n一致性级别兼容性检查:");
    let test_levels = vec![
        ConsistencyLevel::Strong,
        ConsistencyLevel::Causal,
        ConsistencyLevel::Eventual,
    ];
    
    for level1 in &test_levels {
        for level2 in &test_levels {
            let compatible = level1.is_compatible_with(level2);
            let icon = if compatible { "✅" } else { "❌" };
            println!("{} {:?} 与 {:?} 兼容", icon, level1, level2);
        }
    }
    
    Ok(())
}

/// 向量时钟演示
pub fn vector_clock_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 向量时钟演示 ===");
    
    // 创建三个节点的向量时钟
    let mut node_a = VectorClock::new();
    let mut node_b = VectorClock::new();
    let mut node_c = VectorClock::new();
    
    println!("初始状态:");
    println!("节点 A: {:?}", node_a);
    println!("节点 B: {:?}", node_b);
    println!("节点 C: {:?}", node_c);
    
    // 节点 A 发生事件
    node_a.increment("node_a");
    println!("\n节点 A 发生事件后:");
    println!("节点 A: {:?}", node_a);
    
    // 节点 B 发生事件
    node_b.increment("node_b");
    println!("节点 B 发生事件后:");
    println!("节点 B: {:?}", node_b);
    
    // 节点 A 接收节点 B 的消息
    node_a.update(&node_b);
    println!("\n节点 A 接收节点 B 的消息后:");
    println!("节点 A: {:?}", node_a);
    
    // 节点 C 发生事件
    node_c.increment("node_c");
    println!("节点 C 发生事件后:");
    println!("节点 C: {:?}", node_c);
    
    // 节点 C 接收节点 A 的消息
    node_c.update(&node_a);
    println!("\n节点 C 接收节点 A 的消息后:");
    println!("节点 C: {:?}", node_c);
    
    // 比较向量时钟
    println!("\n向量时钟比较:");
    println!("A vs B: A happens before B: {}", node_a.happens_before(&node_b));
    println!("B vs C: B happens before C: {}", node_b.happens_before(&node_c));
    println!("A vs C: A happens before C: {}", node_a.happens_before(&node_c));
    
    Ok(())
}

/// 会话一致性管理器演示
pub fn session_consistency_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 会话一致性管理器演示 ===");
    
    let mut session_manager = SessionConsistencyManager::new();
    
    // 模拟会话操作
    let session_id = "user_123".to_string();
    
    println!("开始会话: {}", session_id);
    
    // 创建会话
    session_manager.create_session(session_id.clone());
    
    // 创建向量时钟用于演示
    let mut write_version = VectorClock::new();
    write_version.increment("node_1");
    
    // 写入操作
    session_manager.update_write_version(&session_id, write_version.clone());
    println!("执行写入操作 - 更新写版本");
    
    // 读取操作（应该能看到刚才的写入）
    let can_read = session_manager.can_read(&session_id, &write_version);
    println!("执行读取操作 - 会话一致性检查: {}", can_read);
    
    // 模拟会话结束（通过清理过期会话）
    session_manager.cleanup_expired_sessions(std::time::Duration::from_secs(0));
    println!("会话结束: {}", session_id);
    
    // 新会话
    let new_session_id = "user_456".to_string();
    println!("\n开始新会话: {}", new_session_id);
    
    session_manager.create_session(new_session_id.clone());
    let new_write_version = write_version.clone();
    session_manager.update_write_version(&new_session_id, new_write_version.clone());
    println!("新会话写入操作");
    
    let can_read_new = session_manager.can_read(&new_session_id, &new_write_version);
    println!("新会话读取操作 - 一致性检查: {}", can_read_new);
    
    Ok(())
}

/// 单调一致性管理器演示
pub fn monotonic_consistency_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 单调一致性管理器演示 ===");
    
    let mut monotonic_manager = MonotonicConsistencyManager::new();
    
    let client_id = "client_1";
    
    // 模拟单调读一致性
    println!("模拟单调读一致性:");
    
    // 创建向量时钟用于演示
    let mut version1 = VectorClock::new();
    version1.increment("node_1");
    
    // 第一次读取
    let read1 = monotonic_manager.check_monotonic_read(client_id, &version1);
    println!("第一次读取检查: {}", read1);
    
    // 创建更新的版本
    let mut version2 = VectorClock::new();
    version2.increment("node_1");
    version2.increment("node_2");
    
    // 写入操作
    let write1 = monotonic_manager.check_monotonic_write(client_id, &version2);
    println!("写入操作检查: {}", write1);
    
    // 第二次读取（应该看到更新的值）
    let read2 = monotonic_manager.check_monotonic_read(client_id, &version2);
    println!("第二次读取检查: {}", read2);
    
    // 模拟单调写一致性
    println!("\n模拟单调写一致性:");
    
    let mut version3 = VectorClock::new();
    version3.increment("node_1");
    version3.increment("node_2");
    version3.increment("node_3");
    
    let write2 = monotonic_manager.check_monotonic_write(client_id, &version3);
    println!("更新操作检查: {}", write2);
    
    // 尝试违反单调性的操作（使用更旧的版本）
    let read3 = monotonic_manager.check_monotonic_read(client_id, &version1);
    println!("违反单调性的读取检查: {}", read3);
    
    Ok(())
}

/// 高级一致性管理器演示
pub fn advanced_consistency_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 高级一致性管理器演示 ===");
    
    let mut advanced_manager = AdvancedConsistencyManager::new(ConsistencyLevel::Causal);
    
    let client_id = "advanced_client";
    
    // 设置一致性级别
    advanced_manager.set_consistency_level(ConsistencyLevel::Causal);
    println!("设置一致性级别为: {:?}", ConsistencyLevel::Causal);
    
    // 创建客户端会话
    let session_id = advanced_manager.create_client_session(client_id.to_string());
    println!("创建客户端会话: {}", session_id);
    
    // 创建向量时钟用于演示
    let mut version = VectorClock::new();
    version.increment("node_1");
    
    // 执行操作
    let write_ok = advanced_manager.check_write_consistency(client_id, &version);
    println!("写入一致性检查: {}", write_ok);
    
    let read_ok = advanced_manager.check_read_consistency(client_id, &version);
    println!("读取一致性检查: {}", read_ok);
    
    // 切换一致性级别
    advanced_manager.set_consistency_level(ConsistencyLevel::Eventual);
    println!("切换到一致性级别: {:?}", ConsistencyLevel::Eventual);
    
    // 获取统计信息
    let stats = advanced_manager.get_stats();
    println!("一致性统计: {:?}", stats);
    
    Ok(())
}

/// CAP 定理策略演示
pub fn cap_strategy_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== CAP 定理策略演示 ===");
    
    // 创建不同的 CAP 策略
    let strategies = vec![
        CAPStrategy::ConsistencyPartition, // 一致性 + 分区容错
        CAPStrategy::AvailabilityPartition, // 可用性 + 分区容错
        CAPStrategy::Balanced, // 平衡策略
    ];
    
    for strategy in &strategies {
        println!("CAP 策略: {:?}", strategy);
        println!("  描述: {}", strategy.description());
        println!("  适用场景: {}", strategy.use_case());
        println!("  权衡: {}", strategy.trade_off());
        println!();
    }
    
    // 模拟网络分区情况下的决策
    println!("网络分区情况下的策略选择:");
    
    let network_partition = true;
    let high_availability_required = true;
    let strong_consistency_required = false;
    
    let recommended_strategy = if network_partition {
        if strong_consistency_required {
            CAPStrategy::ConsistencyPartition
        } else if high_availability_required {
            CAPStrategy::AvailabilityPartition
        } else {
            CAPStrategy::ConsistencyPartition // 默认选择
        }
    } else {
        CAPStrategy::Balanced
    };
    
    println!("推荐策略: {:?}", recommended_strategy);
    println!("理由: {}", recommended_strategy.description());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consistency_levels_demo() {
        consistency_levels_demo().unwrap();
    }
    
    #[test]
    fn test_vector_clock_demo() {
        vector_clock_demo().unwrap();
    }
    
    #[test]
    fn test_session_consistency_demo() {
        session_consistency_demo().unwrap();
    }
    
    #[test]
    fn test_monotonic_consistency_demo() {
        monotonic_consistency_demo().unwrap();
    }
    
    #[test]
    fn test_advanced_consistency_demo() {
        advanced_consistency_demo().unwrap();
    }
    
    #[test]
    fn test_cap_strategy_demo() {
        cap_strategy_demo().unwrap();
    }
}

use distributed::consistency::ConsistencyLevel;
use distributed::replication::{LocalReplicator, Replicator};
use distributed::topology::ConsistentHashRing;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 分布式复制演示开始");
    
    // 1. 创建一致性哈希环
    println!("\n📊 创建一致性哈希环...");
    let mut ring = ConsistentHashRing::new(16);
    let nodes = vec!["node1", "node2", "node3", "node4", "node5"];
    
    for node in &nodes {
        ring.add_node(node);
        println!("  ✅ 添加节点: {}", node);
    }
    
    // 2. 创建复制器
    println!("\n🔄 初始化复制器...");
    let node_strings: Vec<String> = nodes.iter().map(|s| s.to_string()).collect();
    let mut replicator: LocalReplicator<u64> = LocalReplicator::new(ring, node_strings.clone());
    
    // 3. 测试不同一致性级别
    println!("\n🧪 测试不同一致性级别...");
    
    // 测试强一致性
    let start = Instant::now();
    let result_strong = replicator.replicate(100u64, ConsistencyLevel::Strong);
    let duration_strong = start.elapsed();
    println!("  🔒 强一致性 (Strong): {:?} - 耗时: {:?}", result_strong, duration_strong);
    
    // 测试 Quorum 一致性
    let start = Instant::now();
    let result_quorum = replicator.replicate(200u64, ConsistencyLevel::Quorum);
    let duration_quorum = start.elapsed();
    println!("  📊 Quorum 一致性: {:?} - 耗时: {:?}", result_quorum, duration_quorum);
    
    // 测试最终一致性
    let start = Instant::now();
    let result_eventual = replicator.replicate(300u64, ConsistencyLevel::Eventual);
    let duration_eventual = start.elapsed();
    println!("  ⏰ 最终一致性 (Eventual): {:?} - 耗时: {:?}", result_eventual, duration_eventual);
    
    // 4. 批量操作测试
    println!("\n📦 批量操作测试...");
    let batch_data = vec![1000u64, 2000u64, 3000u64, 4000u64, 5000u64];
    
    for (i, data) in batch_data.iter().enumerate() {
        let start = Instant::now();
        let result = replicator.replicate(*data, ConsistencyLevel::Quorum);
        let duration = start.elapsed();
        println!("  📝 批量操作 {}: {:?} - 耗时: {:?}", i + 1, result, duration);
    }
    
    // 5. 性能统计
    println!("\n📈 性能统计:");
    println!("  🎯 强一致性: 最高延迟，最强保证");
    println!("  ⚖️  Quorum 一致性: 平衡延迟和保证");
    println!("  🚀 最终一致性: 最低延迟，最终保证");
    
    println!("\n✅ 分布式复制演示完成！");
    Ok(())
}

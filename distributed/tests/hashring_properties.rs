use c20_distributed::topology::ConsistentHashRing;
use std::collections::HashSet;

#[test]
fn unique_assignment_for_replicas_sequence() {
    let mut ring = ConsistentHashRing::new(16);
    ring.add_node("n1");
    ring.add_node("n2");
    ring.add_node("n3");
    // 多次取 nodes_for，检查不重复且数量匹配
    for key in ["k1", "k2", "k3", "k4", "k5"].iter() {
        let ns = ring.nodes_for(key, 2);
        let set: HashSet<_> = ns.iter().cloned().collect();
        assert_eq!(ns.len(), 2);
        assert_eq!(set.len(), 2);
    }
}

#[test]
fn rebalancing_moves_fraction() {
    let mut ring = ConsistentHashRing::new(16);
    ring.add_node("n1");
    ring.add_node("n2");
    let keys: Vec<_> = (0..200).map(|i| format!("k{i}")).collect();
    let before: Vec<_> = keys
        .iter()
        .map(|k| ring.route(k).unwrap().to_string())
        .collect();
    ring.add_node("n3");
    let after: Vec<_> = keys
        .iter()
        .map(|k| ring.route(k).unwrap().to_string())
        .collect();
    let moved = before
        .iter()
        .zip(after.iter())
        .filter(|(a, b)| *a != *b)
        .count();
    // 期望移动比例 ~ 1/3 左右，放宽阈值到 [10%, 50%] 以适配随机性
    let ratio = moved as f64 / keys.len() as f64;
    assert!(ratio > 0.10 && ratio < 0.50, "ratio={ratio}");
}

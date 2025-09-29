// 测试目的：管线/并行复制与幂等去重
// - 不变量：
//   1) 多数派达成后视为成功；
//   2) 幂等键重复提交不产生副作用；
//   3) 目标集合内默认 success=true 时，写入在 Quorum 下通过。
use distributed::consistency::ConsistencyLevel;
use distributed::replication::LocalReplicator;
use distributed::storage::InMemoryIdempotency;
use distributed::topology::ConsistentHashRing;

#[test]
fn pipeline_majority_with_idempotency() {
    let mut ring = ConsistentHashRing::new(16);
    let nodes = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
    for n in &nodes {
        ring.add_node(n);
    }

    let mut repl: LocalReplicator<String> = LocalReplicator::new(ring, nodes.clone())
        .with_idempotency(Box::new(InMemoryIdempotency::<String>::default()));

    // 默认 successes 为 true，模拟多数派成功
    let targets = nodes;
    let id = "op-1".to_string();
    repl.replicate_idempotent(&id, &targets, b"cmd".to_vec(), ConsistencyLevel::Quorum)
        .unwrap();
    // 再次提交不应重复
    repl.replicate_idempotent(&id, &targets, b"cmd".to_vec(), ConsistencyLevel::Quorum)
        .unwrap();
}

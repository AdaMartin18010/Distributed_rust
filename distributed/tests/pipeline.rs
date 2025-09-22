use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::replication::LocalReplicator;
use c20_distributed::storage::InMemoryIdempotency;
use c20_distributed::topology::ConsistentHashRing;

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

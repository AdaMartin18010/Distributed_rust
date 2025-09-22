use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::replication::{
    LocalReplicator,
    MajorityQuorum,
    //Replicator,
    QuorumPolicy,
};
use c20_distributed::topology::ConsistentHashRing;

fn build(nodes: &[&str]) -> (LocalReplicator<u64>, Vec<String>) {
    let mut ring = ConsistentHashRing::new(8);
    let mut v = Vec::new();
    for n in nodes {
        ring.add_node(n);
        v.push((*n).to_string());
    }
    let repl: LocalReplicator<u64> = LocalReplicator::new(ring, v.clone());
    (repl, v)
}

#[test]
fn required_acks_majority() {
    assert_eq!(
        MajorityQuorum::required_acks(1, ConsistencyLevel::Quorum),
        1
    );
    assert_eq!(
        MajorityQuorum::required_acks(2, ConsistencyLevel::Quorum),
        2
    );
    assert_eq!(
        MajorityQuorum::required_acks(3, ConsistencyLevel::Quorum),
        2
    );
    assert_eq!(
        MajorityQuorum::required_acks(5, ConsistencyLevel::Quorum),
        3
    );
    assert_eq!(
        MajorityQuorum::required_acks(4, ConsistencyLevel::Quorum),
        3
    );
    assert_eq!(
        MajorityQuorum::required_acks(5, ConsistencyLevel::Eventual),
        1
    );
}

#[test]
fn replicate_to_nodes_ok_and_err() {
    let (mut r, nodes) = build(&["n1", "n2", "n3", "n4", "n5"]);
    // 默认 successes 未设置，使用 unwrap_or(true) 视为成功 -> 足够多数
    let targets = nodes.clone();
    let res = r.replicate_to_nodes(&targets, 123u64, ConsistencyLevel::Quorum);
    assert!(res.is_ok());

    // 控制仅 2 个成功，N=5，Quorum=3 -> 应失败
    r.successes.clear();
    r.successes.insert(nodes[0].clone(), true);
    r.successes.insert(nodes[1].clone(), true);
    for i in 2..nodes.len() {
        r.successes.insert(nodes[i].clone(), false);
    }
    let res2 = r.replicate_to_nodes(&targets, 456u64, ConsistencyLevel::Quorum);
    assert!(res2.is_err());

    // Eventual 一致性下，任意一个成功即可
    let res3 = r.replicate_to_nodes(&targets, 789u64, ConsistencyLevel::Eventual);
    assert!(res3.is_ok());
}

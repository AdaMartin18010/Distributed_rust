use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::replication::{LocalReplicator, Replicator};
use c20_distributed::topology::ConsistentHashRing;

fn main() {
    let mut ring = ConsistentHashRing::new(8);
    ring.add_node("n1");
    ring.add_node("n2");
    ring.add_node("n3");
    let nodes = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
    let mut repl: LocalReplicator<u64> = LocalReplicator::new(ring, nodes.clone());
    // 默认全部成功，Quorum=2
    let r = repl.replicate(42u64, ConsistencyLevel::Quorum);
    println!("replicate quorum: {:?}", r);
}

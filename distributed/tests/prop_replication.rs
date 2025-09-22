use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::replication::{LocalReplicator, MajorityQuorum, QuorumPolicy};
use c20_distributed::topology::ConsistentHashRing;
use proptest::prelude::*;

proptest! {
    #[test]
    fn quorum_required_is_majority(total in 1usize..50) {
        let need_q = MajorityQuorum::required_acks(total, ConsistencyLevel::Quorum);
        prop_assert!(need_q >= (total/2 + 1));
        prop_assert!(need_q <= total);
    }
}

proptest! {
    #[test]
    fn replicate_succeeds_iff_acks_ge_need(total in 1usize..20, ok in 0usize..20) {
        let total = total.max(1);
        let ok = ok.min(total);

        let mut ring = ConsistentHashRing::new(8);
        let mut nodes = Vec::new();
        for i in 0..total { let n = format!("n{}", i); ring.add_node(&n); nodes.push(n); }

        let mut repl: LocalReplicator<u64> = LocalReplicator::new(ring, nodes.clone());
        repl.successes.clear();
        for i in 0..ok { repl.successes.insert(nodes[i].clone(), true); }
        for i in ok..total { repl.successes.insert(nodes[i].clone(), false); }

        let need = MajorityQuorum::required_acks(total, ConsistencyLevel::Quorum);
        let res = repl.replicate_to_nodes(&nodes, 1u64, ConsistencyLevel::Quorum);
        prop_assert_eq!(res.is_ok(), ok >= need);
    }
}

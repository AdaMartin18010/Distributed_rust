use c20_distributed::storage::{IdempotencyStore, InMemoryIdempotency};
use c20_distributed::topology::ConsistentHashRing;

#[test]
fn nodes_for_returns_distinct_nodes() {
    let mut ring = ConsistentHashRing::new(16);
    ring.add_node("n1");
    ring.add_node("n2");
    ring.add_node("n3");
    let v = ring.nodes_for(&"alpha", 2);
    assert!(v.len() == 2 && v[0] != v[1]);
}

#[test]
fn idempotency_prevents_duplicate() {
    let mut idem = InMemoryIdempotency::<String>::default();
    let id = "req-1".to_string();
    assert!(!idem.seen(&id));
    idem.record(id.clone());
    assert!(idem.seen(&id));
}

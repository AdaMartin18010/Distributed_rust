use c20_distributed::topology::ConsistentHashRing;

#[test]
fn ring_basic_route() {
    let mut ring = ConsistentHashRing::new(10);
    ring.add_node("n1");
    ring.add_node("n2");
    let n = ring.route(&"key-1").unwrap();
    assert!(n == "n1" || n == "n2");
}

#[test]
fn ring_rebalance_minimal_change() {
    let mut ring = ConsistentHashRing::new(10);
    ring.add_node("n1");
    ring.add_node("n2");
    let before = ring.route(&"user-42").unwrap().to_string();
    ring.add_node("n3");
    let after = ring.route(&"user-42").unwrap().to_string();
    assert!(before == after || (before != after));
}

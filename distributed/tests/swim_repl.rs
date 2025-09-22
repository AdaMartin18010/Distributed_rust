use c20_distributed::*;
use std::time::Duration;

struct DummyPing;
impl SwimTransport for DummyPing {
    fn ping(&self, _to: &str) -> bool {
        true
    }
    fn gossip(&self, _to: &str, _events: &[swim::SwimEvent]) -> bool {
        true
    }
}

#[test]
fn swim_probe_alive() {
    let node = swim::SwimNode::new("n0".into(), DummyPing).with_params(
        Duration::from_millis(500),
        Duration::from_millis(5000),
        2,
    );
    let ev = node.probe("n1");
    assert_eq!(ev.state, swim::SwimMemberState::Alive);
}

#[test]
fn majority_quorum_rules() {
    assert_eq!(
        replication::MajorityQuorum::required_acks(3, ConsistencyLevel::Quorum),
        2
    );
    assert_eq!(
        replication::MajorityQuorum::required_acks(5, ConsistencyLevel::Strong),
        3
    );
    assert_eq!(
        replication::MajorityQuorum::required_acks(4, ConsistencyLevel::Eventual),
        1
    );
}

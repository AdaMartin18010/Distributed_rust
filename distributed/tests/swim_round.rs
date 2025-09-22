use c20_distributed::swim::{SwimMemberState, SwimNode, SwimTransport};
use std::time::Duration;

struct T {
    ok: bool,
}
impl SwimTransport for T {
    fn ping(&self, _to: &str) -> bool {
        self.ok
    }
    fn gossip(&self, _to: &str, _events: &[c20_distributed::swim::SwimEvent]) -> bool {
        true
    }
}

#[test]
fn one_round_events_basic() {
    let node = SwimNode::new("n0".into(), T { ok: true }).with_params(
        Duration::from_millis(1000),
        Duration::from_millis(5000),
        3,
    );
    let peers = vec!["n1".to_string(), "n2".to_string()];
    for p in peers {
        let ev = node.probe(&p);
        assert_eq!(ev.state, SwimMemberState::Alive);
    }
}

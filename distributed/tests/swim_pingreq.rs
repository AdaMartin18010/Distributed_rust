use c20_distributed::swim::{SwimMemberState, SwimNode, SwimTransport};
use std::time::Duration;

struct StubTransport {
    fail_direct: bool,
    indirect_ok: bool,
}
impl SwimTransport for StubTransport {
    fn ping(&self, _to: &str) -> bool {
        !self.fail_direct
    }
    fn ping_req(&self, _relay: &str, _target: &str) -> bool {
        self.indirect_ok
    }
    fn gossip(&self, _to: &str, _events: &[c20_distributed::swim::SwimEvent]) -> bool {
        true
    }
}

#[test]
fn indirect_probe_can_rescue() {
    let node = SwimNode::new(
        "n0".into(),
        StubTransport {
            fail_direct: true,
            indirect_ok: true,
        },
    )
    .with_params(Duration::from_millis(1000), Duration::from_millis(5000), 3);
    let ev = node.probe_indirect("n1", ["n2", "n3"].iter().copied());
    assert_eq!(ev.state, SwimMemberState::Alive);
}

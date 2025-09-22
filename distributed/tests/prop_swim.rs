use c20_distributed::swim::{
    MemberInfo, MembershipView, SwimMemberState, SwimNode, SwimTransport, Version,
};
use proptest::prelude::*;
use std::time::Duration;

#[derive(Clone, Default)]
struct MockTrans {
    reachable: std::collections::HashMap<String, bool>,
}

impl SwimTransport for MockTrans {
    fn ping(&self, to: &str) -> bool {
        *self.reachable.get(to).unwrap_or(&true)
    }
    fn gossip(&self, _to: &str, _events: &[c20_distributed::swim::SwimEvent]) -> bool {
        true
    }
}

proptest! {
    #[test]
    fn direct_probe_reflects_reachability(name in "[a-z]{1,4}", ok in any::<bool>()) {
        let mut t = MockTrans::default();
        t.reachable.insert(name.clone(), ok);
        let node = SwimNode::new("me".to_string(), t).with_params(Duration::from_millis(0), Duration::from_millis(5000), 1);
        let ev = node.probe(&name);
        prop_assert_eq!(ev.node_id, name);
        prop_assert_eq!(ev.state, if ok { SwimMemberState::Alive } else { SwimMemberState::Suspect });
    }
}

proptest! {
    #[test]
    fn membership_merge_respects_version(moniker in "[a-z]{1,4}") {
        let mut a = MembershipView::new("me".to_string());
        let mut b = MembershipView::new("me".to_string());
        a.local_update(&moniker, SwimMemberState::Alive, 1);
        // b 比 a 新
        b.members.insert(moniker.clone(), MemberInfo { state: SwimMemberState::Faulty, version: Version(10), incarnation: 10, last_seen: std::time::SystemTime::now() });
        a.merge_from(&[(moniker.clone(), b.members.get(&moniker).unwrap().clone())]);
        prop_assert_eq!(a.members.get(&moniker).unwrap().state, SwimMemberState::Faulty);
    }
}

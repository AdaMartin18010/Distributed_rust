use c20_distributed::{
    EnhancedSwimTransport, MembershipView, SwimEvent, SwimMemberState, SwimNode, SwimTransport,
};
use std::time::Duration;

/// 简单的测试传输层
struct TestTransport {
    success_rate: f64,
}

impl TestTransport {
    fn new(success_rate: f64) -> Self {
        Self { success_rate }
    }
}

impl SwimTransport for TestTransport {
    fn ping(&self, _to: &str) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        let random = (hash % 100) as f64 / 100.0;

        random < self.success_rate
    }

    fn ping_req(&self, _relay: &str, _target: &str) -> bool {
        self.ping("")
    }

    fn ack(&self, _from: &str) -> bool {
        self.ping("")
    }

    fn gossip(&self, _to: &str, _events: &[SwimEvent]) -> bool {
        self.ping("")
    }
}

#[test]
fn test_enhanced_swim_node_creation() {
    let transport = TestTransport::new(0.9);
    let node = SwimNode::new("node1".to_string(), transport);

    assert_eq!(node.node_id, "node1");
    assert_eq!(node.get_incarnation(), 0);
    assert_eq!(node.fanout, 3);
    assert_eq!(node.indirect_probes, 3);
}

#[test]
fn test_swim_node_incarnation() {
    let transport = TestTransport::new(0.9);
    let node = SwimNode::new("node1".to_string(), transport);

    assert_eq!(node.get_incarnation(), 0);
    assert_eq!(node.increment_incarnation(), 1);
    assert_eq!(node.get_incarnation(), 1);
    assert_eq!(node.increment_incarnation(), 2);
    assert_eq!(node.get_incarnation(), 2);
}

#[test]
fn test_swim_probe() {
    let transport = TestTransport::new(1.0); // 100%成功率
    let node = SwimNode::new("node1".to_string(), transport);

    let event = node.probe("node2");
    assert_eq!(event.node_id, "node2");
    assert_eq!(event.state, SwimMemberState::Alive);
    assert_eq!(event.incarnation, 0);
}

#[test]
fn test_swim_probe_failure() {
    let transport = TestTransport::new(0.0); // 0%成功率
    let node = SwimNode::new("node1".to_string(), transport);

    let event = node.probe("node2");
    assert_eq!(event.node_id, "node2");
    assert_eq!(event.state, SwimMemberState::Suspect);
    assert_eq!(event.incarnation, 0);
}

#[test]
fn test_swim_indirect_probe() {
    let transport = TestTransport::new(0.0); // 直接探测失败
    let node = SwimNode::new("node1".to_string(), transport);

    let relays = vec!["relay1", "relay2"];
    let event = node.probe_indirect("target", relays);

    assert_eq!(event.node_id, "target");
    assert_eq!(event.state, SwimMemberState::Suspect);
}

#[test]
fn test_membership_view_creation() {
    let view = MembershipView::new("node1".to_string());

    assert_eq!(view.me, "node1");
    assert_eq!(view.size(), 0);
    assert_eq!(view.alive_count(), 0);
    assert!(view.alive_members().is_empty());
    assert!(view.suspect_members().is_empty());
    assert!(view.faulty_members().is_empty());
}

#[test]
fn test_membership_view_local_update() {
    let mut view = MembershipView::new("node1".to_string());

    view.local_update("node2", SwimMemberState::Alive, 1);

    assert_eq!(view.size(), 1);
    assert_eq!(view.alive_count(), 1);
    assert!(view.contains("node2"));
    assert!(!view.contains("node3"));

    let member = view.get_member("node2").unwrap();
    assert_eq!(member.state, SwimMemberState::Alive);
    assert_eq!(member.incarnation, 1);
}

#[test]
fn test_membership_view_update_from_event() {
    let mut view = MembershipView::new("node1".to_string());

    let event = SwimEvent::new("node2".to_string(), SwimMemberState::Alive, 1);
    let updated = view.update_from_event(&event);

    assert!(updated);
    assert_eq!(view.size(), 1);
    assert_eq!(view.alive_count(), 1);

    let member = view.get_member("node2").unwrap();
    assert_eq!(member.state, SwimMemberState::Alive);
    assert_eq!(member.incarnation, 1);
}

#[test]
fn test_membership_view_incarnation_check() {
    let mut view = MembershipView::new("node1".to_string());

    // 添加初始事件
    let event1 = SwimEvent::new("node2".to_string(), SwimMemberState::Alive, 1);
    view.update_from_event(&event1);

    // 尝试用更低的incarnation更新
    let event2 = SwimEvent::new("node2".to_string(), SwimMemberState::Suspect, 0);
    let updated = view.update_from_event(&event2);

    assert!(!updated); // 应该被拒绝
    let member = view.get_member("node2").unwrap();
    assert_eq!(member.state, SwimMemberState::Alive); // 状态应该保持不变
    assert_eq!(member.incarnation, 1);
}

#[test]
fn test_membership_view_state_transitions() {
    let mut view = MembershipView::new("node1".to_string());

    // Alive -> Suspect -> Faulty
    view.local_update("node2", SwimMemberState::Alive, 1);
    assert_eq!(view.alive_count(), 1);
    assert!(view.alive_members().contains(&"node2".to_string()));

    view.local_update("node2", SwimMemberState::Suspect, 2);
    assert_eq!(view.alive_count(), 0);
    assert!(view.suspect_members().contains(&"node2".to_string()));

    view.local_update("node2", SwimMemberState::Faulty, 3);
    assert_eq!(view.alive_count(), 0);
    assert!(view.faulty_members().contains(&"node2".to_string()));
}

#[test]
fn test_enhanced_swim_transport() {
    let transport =
        EnhancedSwimTransport::new(Duration::from_millis(100), 3, Duration::from_millis(10));

    // 测试基本功能（由于是随机模拟，我们只测试调用不会panic）
    let _result1 = transport.ping("node1");
    let _result2 = transport.ping_req("relay", "target");
    let _result3 = transport.ack("node1");
    let _result4 = transport.gossip("node1", &[]);
}

#[test]
fn test_swim_node_suspect_timeouts() {
    let transport = TestTransport::new(0.0);
    let mut node = SwimNode::new("node1".to_string(), transport).with_params(
        Duration::from_millis(100),
        Duration::from_millis(50), // 很短的超时时间
        3,
    );

    // 添加可疑节点
    node.suspect_timers
        .insert("node2".to_string(), std::time::Instant::now());

    // 等待超时
    std::thread::sleep(Duration::from_millis(100));

    let events = node.check_suspect_timeouts();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].node_id, "node2");
    assert_eq!(events[0].state, SwimMemberState::Faulty);
}

#[test]
fn test_swim_node_handle_events() {
    let transport = TestTransport::new(0.9);
    let mut node = SwimNode::new("node1".to_string(), transport);

    // 处理Alive事件
    let alive_event = SwimEvent::new("node2".to_string(), SwimMemberState::Alive, 1);
    let handled = node.handle_swim_event(&alive_event);
    assert!(handled);
    assert!(!node.suspect_timers.contains_key("node2"));

    // 处理Suspect事件
    let suspect_event = SwimEvent::new("node3".to_string(), SwimMemberState::Suspect, 1);
    let handled = node.handle_swim_event(&suspect_event);
    assert!(handled);
    assert!(node.suspect_timers.contains_key("node3"));

    // 处理Faulty事件
    let faulty_event = SwimEvent::new("node4".to_string(), SwimMemberState::Faulty, 1);
    let handled = node.handle_swim_event(&faulty_event);
    assert!(handled);
    assert!(!node.suspect_timers.contains_key("node4"));
    assert!(!node.last_probe_time.contains_key("node4"));
}

// 测试目的：
// - 验证 SWIM 视图在更高 incarnation/version 到达后能收敛到更新状态。
// 关键不变量：
// - 版本单调性：成员条目 (incarnation, version) 单调推进；较新事件覆盖较旧事件。
use distributed::swim::{MembershipView, SwimMemberState};

#[test]
fn swim_view_converges_on_higher_version() {
    let mut a = MembershipView::new("a".into());
    a.local_update("n1", SwimMemberState::Alive, 1);
    let mut b = a.clone();
    b.local_update("n1", SwimMemberState::Suspect, 2);
    let payload = b.gossip_payload();
    a.merge_from(&payload);
    let n1 = a.members.get("n1").unwrap();
    assert_eq!(n1.state, SwimMemberState::Suspect);
    assert!(n1.version.0 >= 2);
}

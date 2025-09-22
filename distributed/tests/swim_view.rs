use c20_distributed::swim::{MembershipView, SwimMemberState};

#[test]
fn view_merge_prefers_newer_version() {
    let mut a = MembershipView::new("a".into());
    a.local_update("n1", SwimMemberState::Alive, 1);
    let mut b = a.clone();
    // b bumps n1 to Suspect with higher version
    b.local_update("n1", SwimMemberState::Suspect, 2);
    // a merges from b
    let payload = b.gossip_payload();
    a.merge_from(&payload);
    let n1 = a.members.get("n1").unwrap();
    assert_eq!(n1.state, SwimMemberState::Suspect);
    assert_eq!(n1.version.0, 2);
}

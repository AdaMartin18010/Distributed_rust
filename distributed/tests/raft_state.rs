// 测试目的：角色/任期单调不变量
// - 不变量：
//   1) 接收到更大任期的 RPC 后，任期单调不减且保持/降级为 Follower；
//   2) 在相同或更小任期下不得提升任期；
//   3) 对无效请求返回当前任期。
#[cfg(feature = "consensus-raft")]
mod raft_state_flow {
    use c20_distributed::*;

    #[test]
    fn follower_accepts_newer_term_append() {
        let mut r: MinimalRaft<Vec<u8>> = MinimalRaft::new();
        let req = AppendEntriesReq {
            term: Term(1),
            leader_id: "n1".into(),
            prev_log_index: LogIndex(0),
            prev_log_term: Term(0),
            entries: vec![],
            leader_commit: LogIndex(0),
        };
        let resp = r.handle_append_entries(req).unwrap();
        assert!(resp.success);
        assert_eq!(r.state(), RaftState::Follower);
        assert_eq!(r.current_term().0, 1);
    }
}

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

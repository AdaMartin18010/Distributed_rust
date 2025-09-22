#[cfg(feature = "consensus-raft")]
mod raft_log {
    use c20_distributed::consensus_raft::{
        AppendEntriesReq, AppendEntriesResp, LogIndex, MinimalRaft, RaftNode, Term,
    };

    #[test]
    fn append_with_prev_check_and_overwrite() {
        let mut r: MinimalRaft<Vec<u8>> = MinimalRaft::new();
        // first append at prev=0
        let a1 = AppendEntriesReq {
            term: Term(1),
            leader_id: "n1".into(),
            prev_log_index: LogIndex(0),
            prev_log_term: Term(0),
            entries: vec![b"a".to_vec(), b"b".to_vec()],
            leader_commit: LogIndex(0),
        };
        let AppendEntriesResp { success, .. } = r.handle_append_entries(a1).unwrap();
        assert!(success);

        // conflicting append at prev=2 but older term should still match prev term 1
        let a2 = AppendEntriesReq {
            term: Term(1),
            leader_id: "n1".into(),
            prev_log_index: LogIndex(2),
            prev_log_term: Term(1),
            entries: vec![b"c".to_vec()],
            leader_commit: LogIndex(0),
        };
        let resp2 = r.handle_append_entries(a2).unwrap();
        assert!(resp2.success);

        // newer term, overwrite from prev=1
        let a3 = AppendEntriesReq {
            term: Term(2),
            leader_id: "n1".into(),
            prev_log_index: LogIndex(1),
            prev_log_term: Term(1),
            entries: vec![b"x".to_vec(), b"y".to_vec()],
            leader_commit: LogIndex(0),
        };
        let resp3 = r.handle_append_entries(a3).unwrap();
        assert!(resp3.success);
    }
}

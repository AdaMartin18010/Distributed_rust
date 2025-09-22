#[cfg(feature = "consensus-raft")]
mod raft_commit {
    use c20_distributed::consensus_raft::{
        AppendEntriesReq, LogIndex, MinimalRaft, RaftNode, Term,
    };
    use std::sync::{Arc, Mutex};

    #[test]
    fn apply_happens_up_to_leader_commit() {
        let applied: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let capture = applied.clone();
        let mut r: MinimalRaft<Vec<u8>> = MinimalRaft::new();
        r.set_apply(Box::new(move |e| {
            capture.lock().unwrap().push(e.clone());
        }));

        // append two entries, leader commit=1 -> apply only first
        let req = AppendEntriesReq {
            term: Term(1),
            leader_id: "n1".into(),
            prev_log_index: LogIndex(0),
            prev_log_term: Term(0),
            entries: vec![b"a".to_vec(), b"b".to_vec()],
            leader_commit: LogIndex(1),
        };
        let _ = r.handle_append_entries(req);
        assert_eq!(applied.lock().unwrap().len(), 1);

        // advance commit to 2 -> apply second
        let req2 = AppendEntriesReq {
            term: Term(1),
            leader_id: "n1".into(),
            prev_log_index: LogIndex(2),
            prev_log_term: Term(1),
            entries: vec![],
            leader_commit: LogIndex(2),
        };
        let _ = r.handle_append_entries(req2);
        assert_eq!(applied.lock().unwrap().len(), 2);
    }
}

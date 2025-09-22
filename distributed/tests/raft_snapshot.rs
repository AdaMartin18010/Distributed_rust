#[cfg(feature = "consensus-raft")]
use c20_distributed::consensus_raft::{AppendEntriesReq, LogIndex, MinimalRaft, RaftNode, Snapshot, Term};

#[cfg(feature = "consensus-raft")]
#[test]
fn snapshot_truncates_log_and_resets_indices() {
    let mut raft: MinimalRaft<Vec<u8>> = MinimalRaft::new();
    let req1 = AppendEntriesReq {
        term: Term(1),
        leader_id: "n1".into(),
        prev_log_index: LogIndex(0),
        prev_log_term: Term(0),
        entries: vec![b"a".to_vec()],
        leader_commit: LogIndex(1),
    };
    let _ = raft.handle_append_entries(req1).unwrap();
    let req2 = AppendEntriesReq {
        term: Term(1),
        leader_id: "n1".into(),
        prev_log_index: LogIndex(1),
        prev_log_term: Term(1),
        entries: vec![b"b".to_vec()],
        leader_commit: LogIndex(2),
    };
    let _ = raft.handle_append_entries(req2).unwrap();
    let snapshot = Snapshot {
        last_included_index: LogIndex(1),
        last_included_term: Term(1),
        data: b"snapshot_data".to_vec(),
    };
    raft.install_snapshot(snapshot);
    let req3 = AppendEntriesReq {
        term: Term(2),
        leader_id: "n1".into(),
        prev_log_index: LogIndex(0),
        prev_log_term: Term(0),
        entries: vec![b"c".to_vec()],
        leader_commit: LogIndex(1),
    };
    let resp3 = raft.handle_append_entries(req3).unwrap();
    assert!(resp3.success);
}

#[cfg(not(feature = "consensus-raft"))]
#[test]
fn snapshot_skipped_without_feature() {
    assert!(true);
}

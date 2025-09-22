#[cfg(feature = "consensus-raft")]
use c20_distributed::consensus_raft::{AppendEntriesReq, LogIndex, MinimalRaft, RaftNode, Term};
#[cfg(feature = "consensus-raft")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "consensus-raft")]
#[test]
fn read_after_commit_is_visible_via_apply_barrier() {
    let mut raft: MinimalRaft<Vec<u8>> = MinimalRaft::new();
    let applied: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let capture = applied.clone();
    raft.set_apply(Box::new(move |e: &Vec<u8>| {
        capture.lock().unwrap().push(e.clone());
    }));

    let req = AppendEntriesReq {
        term: Term(1),
        leader_id: "n1".into(),
        prev_log_index: LogIndex(0),
        prev_log_term: Term(0),
        entries: vec![b"x".to_vec()],
        leader_commit: LogIndex(1),
    };
    let resp = raft.handle_append_entries(req).unwrap();
    assert!(resp.success);

    assert_eq!(applied.lock().unwrap().len(), 1);
    assert_eq!(applied.lock().unwrap()[0], b"x".to_vec());
}

#[cfg(feature = "consensus-raft")]
#[test]
fn multiple_commits_are_applied_in_order() {
    let mut raft: MinimalRaft<Vec<u8>> = MinimalRaft::new();
    let applied: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let capture = applied.clone();
    raft.set_apply(Box::new(move |e: &Vec<u8>| {
        capture.lock().unwrap().push(e.clone());
    }));

    // 先追加两条但只提交第一条
    let req1 = AppendEntriesReq {
        term: Term(1),
        leader_id: "n1".into(),
        prev_log_index: LogIndex(0),
        prev_log_term: Term(0),
        entries: vec![b"a".to_vec(), b"b".to_vec()],
        leader_commit: LogIndex(1),
    };
    let resp1 = raft.handle_append_entries(req1).unwrap();
    assert!(resp1.success);
    assert_eq!(applied.lock().unwrap().as_slice(), &[b"a".to_vec()]);

    // 再提交第二条
    let req2 = AppendEntriesReq {
        term: Term(1),
        leader_id: "n1".into(),
        prev_log_index: LogIndex(2),
        prev_log_term: Term(1),
        entries: vec![],
        leader_commit: LogIndex(2),
    };
    let resp2 = raft.handle_append_entries(req2).unwrap();
    assert!(resp2.success);
    assert_eq!(applied.lock().unwrap().as_slice(), &[b"a".to_vec(), b"b".to_vec()]);
}

#[cfg(feature = "consensus-raft")]
#[test]
fn append_fails_when_prev_log_mismatch() {
    let mut raft: MinimalRaft<Vec<u8>> = MinimalRaft::new();
    // 不设置 apply 也应安全
    let req = AppendEntriesReq {
        term: Term(1),
        leader_id: "n1".into(),
        prev_log_index: LogIndex(1), // 期望有一条，但当前日志为空
        prev_log_term: Term(1),
        entries: vec![b"z".to_vec()],
        leader_commit: LogIndex(1),
    };
    let resp = raft.handle_append_entries(req).unwrap();
    assert!(!resp.success);
}

#[cfg(feature = "consensus-raft")]
#[test]
fn scoped_apply_allows_non_static_callback() {
    let mut raft: MinimalRaft<Vec<u8>> = MinimalRaft::new();
    let mut buf: Vec<Vec<u8>> = Vec::new();
    {
        let mut cb = |e: &Vec<u8>| buf.push(e.clone());
        let mut guard = raft.set_apply_scoped(&mut cb);
        let req = AppendEntriesReq {
            term: Term(1),
            leader_id: "n1".into(),
            prev_log_index: LogIndex(0),
            prev_log_term: Term(0),
            entries: vec![b"s".to_vec()],
            leader_commit: LogIndex(1),
        };
        let resp = guard.handle_append_entries(req).unwrap();
        assert!(resp.success);
    }
    assert_eq!(buf, vec![b"s".to_vec()]);
}

#[cfg(not(feature = "consensus-raft"))]
#[test]
fn read_after_commit_skipped_without_feature() {
    // feature 未启用时使用桩测试，保持通过以不阻断 CI
    assert!(true);
}

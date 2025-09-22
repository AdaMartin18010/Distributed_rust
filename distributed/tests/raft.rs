#[cfg(feature = "consensus-raft")]
mod raft_smoke {
    use c20_distributed::*;

    #[test]
    fn api_exports_exist() {
        let _s = RaftState::Follower;
        let _t = Term(1);
        let _i = LogIndex(0);
        let _ = AppendEntriesResp {
            term: _t,
            success: true,
        };
    }
}

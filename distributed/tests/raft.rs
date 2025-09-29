// 测试目的：
// - 验证 Raft 最小接口可用性（API 导出与基本类型/消息存在）。
// 关键不变量（针对更完整用例）：
// - 任期单调不减、前缀匹配、提交单调、领导者唯一性（此处仅做烟囱检查）。
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

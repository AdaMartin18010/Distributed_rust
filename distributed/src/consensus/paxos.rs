//! Paxos 接口骨架
//!
//! 设计意图：
//! - 提供角色与 API 占位，后续引入 Prepare/Promise、Accept/Accepted、Learn 阶段。
//! - 通过多数派交叠性质保证唯一选择值；在工程化实现中接入稳定存储与重试策略。
//!
//! 安全要点（草图）：
//! - 多数派交叠：任意两个多数派必有非空交集，确保已接受值在更高提案编号被沿袭。
//! - 提案编号单调：更高编号的提案需承诺继承已知最高的已接受值，防止冲突。
//!
//! 参考：见 `consensus::mod` 顶部列表（Lamport 1998；Chandra et al. 2007）。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusRole {
    Leader,
    Follower,
    Candidate,
}

pub trait ConsensusApi {
    fn role(&self) -> ConsensusRole;
}

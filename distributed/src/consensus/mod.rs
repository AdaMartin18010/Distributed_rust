//! 共识算法模块
//!
//! 目标与范围：
//! - 提供典型共识协议的最小可用骨架与可扩展接口（Raft、Paxos、PBFT）。
//! - 统一暴露基本角色、法定人数与消息处理抽象，便于在更高层编排（复制、事务、调度）。
//!
//! 核心性质与术语（非正式）：
//! - 安全性（Safety）：不产生冲突提交；同一索引最多有一个提交值。
//! - 活性（Liveness）：在足够弱的失败与网络条件下，提交最终发生。
//! - 一致性模型：Raft 与 Paxos 均可实现线性一致性；PBFT 在 f < n/3 条件下保证安全与活性。
//!
//! 形式化不变量（草图）：
//! - 单调任期不变量：任期 `term` 在本地与消息中均单调不减。
//! - 前缀匹配不变量：若条目 (index, term) 在多数派已提交，则任何合法领导者的日志在该 index 处匹配该 term。
//! - PBFT 法定人数覆盖：任意两组大小 ≥ 2f+1 的集合必然交叠，从而保证证书唯一性。
//!
//! 证明线索与参考：
//! - Raft: 通过选举限制（任期内至多一个领导者）与日志复制规则推导提交唯一性；见 [Ongaro & Ousterhout, 2014].
//! - Paxos: 利用多数派交叠性质证明唯一提议被选择；见 [Lamport, 1998].
//! - PBFT: 通过三阶段提交与视图变更证书，证明在 f < n/3 下安全与活性；见 [Castro & Liskov, 1999].
//!
//! 进一步阅读：
//! - Ongaro, D., Ousterhout, J. In Search of an Understandable Consensus Algorithm (Raft), 2014.
//! - Lamport, L. The Part-Time Parliament (Paxos), ACM TOCS, 1998.
//! - Castro, M., Liskov, B. Practical Byzantine Fault Tolerance, OSDI, 1999.
//! - Chandra, T. D., Griesemer, R., Redstone, J. Paxos Made Live, PODC, 2007.
//! - Howard, H. et al. Raft Refloated, 2015.

pub mod raft;
pub mod paxos;
pub mod byzantine;

pub use raft::*;
pub use paxos::*;
pub use byzantine::*;

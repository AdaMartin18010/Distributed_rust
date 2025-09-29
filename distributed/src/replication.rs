//! 复制与仲裁模块
//!
//! 设计目标：
//! - 提供最小可用的复制接口 `Replicator` 与多数派（及可插拔）仲裁策略，支持不同一致性级别映射。
//! - 支持一次性幂等控制（`IdempotencyStore`）以避免重放副作用。
//!
//! 不变量与性质（草图）：
//! - 多数派交叠：当读/写均为多数派时，`R + W > N` 保证读取必与最新一次提交写操作交叠，实现线性化读取。
//! - 幂等性：若 `idempotency.seen(id)` 为真，则重复的相同指令不应再次产生副作用。
//! - 失败封装：复制结果由 `required_acks(total, level)` 决定，当收到的确认数 ≥ 需求值时视为成功。
//!
//! 工程化注意：
//! - 网络/节点错误需要与重试策略配套；一次写入的“副作用是否可重试”需由上层定义。
//! - 一致性级别到 `required_acks` 的映射在此为示例，可按产品语义调整。
//! - 读/写分离策略可使用 `CompositeQuorum<R,W>` 实现 `R ≠ W` 的灵活配置。
//!
//! 参考：
//! - Vogels, W. Eventually Consistent, 2009.
//! - Gilbert & Lynch, Brewer’s Conjecture and the Feasibility of Consistent, Available, Partition-Tolerant Web Services, 2002.
//! - Amazon Dynamo 与 Riak 文献对 `R/W/N` 模型的实践。
use crate::consistency::ConsistencyLevel;
use crate::errors::DistributedError;
use crate::storage::IdempotencyStore;
use crate::topology::ConsistentHashRing;

pub trait Replicator<C> {
    fn replicate(&mut self, command: C, level: ConsistencyLevel) -> Result<(), DistributedError>;
}

pub trait QuorumPolicy {
    fn required_acks(total: usize, level: ConsistencyLevel) -> usize;
}

pub struct MajorityQuorum;

impl QuorumPolicy for MajorityQuorum {
    fn required_acks(total: usize, level: ConsistencyLevel) -> usize {
        match level {
            ConsistencyLevel::Strong
            | ConsistencyLevel::Linearizable
            | ConsistencyLevel::Quorum => (total / 2) + 1,
            ConsistencyLevel::Sequential | ConsistencyLevel::Causal => (total / 2) + 1,
            ConsistencyLevel::Session
            | ConsistencyLevel::MonotonicRead
            | ConsistencyLevel::MonotonicWrite => (total / 2) + 1,
            ConsistencyLevel::ReadYourWrites
            | ConsistencyLevel::MonotonicReads
            | ConsistencyLevel::MonotonicWrites
            | ConsistencyLevel::WritesFollowReads
            | ConsistencyLevel::CausalConsistency => (total / 2) + 1,
            ConsistencyLevel::StrongEventual => 1,
            ConsistencyLevel::Eventual => 1,
        }
    }
}

// ---------------- Read/Write 可插拔仲裁（不破坏现有 API） ----------------

pub trait ReadQuorumPolicy {
    fn required_read_acks(total: usize, level: ConsistencyLevel) -> usize;
}

pub trait WriteQuorumPolicy {
    fn required_write_acks(total: usize, level: ConsistencyLevel) -> usize;
}

pub struct MajorityRead;
pub struct MajorityWrite;

impl ReadQuorumPolicy for MajorityRead {
    fn required_read_acks(total: usize, level: ConsistencyLevel) -> usize {
        MajorityQuorum::required_acks(total, level)
    }
}

impl WriteQuorumPolicy for MajorityWrite {
    fn required_write_acks(total: usize, level: ConsistencyLevel) -> usize {
        MajorityQuorum::required_acks(total, level)
    }
}

/// 读/写仲裁可分别配置的组合策略
pub struct CompositeQuorum<R, W> {
    _r: std::marker::PhantomData<R>,
    _w: std::marker::PhantomData<W>,
}

impl<R, W> CompositeQuorum<R, W> {
    pub fn required_read(total: usize, level: ConsistencyLevel) -> usize
    where
        R: ReadQuorumPolicy,
    {
        R::required_read_acks(total, level)
    }

    pub fn required_write(total: usize, level: ConsistencyLevel) -> usize
    where
        W: WriteQuorumPolicy,
    {
        W::required_write_acks(total, level)
    }
}

use std::collections::HashMap;

pub struct LocalReplicator<ID> {
    pub ring: ConsistentHashRing,
    pub nodes: Vec<String>,
    pub successes: HashMap<String, bool>,
    pub idempotency: Option<Box<dyn IdempotencyStore<ID> + Send>>,
}

impl<ID> LocalReplicator<ID> {
    pub fn new(ring: ConsistentHashRing, nodes: Vec<String>) -> Self {
        Self {
            ring,
            nodes,
            successes: HashMap::new(),
            idempotency: None,
        }
    }

    pub fn with_idempotency(mut self, store: Box<dyn IdempotencyStore<ID> + Send>) -> Self {
        self.idempotency = Some(store);
        self
    }

    pub fn replicate_to_nodes<C: Clone>(
        &mut self,
        targets: &[String],
        _command: C,
        level: ConsistencyLevel,
    ) -> Result<(), DistributedError> {
        let total = targets.len();
        let need = MajorityQuorum::required_acks(total, level);
        let mut acks = 0usize;
        for n in targets {
            if *self.successes.get(n).unwrap_or(&true) {
                acks += 1;
            }
        }
        if acks >= need {
            Ok(())
        } else {
            Err(DistributedError::Network(format!("acks {acks}/{need}")))
        }
    }

    pub fn replicate_idempotent<C: Clone>(
        &mut self,
        id: &ID,
        targets: &[String],
        command: C,
        level: ConsistencyLevel,
    ) -> Result<(), DistributedError>
    where
        ID: Clone,
    {
        if let Some(store) = &self.idempotency
            && store.seen(id) {
                return Ok(());
            }
        let res = self.replicate_to_nodes(targets, command, level);
        if res.is_ok()
            && let Some(store) = &mut self.idempotency {
                store.record(id.clone());
            }
        res
    }
}

impl<C: Clone, ID> Replicator<C> for LocalReplicator<ID> {
    fn replicate(&mut self, command: C, level: ConsistencyLevel) -> Result<(), DistributedError> {
        let nodes = self.nodes.clone();
        self.replicate_to_nodes(&nodes, command, level)
    }
}

//! 分区与路由
//!
//! 目标：
//! - 提供基于哈希的分区器与一致性哈希路由器，支撑键到分片/节点的确定性映射。
//!
//! 性质（草图）：
//! - 均衡性：`ahash` 作为快速哈希，结合足够分片或虚拟节点可近似均衡。
//! - 稳定性：拓扑小幅变更时，受影响键的比例较低（与一致性哈希性质相关）。
//!
//! 参考：Dynamo/Riak 分区与副本放置文献。
use crate::core::topology::{ConsistentHashRing, ShardId};
use std::hash::{Hash, Hasher};

pub trait Partitioner<K> {
    fn shard_of(&self, key: &K) -> ShardId;
}

pub struct HashPartitioner {
    pub shard_count: u64,
}

impl<K: Hash> Partitioner<K> for HashPartitioner {
    fn shard_of(&self, key: &K) -> ShardId {
        let mut hasher = ahash::AHasher::default();
        key.hash(&mut hasher);
        let v = hasher.finish() % self.shard_count;
        ShardId(v)
    }
}

pub struct HashRingRouter {
    pub ring: ConsistentHashRing,
}

impl HashRingRouter {
    pub fn new(ring: ConsistentHashRing) -> Self {
        Self { ring }
    }
    pub fn owner_of<K: std::hash::Hash>(&self, key: &K) -> Option<String> {
        self.ring.route(key).map(|s| s.to_string())
    }
}

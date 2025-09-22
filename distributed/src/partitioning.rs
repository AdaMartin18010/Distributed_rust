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

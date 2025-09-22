#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShardId(pub u64);

#[derive(Debug, Clone)]
pub struct ClusterTopology {
    pub shard_count: u64,
}

impl ClusterTopology {
    pub fn shards(&self) -> impl Iterator<Item = ShardId> + '_ {
        (0..self.shard_count).map(ShardId)
    }
}

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ConsistentHashRing {
    ring: BTreeMap<u64, String>,
    replicas: u32,
}

impl ConsistentHashRing {
    pub fn new(replicas: u32) -> Self {
        Self {
            ring: BTreeMap::new(),
            replicas,
        }
    }

    pub fn add_node(&mut self, node: &str) {
        for r in 0..self.replicas {
            let mut h = ahash::AHasher::default();
            (node, r).hash(&mut h);
            self.ring.insert(h.finish(), node.to_string());
        }
    }

    pub fn remove_node(&mut self, node: &str) {
        let mut keys = Vec::new();
        for r in 0..self.replicas {
            let mut h = ahash::AHasher::default();
            (node, r).hash(&mut h);
            keys.push(h.finish());
        }
        for k in keys {
            self.ring.remove(&k);
        }
    }

    pub fn route<K: Hash>(&self, key: &K) -> Option<&str> {
        if self.ring.is_empty() {
            return None;
        }
        let mut h = ahash::AHasher::default();
        key.hash(&mut h);
        let k = h.finish();
        let (_, node) = self
            .ring
            .range(k..)
            .next()
            .or_else(|| self.ring.iter().next())
            .unwrap();
        Some(node.as_str())
    }

    pub fn nodes_for<K: Hash>(&self, key: &K, replicas: usize) -> Vec<String> {
        if self.ring.is_empty() || replicas == 0 {
            return Vec::new();
        }
        let mut h = ahash::AHasher::default();
        key.hash(&mut h);
        let k = h.finish();
        let mut res = Vec::with_capacity(replicas);
        let mut seen = std::collections::HashSet::new();
        for (_, n) in self.ring.range(k..).chain(self.ring.iter()) {
            if seen.insert(n) {
                res.push(n.clone());
                if res.len() == replicas {
                    break;
                }
            }
        }
        res
    }
}

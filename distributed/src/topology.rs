//! 拓扑与一致性哈希
//!
//! 目标：
//! - 提供分片标识与一致性哈希环，支持节点增删与副本候选选择（`nodes_for`）。
//! - 与 `partitioning.rs`/`replication.rs`、负载均衡策略协同使用。
//!
//! 不变量与性质（草图）：
//! - 环有序性：`BTreeMap` 保持虚拟节点按哈希排序；路由按 `range(k..)` 回落至首元素实现环回。
//! - 迁移上界：单节点变更引发的键迁移期望占比 O(1/replicas)；`nodes_for` 去重保证副本唯一。
//! - 哈希稳定性：同一 key 与 ring 状态下路由稳定；引入虚拟节点以平衡分布。
//!
//! 参考：Karger 等 Consistent Hashing 论文；Jump Consistent Hash。
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

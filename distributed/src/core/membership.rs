use serde::{Deserialize, Serialize};

pub type ClusterNodeId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterMembership {
    pub nodes: Vec<ClusterNodeId>,
}

impl ClusterMembership {
    pub fn is_member(&self, node: &ClusterNodeId) -> bool {
        self.nodes.iter().any(|n| n == node)
    }
}

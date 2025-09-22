use crate::consistency::ConsistencyLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// 拜占庭容错节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ByzantineNodeState {
    /// 正常节点
    Honest,
    /// 拜占庭节点（恶意或故障）
    Byzantine,
    /// 未知状态
    Unknown,
}

/// 拜占庭容错消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByzantineMessage {
    /// 请求消息
    Request {
        id: String,
        content: Vec<u8>,
        timestamp: SystemTime,
        sender: String,
    },
    /// 准备消息（PBFT第一阶段）
    Prepare {
        view: u64,
        sequence: u64,
        digest: String,
        sender: String,
        timestamp: SystemTime,
    },
    /// 预提交消息（PBFT第二阶段）
    PreCommit {
        view: u64,
        sequence: u64,
        digest: String,
        sender: String,
        timestamp: SystemTime,
    },
    /// 提交消息（PBFT第三阶段）
    Commit {
        view: u64,
        sequence: u64,
        digest: String,
        sender: String,
        timestamp: SystemTime,
    },
    /// 视图变更消息
    ViewChange {
        new_view: u64,
        sender: String,
        prepared_certificates: Vec<PreparedCertificate>,
        timestamp: SystemTime,
    },
    /// 新视图消息
    NewView {
        new_view: u64,
        sender: String,
        view_change_certificates: Vec<ViewChangeCertificate>,
        timestamp: SystemTime,
    },
}

/// 准备证书
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedCertificate {
    pub view: u64,
    pub sequence: u64,
    pub digest: String,
    pub prepare_messages: Vec<ByzantineMessage>,
}

/// 视图变更证书
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewChangeCertificate {
    pub new_view: u64,
    pub view_change_messages: Vec<ByzantineMessage>,
}

/// PBFT（Practical Byzantine Fault Tolerance）实现
#[derive(Debug, Clone)]
pub struct PBFTNode {
    pub node_id: String,
    pub view: u64,
    pub sequence: u64,
    pub node_states: HashMap<String, ByzantineNodeState>,
    pub prepared_certificates: HashMap<String, PreparedCertificate>,
    pub committed_requests: HashMap<String, Vec<u8>>,
    pub pending_requests: HashMap<String, ByzantineMessage>,
    pub max_faulty_nodes: usize,
    pub total_nodes: usize,
}

impl PBFTNode {
    /// 创建新的PBFT节点
    pub fn new(node_id: String, total_nodes: usize) -> Self {
        let max_faulty_nodes = (total_nodes - 1) / 3; // PBFT要求最多1/3的节点是拜占庭节点

        Self {
            node_id,
            view: 0,
            sequence: 0,
            node_states: HashMap::new(),
            prepared_certificates: HashMap::new(),
            committed_requests: HashMap::new(),
            pending_requests: HashMap::new(),
            max_faulty_nodes,
            total_nodes,
        }
    }

    /// 检查是否满足拜占庭容错要求
    pub fn is_byzantine_fault_tolerant(&self) -> bool {
        self.total_nodes > 3 * self.max_faulty_nodes && self.total_nodes >= 4
    }

    /// 获取法定人数大小
    pub fn quorum_size(&self) -> usize {
        2 * self.max_faulty_nodes + 1
    }

    /// 处理请求消息
    pub fn handle_request(
        &mut self,
        message: ByzantineMessage,
    ) -> Result<Vec<ByzantineMessage>, String> {
        if let ByzantineMessage::Request { id, content, .. } = message.clone() {
            // 检查是否是主节点
            if self.is_primary() {
                self.sequence += 1;
                let digest = self.compute_digest(&content);

                // 创建准备消息
                let prepare_message = ByzantineMessage::Prepare {
                    view: self.view,
                    sequence: self.sequence,
                    digest,
                    sender: self.node_id.clone(),
                    timestamp: SystemTime::now(),
                };

                self.pending_requests.insert(id.clone(), message);

                Ok(vec![prepare_message])
            } else {
                Err("只有主节点可以处理请求".to_string())
            }
        } else {
            Err("无效的消息类型".to_string())
        }
    }

    /// 处理准备消息
    pub fn handle_prepare(
        &mut self,
        message: ByzantineMessage,
    ) -> Result<Vec<ByzantineMessage>, String> {
        if let ByzantineMessage::Prepare {
            view,
            sequence,
            digest,
            ..
        } = message.clone()
        {
            // 验证消息
            if !self.validate_prepare_message(&message) {
                return Err("无效的准备消息".to_string());
            }

            // 收集准备消息
            let key = format!("{}-{}", view, sequence);
            let certificate = self
                .prepared_certificates
                .entry(key.clone())
                .or_insert_with(|| PreparedCertificate {
                    view,
                    sequence,
                    digest: digest.clone(),
                    prepare_messages: Vec::new(),
                });

            certificate.prepare_messages.push(message);

            // 检查是否收集到足够的准备消息
            if certificate.prepare_messages.len() >= self.quorum_size() {
                // 创建预提交消息
                let pre_commit_message = ByzantineMessage::PreCommit {
                    view,
                    sequence,
                    digest,
                    sender: self.node_id.clone(),
                    timestamp: SystemTime::now(),
                };

                Ok(vec![pre_commit_message])
            } else {
                Ok(vec![])
            }
        } else {
            Err("无效的消息类型".to_string())
        }
    }

    /// 处理预提交消息
    pub fn handle_pre_commit(
        &mut self,
        message: ByzantineMessage,
    ) -> Result<Vec<ByzantineMessage>, String> {
        if let ByzantineMessage::PreCommit {
            view,
            sequence,
            digest,
            ..
        } = message.clone()
        {
            // 验证消息
            if !self.validate_pre_commit_message(&message) {
                return Err("无效的预提交消息".to_string());
            }

            // 收集预提交消息
            let key = format!("{}-{}", view, sequence);
            let mut pre_commit_count = 0;

            // 这里应该维护预提交消息的收集，简化实现
            pre_commit_count += 1;

            // 检查是否收集到足够的预提交消息
            if pre_commit_count >= self.quorum_size() {
                // 提交请求
                if let Some(request) = self.pending_requests.get(&key)
                    && let ByzantineMessage::Request { content, .. } = request {
                        self.committed_requests.insert(key.clone(), content.clone());
                    }

                // 创建提交消息
                let commit_message = ByzantineMessage::Commit {
                    view,
                    sequence,
                    digest,
                    sender: self.node_id.clone(),
                    timestamp: SystemTime::now(),
                };

                Ok(vec![commit_message])
            } else {
                Ok(vec![])
            }
        } else {
            Err("无效的消息类型".to_string())
        }
    }

    /// 处理提交消息
    pub fn handle_commit(&mut self, message: ByzantineMessage) -> Result<(), String> {
        if let ByzantineMessage::Commit { view, sequence, .. } = message.clone() {
            // 验证消息
            if !self.validate_commit_message(&message) {
                return Err("无效的提交消息".to_string());
            }

            // 这里应该收集提交消息并执行最终提交
            // 简化实现，直接标记为已提交
            let key = format!("{}-{}", view, sequence);
            println!("节点 {} 提交了请求 {}", self.node_id, key);

            Ok(())
        } else {
            Err("无效的消息类型".to_string())
        }
    }

    /// 检查是否是主节点
    pub fn is_primary(&self) -> bool {
        self.node_id == self.get_primary_id()
    }

    /// 获取主节点ID
    pub fn get_primary_id(&self) -> String {
        format!("node_{}", self.view % self.total_nodes as u64)
    }

    /// 验证准备消息
    fn validate_prepare_message(&self, message: &ByzantineMessage) -> bool {
        if let ByzantineMessage::Prepare {
            view,
            sequence,
            sender,
            ..
        } = message
        {
            // 检查视图号
            if *view != self.view {
                return false;
            }

            // 检查序列号
            if *sequence <= self.sequence {
                return false;
            }

            // 检查发送者状态
            if let Some(state) = self.node_states.get(sender)
                && *state == ByzantineNodeState::Byzantine {
                    return false;
                }

            true
        } else {
            false
        }
    }

    /// 验证预提交消息
    fn validate_pre_commit_message(&self, message: &ByzantineMessage) -> bool {
        if let ByzantineMessage::PreCommit {
            view,
            sequence,
            sender,
            ..
        } = message
        {
            // 检查视图号
            if *view != self.view {
                return false;
            }

            // 检查序列号
            if *sequence <= self.sequence {
                return false;
            }

            // 检查发送者状态
            if let Some(state) = self.node_states.get(sender)
                && *state == ByzantineNodeState::Byzantine {
                    return false;
                }

            true
        } else {
            false
        }
    }

    /// 验证提交消息
    fn validate_commit_message(&self, message: &ByzantineMessage) -> bool {
        if let ByzantineMessage::Commit {
            view,
            sequence,
            sender,
            ..
        } = message
        {
            // 检查视图号
            if *view != self.view {
                return false;
            }

            // 检查序列号
            if *sequence <= self.sequence {
                return false;
            }

            // 检查发送者状态
            if let Some(state) = self.node_states.get(sender)
                && *state == ByzantineNodeState::Byzantine {
                    return false;
                }

            true
        } else {
            false
        }
    }

    /// 计算消息摘要
    fn compute_digest(&self, content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 标记节点为拜占庭节点
    pub fn mark_node_byzantine(&mut self, node_id: &str) {
        self.node_states
            .insert(node_id.to_string(), ByzantineNodeState::Byzantine);
    }

    /// 标记节点为正常节点
    pub fn mark_node_honest(&mut self, node_id: &str) {
        self.node_states
            .insert(node_id.to_string(), ByzantineNodeState::Honest);
    }

    /// 获取节点状态
    pub fn get_node_state(&self, node_id: &str) -> ByzantineNodeState {
        self.node_states
            .get(node_id)
            .copied()
            .unwrap_or(ByzantineNodeState::Unknown)
    }

    /// 检查是否应该触发视图变更
    pub fn should_trigger_view_change(&self) -> bool {
        // 简化实现：如果主节点是拜占庭节点，则触发视图变更
        let primary_id = self.get_primary_id();
        self.get_node_state(&primary_id) == ByzantineNodeState::Byzantine
    }

    /// 执行视图变更
    pub fn trigger_view_change(&mut self) -> Result<Vec<ByzantineMessage>, String> {
        if !self.should_trigger_view_change() {
            return Err("不需要视图变更".to_string());
        }

        self.view += 1;

        let view_change_message = ByzantineMessage::ViewChange {
            new_view: self.view,
            sender: self.node_id.clone(),
            prepared_certificates: self.prepared_certificates.values().cloned().collect(),
            timestamp: SystemTime::now(),
        };

        Ok(vec![view_change_message])
    }
}

/// 拜占庭容错网络模拟器
#[derive(Debug, Clone)]
pub struct ByzantineNetwork {
    pub nodes: HashMap<String, PBFTNode>,
    pub message_queue: Vec<ByzantineMessage>,
    pub network_delay: Duration,
    pub message_loss_rate: f64,
}

impl ByzantineNetwork {
    /// 创建新的拜占庭网络
    pub fn new(total_nodes: usize, network_delay: Duration, message_loss_rate: f64) -> Self {
        let mut nodes = HashMap::new();

        for i in 0..total_nodes {
            let node_id = format!("node_{}", i);
            nodes.insert(node_id.clone(), PBFTNode::new(node_id, total_nodes));
        }

        Self {
            nodes,
            message_queue: Vec::new(),
            network_delay,
            message_loss_rate,
        }
    }

    /// 发送消息
    pub fn send_message(&mut self, message: ByzantineMessage) -> Result<(), String> {
        // 模拟网络延迟和消息丢失
        if self.should_drop_message() {
            return Ok(()); // 消息丢失
        }

        self.message_queue.push(message);
        Ok(())
    }

    /// 处理消息队列
    pub fn process_messages(&mut self) -> Result<(), String> {
        let messages = std::mem::take(&mut self.message_queue);

        for message in messages {
            self.route_message(message)?;
        }

        Ok(())
    }

    /// 路由消息到目标节点
    fn route_message(&mut self, message: ByzantineMessage) -> Result<(), String> {
        let target_node = match &message {
            ByzantineMessage::Request { sender, .. } => sender.clone(),
            ByzantineMessage::Prepare { sender, .. } => sender.clone(),
            ByzantineMessage::PreCommit { sender, .. } => sender.clone(),
            ByzantineMessage::Commit { sender, .. } => sender.clone(),
            ByzantineMessage::ViewChange { sender, .. } => sender.clone(),
            ByzantineMessage::NewView { sender, .. } => sender.clone(),
        };

        if let Some(node) = self.nodes.get_mut(&target_node) {
            let responses = match &message {
                ByzantineMessage::Request { .. } => node.handle_request(message)?,
                ByzantineMessage::Prepare { .. } => node.handle_prepare(message)?,
                ByzantineMessage::PreCommit { .. } => {
                    
                    node.handle_pre_commit(message)?
                }
                ByzantineMessage::Commit { .. } => {
                    node.handle_commit(message)?;
                    vec![]
                }
                _ => vec![],
            };

            // 发送响应消息
            for response in responses {
                self.send_message(response)?;
            }
        }

        Ok(())
    }

    /// 检查是否应该丢弃消息
    fn should_drop_message(&self) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        let random = (hash % 100) as f64 / 100.0;

        random < self.message_loss_rate
    }

    /// 获取网络统计信息
    pub fn get_network_stats(&self) -> ByzantineNetworkStats {
        let total_nodes = self.nodes.len();
        let byzantine_nodes = self
            .nodes
            .values()
            .flat_map(|node| node.node_states.values())
            .filter(|&&state| state == ByzantineNodeState::Byzantine)
            .count();

        ByzantineNetworkStats {
            total_nodes,
            byzantine_nodes,
            honest_nodes: total_nodes - byzantine_nodes,
            message_queue_size: self.message_queue.len(),
            network_delay: self.network_delay,
            message_loss_rate: self.message_loss_rate,
        }
    }
}

/// 拜占庭网络统计信息
#[derive(Debug, Clone)]
pub struct ByzantineNetworkStats {
    pub total_nodes: usize,
    pub byzantine_nodes: usize,
    pub honest_nodes: usize,
    pub message_queue_size: usize,
    pub network_delay: Duration,
    pub message_loss_rate: f64,
}

/// 拜占庭容错一致性级别
impl ConsistencyLevel {
    /// 检查一致性级别是否支持拜占庭容错
    pub fn supports_byzantine_fault_tolerance(&self) -> bool {
        match self {
            ConsistencyLevel::Strong | ConsistencyLevel::Linearizable => true,
            _ => false,
        }
    }
}

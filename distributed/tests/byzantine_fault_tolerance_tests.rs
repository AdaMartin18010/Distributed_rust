use c20_distributed::{
    ByzantineMessage, ByzantineNetwork, ByzantineNetworkStats, ByzantineNodeState, PBFTNode,
    PreparedCertificate, ViewChangeCertificate,
};
use std::time::Duration;

#[test]
fn test_pbft_node_creation() {
    let node = PBFTNode::new("node_0".to_string(), 4);

    assert_eq!(node.node_id, "node_0");
    assert_eq!(node.view, 0);
    assert_eq!(node.sequence, 0);
    assert_eq!(node.total_nodes, 4);
    assert_eq!(node.max_faulty_nodes, 1); // (4-1)/3 = 1
}

#[test]
fn test_pbft_byzantine_fault_tolerance_requirement() {
    let node = PBFTNode::new("node_0".to_string(), 4);

    assert!(node.is_byzantine_fault_tolerant());
    assert_eq!(node.quorum_size(), 3); // 2*1+1 = 3
}

#[test]
fn test_pbft_insufficient_nodes() {
    let node = PBFTNode::new("node_0".to_string(), 2);

    assert!(!node.is_byzantine_fault_tolerant());
}

#[test]
fn test_pbft_primary_detection() {
    let node = PBFTNode::new("node_0".to_string(), 4);

    assert_eq!(node.get_primary_id(), "node_0");
    assert!(node.is_primary());
}

#[test]
fn test_pbft_view_change() {
    let mut node = PBFTNode::new("node_1".to_string(), 4);

    // 初始视图
    assert_eq!(node.view, 0);
    assert_eq!(node.get_primary_id(), "node_0");

    // 标记主节点为拜占庭节点
    node.mark_node_byzantine("node_0");

    // 检查是否应该触发视图变更
    assert!(node.should_trigger_view_change());

    // 触发视图变更
    let messages = node.trigger_view_change().unwrap();
    assert_eq!(node.view, 1);
    assert_eq!(node.get_primary_id(), "node_1");
    assert_eq!(messages.len(), 1);
}

#[test]
fn test_pbft_node_state_management() {
    let mut node = PBFTNode::new("node_0".to_string(), 4);

    // 初始状态
    assert_eq!(node.get_node_state("node_1"), ByzantineNodeState::Unknown);

    // 标记为正常节点
    node.mark_node_honest("node_1");
    assert_eq!(node.get_node_state("node_1"), ByzantineNodeState::Honest);

    // 标记为拜占庭节点
    node.mark_node_byzantine("node_1");
    assert_eq!(node.get_node_state("node_1"), ByzantineNodeState::Byzantine);
}

#[test]
fn test_pbft_request_handling() {
    let mut node = PBFTNode::new("node_0".to_string(), 4);

    // 创建请求消息
    let request = ByzantineMessage::Request {
        id: "req_1".to_string(),
        content: b"test data".to_vec(),
        timestamp: std::time::SystemTime::now(),
        sender: "client_1".to_string(),
    };

    // 主节点处理请求
    let responses = node.handle_request(request).unwrap();
    assert_eq!(responses.len(), 1);
    assert_eq!(node.sequence, 1);

    // 非主节点处理请求应该失败
    let mut non_primary = PBFTNode::new("node_1".to_string(), 4);
    let request2 = ByzantineMessage::Request {
        id: "req_2".to_string(),
        content: b"test data".to_vec(),
        timestamp: std::time::SystemTime::now(),
        sender: "client_1".to_string(),
    };

    let result = non_primary.handle_request(request2);
    assert!(result.is_err());
}

#[test]
fn test_pbft_prepare_message_handling() {
    let mut node = PBFTNode::new("node_0".to_string(), 4);

    // 创建准备消息
    let prepare = ByzantineMessage::Prepare {
        view: 0,
        sequence: 1,
        digest: "test_digest".to_string(),
        sender: "node_1".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    // 处理准备消息
    let responses = node.handle_prepare(prepare).unwrap();
    // 由于只有一个准备消息，不满足法定人数，应该没有响应
    assert_eq!(responses.len(), 0);
}

#[test]
fn test_pbft_commit_message_handling() {
    let mut node = PBFTNode::new("node_0".to_string(), 4);

    // 创建提交消息
    let commit = ByzantineMessage::Commit {
        view: 0,
        sequence: 1,
        digest: "test_digest".to_string(),
        sender: "node_1".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    // 处理提交消息
    let result = node.handle_commit(commit);
    assert!(result.is_ok());
}

#[test]
fn test_byzantine_network_creation() {
    let network = ByzantineNetwork::new(4, Duration::from_millis(100), 0.1);

    assert_eq!(network.nodes.len(), 4);
    assert_eq!(network.network_delay, Duration::from_millis(100));
    assert_eq!(network.message_loss_rate, 0.1);
}

#[test]
fn test_byzantine_network_message_sending() {
    let mut network = ByzantineNetwork::new(4, Duration::from_millis(100), 0.0); // 无消息丢失

    let message = ByzantineMessage::Request {
        id: "req_1".to_string(),
        content: b"test data".to_vec(),
        timestamp: std::time::SystemTime::now(),
        sender: "client_1".to_string(),
    };

    let result = network.send_message(message);
    assert!(result.is_ok());
    assert_eq!(network.message_queue.len(), 1);
}

#[test]
fn test_byzantine_network_stats() {
    let network = ByzantineNetwork::new(4, Duration::from_millis(100), 0.1);
    let stats = network.get_network_stats();

    assert_eq!(stats.total_nodes, 4);
    assert_eq!(stats.byzantine_nodes, 0);
    assert_eq!(stats.honest_nodes, 4);
    assert_eq!(stats.message_queue_size, 0);
    assert_eq!(stats.network_delay, Duration::from_millis(100));
    assert_eq!(stats.message_loss_rate, 0.1);
}

#[test]
fn test_byzantine_message_creation() {
    let request = ByzantineMessage::Request {
        id: "req_1".to_string(),
        content: b"test data".to_vec(),
        timestamp: std::time::SystemTime::now(),
        sender: "client_1".to_string(),
    };

    match request {
        ByzantineMessage::Request {
            id,
            content,
            sender,
            ..
        } => {
            assert_eq!(id, "req_1");
            assert_eq!(content, b"test data");
            assert_eq!(sender, "client_1");
        }
        _ => panic!("Expected Request message"),
    }
}

#[test]
fn test_byzantine_message_prepare() {
    let prepare = ByzantineMessage::Prepare {
        view: 1,
        sequence: 2,
        digest: "test_digest".to_string(),
        sender: "node_1".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    match prepare {
        ByzantineMessage::Prepare {
            view,
            sequence,
            digest,
            sender,
            ..
        } => {
            assert_eq!(view, 1);
            assert_eq!(sequence, 2);
            assert_eq!(digest, "test_digest");
            assert_eq!(sender, "node_1");
        }
        _ => panic!("Expected Prepare message"),
    }
}

#[test]
fn test_byzantine_message_pre_commit() {
    let pre_commit = ByzantineMessage::PreCommit {
        view: 1,
        sequence: 2,
        digest: "test_digest".to_string(),
        sender: "node_1".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    match pre_commit {
        ByzantineMessage::PreCommit {
            view,
            sequence,
            digest,
            sender,
            ..
        } => {
            assert_eq!(view, 1);
            assert_eq!(sequence, 2);
            assert_eq!(digest, "test_digest");
            assert_eq!(sender, "node_1");
        }
        _ => panic!("Expected PreCommit message"),
    }
}

#[test]
fn test_byzantine_message_commit() {
    let commit = ByzantineMessage::Commit {
        view: 1,
        sequence: 2,
        digest: "test_digest".to_string(),
        sender: "node_1".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    match commit {
        ByzantineMessage::Commit {
            view,
            sequence,
            digest,
            sender,
            ..
        } => {
            assert_eq!(view, 1);
            assert_eq!(sequence, 2);
            assert_eq!(digest, "test_digest");
            assert_eq!(sender, "node_1");
        }
        _ => panic!("Expected Commit message"),
    }
}

#[test]
fn test_byzantine_message_view_change() {
    let view_change = ByzantineMessage::ViewChange {
        new_view: 2,
        sender: "node_1".to_string(),
        prepared_certificates: vec![],
        timestamp: std::time::SystemTime::now(),
    };

    match view_change {
        ByzantineMessage::ViewChange {
            new_view,
            sender,
            prepared_certificates,
            ..
        } => {
            assert_eq!(new_view, 2);
            assert_eq!(sender, "node_1");
            assert_eq!(prepared_certificates.len(), 0);
        }
        _ => panic!("Expected ViewChange message"),
    }
}

#[test]
fn test_byzantine_message_new_view() {
    let new_view = ByzantineMessage::NewView {
        new_view: 2,
        sender: "node_1".to_string(),
        view_change_certificates: vec![],
        timestamp: std::time::SystemTime::now(),
    };

    match new_view {
        ByzantineMessage::NewView {
            new_view,
            sender,
            view_change_certificates,
            ..
        } => {
            assert_eq!(new_view, 2);
            assert_eq!(sender, "node_1");
            assert_eq!(view_change_certificates.len(), 0);
        }
        _ => panic!("Expected NewView message"),
    }
}

#[test]
fn test_byzantine_node_state() {
    assert_eq!(ByzantineNodeState::Honest, ByzantineNodeState::Honest);
    assert_eq!(ByzantineNodeState::Byzantine, ByzantineNodeState::Byzantine);
    assert_eq!(ByzantineNodeState::Unknown, ByzantineNodeState::Unknown);

    assert_ne!(ByzantineNodeState::Honest, ByzantineNodeState::Byzantine);
    assert_ne!(ByzantineNodeState::Honest, ByzantineNodeState::Unknown);
    assert_ne!(ByzantineNodeState::Byzantine, ByzantineNodeState::Unknown);
}

#[test]
fn test_prepared_certificate() {
    let certificate = PreparedCertificate {
        view: 1,
        sequence: 2,
        digest: "test_digest".to_string(),
        prepare_messages: vec![],
    };

    assert_eq!(certificate.view, 1);
    assert_eq!(certificate.sequence, 2);
    assert_eq!(certificate.digest, "test_digest");
    assert_eq!(certificate.prepare_messages.len(), 0);
}

#[test]
fn test_view_change_certificate() {
    let certificate = ViewChangeCertificate {
        new_view: 2,
        view_change_messages: vec![],
    };

    assert_eq!(certificate.new_view, 2);
    assert_eq!(certificate.view_change_messages.len(), 0);
}

#[test]
fn test_byzantine_network_stats_creation() {
    let stats = ByzantineNetworkStats {
        total_nodes: 4,
        byzantine_nodes: 1,
        honest_nodes: 3,
        message_queue_size: 5,
        network_delay: Duration::from_millis(100),
        message_loss_rate: 0.1,
    };

    assert_eq!(stats.total_nodes, 4);
    assert_eq!(stats.byzantine_nodes, 1);
    assert_eq!(stats.honest_nodes, 3);
    assert_eq!(stats.message_queue_size, 5);
    assert_eq!(stats.network_delay, Duration::from_millis(100));
    assert_eq!(stats.message_loss_rate, 0.1);
}

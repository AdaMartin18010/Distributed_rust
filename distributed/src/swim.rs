use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwimMemberState {
    Alive,
    Suspect,
    Faulty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwimEvent {
    pub node_id: String,
    pub state: SwimMemberState,
    pub timestamp: SystemTime,
    pub incarnation: u64,
}

impl SwimEvent {
    pub fn new(node_id: String, state: SwimMemberState, incarnation: u64) -> Self {
        Self {
            node_id,
            state,
            timestamp: SystemTime::now(),
            incarnation,
        }
    }
}

pub trait SwimTransport {
    fn ping(&self, to: &str) -> bool;
    fn ping_req(&self, relay: &str, target: &str) -> bool {
        let _ = relay;
        self.ping(target)
    }
    fn ack(&self, from: &str) -> bool {
        self.ping(from)
    }
    fn gossip(&self, to: &str, events: &[SwimEvent]) -> bool;
}

/// 增强的SWIM传输层，支持超时和重试
#[allow(dead_code)]
pub struct EnhancedSwimTransport {
    timeout: Duration,
    max_retries: usize,
    network_delay: Duration,
}

impl EnhancedSwimTransport {
    pub fn new(timeout: Duration, max_retries: usize, network_delay: Duration) -> Self {
        Self {
            timeout,
            max_retries,
            network_delay,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_network_delay(mut self, delay: Duration) -> Self {
        self.network_delay = delay;
        self
    }

    fn simulate_network_call(&self, success_rate: f64) -> bool {
        // 模拟网络调用的成功率和延迟
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        let random = (hash % 100) as f64 / 100.0;

        random < success_rate
    }
}

impl SwimTransport for EnhancedSwimTransport {
    fn ping(&self, _to: &str) -> bool {
        // 模拟网络延迟
        std::thread::sleep(self.network_delay);
        self.simulate_network_call(0.9) // 90%成功率
    }

    fn ping_req(&self, _relay: &str, _target: &str) -> bool {
        // 间接ping需要两个网络调用
        std::thread::sleep(self.network_delay * 2);
        self.simulate_network_call(0.8) // 80%成功率
    }

    fn ack(&self, _from: &str) -> bool {
        std::thread::sleep(self.network_delay);
        self.simulate_network_call(0.95) // 95%成功率
    }

    fn gossip(&self, _to: &str, _events: &[SwimEvent]) -> bool {
        std::thread::sleep(self.network_delay);
        self.simulate_network_call(0.85) // 85%成功率
    }
}

/// 增强的SWIM节点，支持完整的故障检测协议
pub struct SwimNode<T: SwimTransport> {
    pub node_id: String,
    pub transport: T,
    pub incarnation: Arc<AtomicU64>,
    pub probe_interval: Duration,
    pub suspect_timeout: Duration,
    pub fanout: usize,
    pub indirect_probes: usize,
    pub last_probe_time: HashMap<String, Instant>,
    pub suspect_timers: HashMap<String, Instant>,
}

impl<T: SwimTransport> SwimNode<T> {
    pub fn new(node_id: String, transport: T) -> Self {
        Self {
            node_id,
            transport,
            incarnation: Arc::new(AtomicU64::new(0)),
            probe_interval: Duration::from_millis(1000),
            suspect_timeout: Duration::from_millis(5000),
            fanout: 3,
            indirect_probes: 3,
            last_probe_time: HashMap::new(),
            suspect_timers: HashMap::new(),
        }
    }

    pub fn with_params(
        mut self,
        probe_interval: Duration,
        suspect_timeout: Duration,
        fanout: usize,
    ) -> Self {
        self.probe_interval = probe_interval;
        self.suspect_timeout = suspect_timeout;
        self.fanout = fanout;
        self
    }

    pub fn increment_incarnation(&self) -> u64 {
        self.incarnation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn get_incarnation(&self) -> u64 {
        self.incarnation.load(Ordering::SeqCst)
    }
}

impl<T: SwimTransport> SwimNode<T> {
    /// 直接探测目标节点
    pub fn probe(&self, peer: &str) -> SwimEvent {
        let ok = self.transport.ping(peer);
        let incarnation = self.get_incarnation();
        SwimEvent::new(
            peer.to_string(),
            if ok {
                SwimMemberState::Alive
            } else {
                SwimMemberState::Suspect
            },
            incarnation,
        )
    }

    /// 间接探测目标节点
    pub fn probe_indirect<'a>(
        &self,
        target: &str,
        relays: impl IntoIterator<Item = &'a str>,
    ) -> SwimEvent {
        if self.transport.ping(target) {
            return SwimEvent::new(
                target.to_string(),
                SwimMemberState::Alive,
                self.get_incarnation(),
            );
        }

        for r in relays {
            if self.transport.ping_req(r, target) {
                return SwimEvent::new(
                    target.to_string(),
                    SwimMemberState::Alive,
                    self.get_incarnation(),
                );
            }
        }

        SwimEvent::new(
            target.to_string(),
            SwimMemberState::Suspect,
            self.get_incarnation(),
        )
    }

    /// 执行完整的SWIM探测协议
    pub fn swim_probe(&mut self, target: &str, peers: &[String]) -> SwimEvent {
        // 记录探测时间
        self.last_probe_time
            .insert(target.to_string(), Instant::now());

        // 直接探测
        let direct_result = self.probe(target);
        if direct_result.state == SwimMemberState::Alive {
            return direct_result;
        }

        // 间接探测
        let mut available_peers: Vec<&String> = peers
            .iter()
            .filter(|p| **p != target && **p != self.node_id)
            .collect();

        // 随机选择一些节点进行间接探测
        if available_peers.len() > self.indirect_probes {
            available_peers.truncate(self.indirect_probes);
        }

        self.probe_indirect(target, available_peers.iter().map(|s| s.as_str()))
    }

    /// 检查是否需要将可疑节点标记为故障
    pub fn check_suspect_timeouts(&mut self) -> Vec<SwimEvent> {
        let mut events = Vec::new();
        let now = Instant::now();

        let expired_suspects: Vec<String> = self
            .suspect_timers
            .iter()
            .filter(|(_, timer)| now.duration_since(**timer) > self.suspect_timeout)
            .map(|(node_id, _)| node_id.clone())
            .collect();

        for node_id in expired_suspects {
            self.suspect_timers.remove(&node_id);
            events.push(SwimEvent::new(
                node_id.clone(),
                SwimMemberState::Faulty,
                self.get_incarnation(),
            ));
        }

        events
    }

    /// 处理接收到的SWIM事件
    pub fn handle_swim_event(&mut self, event: &SwimEvent) -> bool {
        match event.state {
            SwimMemberState::Alive => {
                // 清除可疑状态
                self.suspect_timers.remove(&event.node_id);
                true
            }
            SwimMemberState::Suspect => {
                // 记录可疑时间
                self.suspect_timers
                    .insert(event.node_id.clone(), Instant::now());
                true
            }
            SwimMemberState::Faulty => {
                // 移除故障节点
                self.suspect_timers.remove(&event.node_id);
                self.last_probe_time.remove(&event.node_id);
                true
            }
        }
    }

    /// 生成gossip消息
    pub fn generate_gossip(&self, view: &MembershipView) -> Vec<SwimEvent> {
        let mut events = Vec::new();
        let incarnation = self.get_incarnation();

        for (node_id, info) in &view.members {
            if node_id != &self.node_id {
                events.push(SwimEvent::new(node_id.clone(), info.state, incarnation));
            }
        }

        events
    }

    /// 广播gossip消息
    pub fn broadcast_gossip(&self, events: &[SwimEvent], peers: &[String]) -> usize {
        let mut success_count = 0;

        for peer in peers {
            if peer != &self.node_id && self.transport.gossip(peer, events) {
                success_count += 1;
            }
        }

        success_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Version(pub u64);

#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub state: SwimMemberState,
    pub version: Version,
    pub incarnation: u64,
    pub last_seen: SystemTime,
}

#[derive(Default, Debug, Clone)]
pub struct MembershipView {
    pub me: String,
    pub members: HashMap<String, MemberInfo>,
    pub version: Version,
}

impl MembershipView {
    pub fn new(me: String) -> Self {
        Self {
            me,
            members: HashMap::new(),
            version: Version(0),
        }
    }

    pub fn local_update(&mut self, node: &str, state: SwimMemberState, incarnation: u64) {
        let ent = self.members.entry(node.to_string()).or_insert(MemberInfo {
            state,
            version: Version(0),
            incarnation: 0,
            last_seen: SystemTime::now(),
        });
        ent.version.0 += 1;
        ent.state = state;
        ent.incarnation = incarnation;
        ent.last_seen = SystemTime::now();
        self.version.0 += 1;
    }

    pub fn update_from_event(&mut self, event: &SwimEvent) -> bool {
        let ent = self
            .members
            .entry(event.node_id.clone())
            .or_insert(MemberInfo {
                state: event.state,
                version: Version(0),
                incarnation: 0,
                last_seen: SystemTime::now(),
            });

        // 检查incarnation号，只有更新的才接受
        if event.incarnation >= ent.incarnation {
            ent.state = event.state;
            ent.incarnation = event.incarnation;
            ent.last_seen = event.timestamp;
            ent.version.0 += 1;
            self.version.0 += 1;
            true
        } else {
            false
        }
    }

    pub fn gossip_payload(&self) -> Vec<(String, MemberInfo)> {
        self.members
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn merge_from(&mut self, incoming: &[(String, MemberInfo)]) {
        for (node, info) in incoming {
            let ent = self.members.entry(node.clone()).or_insert(MemberInfo {
                state: info.state,
                version: info.version,
                incarnation: info.incarnation,
                last_seen: info.last_seen,
            });

            // 使用incarnation号来决定是否更新
            if info.incarnation > ent.incarnation
                || (info.incarnation == ent.incarnation && info.version.0 > ent.version.0)
            {
                *ent = info.clone();
                self.version.0 += 1;
            }
        }
    }

    /// 获取活跃节点列表
    pub fn alive_members(&self) -> Vec<String> {
        self.members
            .iter()
            .filter(|(_, info)| info.state == SwimMemberState::Alive)
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    /// 获取可疑节点列表
    pub fn suspect_members(&self) -> Vec<String> {
        self.members
            .iter()
            .filter(|(_, info)| info.state == SwimMemberState::Suspect)
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    /// 获取故障节点列表
    pub fn faulty_members(&self) -> Vec<String> {
        self.members
            .iter()
            .filter(|(_, info)| info.state == SwimMemberState::Faulty)
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    /// 清理过期的故障节点
    pub fn cleanup_faulty_members(&mut self, max_age: Duration) {
        let now = SystemTime::now();
        self.members.retain(|_, info| {
            if info.state == SwimMemberState::Faulty {
                now.duration_since(info.last_seen)
                    .unwrap_or(Duration::from_secs(0))
                    < max_age
            } else {
                true
            }
        });
    }

    /// 检查节点是否在集群中
    pub fn contains(&self, node_id: &str) -> bool {
        self.members.contains_key(node_id)
    }

    /// 获取节点信息
    pub fn get_member(&self, node_id: &str) -> Option<&MemberInfo> {
        self.members.get(node_id)
    }

    /// 获取集群大小
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// 获取活跃节点数量
    pub fn alive_count(&self) -> usize {
        self.members
            .values()
            .filter(|info| info.state == SwimMemberState::Alive)
            .count()
    }
}

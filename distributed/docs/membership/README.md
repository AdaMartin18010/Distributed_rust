# 成员管理

> 分布式系统中的成员检测、管理和服务发现机制

## 目录

- [成员管理](#成员管理)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [成员状态](#成员状态)
    - [成员视图](#成员视图)
  - [🔍 故障检测机制](#-故障检测机制)
    - [基于心跳的故障检测](#基于心跳的故障检测)
    - [SWIM 协议实现](#swim-协议实现)
  - [🔍 服务发现](#-服务发现)
    - [服务注册中心](#服务注册中心)
    - [健康检查器](#健康检查器)
  - [🔄 成员变更处理](#-成员变更处理)
    - [成员变更事件](#成员变更事件)
    - [成员变更处理器](#成员变更处理器)
  - [🧪 测试策略](#-测试策略)
    - [成员管理测试](#成员管理测试)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

成员管理是分布式系统的基础组件，负责维护集群中节点的成员信息，检测节点故障，并提供服务发现功能。

## 🎯 核心概念

### 成员状态

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MemberStatus {
    Joining,    // 正在加入
    Active,     // 活跃状态
    Leaving,    // 正在离开
    Failed,     // 故障状态
    Suspicious, // 可疑状态
}

#[derive(Debug, Clone)]
pub struct Member {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub status: MemberStatus,
    pub incarnation: u64,
    pub last_seen: Instant,
    pub metadata: HashMap<String, String>,
}

impl Member {
    pub fn new(id: String, address: String, port: u16) -> Self {
        Self {
            id,
            address,
            port,
            status: MemberStatus::Joining,
            incarnation: 0,
            last_seen: Instant::now(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn is_alive(&self) -> bool {
        matches!(self.status, MemberStatus::Active)
    }
    
    pub fn is_failed(&self) -> bool {
        matches!(self.status, MemberStatus::Failed)
    }
    
    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }
    
    pub fn increment_incarnation(&mut self) {
        self.incarnation += 1;
    }
}
```

### 成员视图

```rust
pub struct MembershipView {
    members: HashMap<String, Member>,
    local_member: String,
    version: u64,
    last_updated: Instant,
}

impl MembershipView {
    pub fn new(local_member_id: String) -> Self {
        Self {
            members: HashMap::new(),
            local_member: local_member_id,
            version: 0,
            last_updated: Instant::now(),
        }
    }
    
    pub fn add_member(&mut self, member: Member) {
        self.members.insert(member.id.clone(), member);
        self.version += 1;
        self.last_updated = Instant::now();
    }
    
    pub fn remove_member(&mut self, member_id: &str) -> Option<Member> {
        let removed = self.members.remove(member_id);
        if removed.is_some() {
            self.version += 1;
            self.last_updated = Instant::now();
        }
        removed
    }
    
    pub fn update_member(&mut self, member: Member) -> bool {
        if let Some(existing) = self.members.get_mut(&member.id) {
            // 检查版本号，只接受更新的信息
            if member.incarnation > existing.incarnation {
                *existing = member;
                self.version += 1;
                self.last_updated = Instant::now();
                return true;
            }
        }
        false
    }
    
    pub fn get_member(&self, member_id: &str) -> Option<&Member> {
        self.members.get(member_id)
    }
    
    pub fn get_alive_members(&self) -> Vec<&Member> {
        self.members.values()
            .filter(|m| m.is_alive())
            .collect()
    }
    
    pub fn get_failed_members(&self) -> Vec<&Member> {
        self.members.values()
            .filter(|m| m.is_failed())
            .collect()
    }
    
    pub fn merge_view(&mut self, other: &MembershipView) -> bool {
        let mut updated = false;
        
        for (id, member) in &other.members {
            if let Some(existing) = self.members.get(id) {
                if member.incarnation > existing.incarnation {
                    self.members.insert(id.clone(), member.clone());
                    updated = true;
                }
            } else {
                self.members.insert(id.clone(), member.clone());
                updated = true;
            }
        }
        
        if updated {
            self.version += 1;
            self.last_updated = Instant::now();
        }
        
        updated
    }
}
```

## 🔍 故障检测机制

### 基于心跳的故障检测

```rust
pub struct HeartbeatFailureDetector {
    members: HashMap<String, MemberHeartbeatInfo>,
    heartbeat_interval: Duration,
    timeout: Duration,
    detector_task: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct MemberHeartbeatInfo {
    pub member: Member,
    pub last_heartbeat: Option<Instant>,
    pub missed_heartbeats: u32,
    pub status: HeartbeatStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeartbeatStatus {
    Healthy,
    Suspect,
    Failed,
}

impl HeartbeatFailureDetector {
    pub fn new(heartbeat_interval: Duration, timeout: Duration) -> Self {
        Self {
            members: HashMap::new(),
            heartbeat_interval,
            timeout,
            detector_task: None,
        }
    }
    
    pub fn add_member(&mut self, member: Member) {
        self.members.insert(member.id.clone(), MemberHeartbeatInfo {
            member,
            last_heartbeat: None,
            missed_heartbeats: 0,
            status: HeartbeatStatus::Healthy,
        });
    }
    
    pub async fn start_detection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut members = self.members.clone();
        let heartbeat_interval = self.heartbeat_interval;
        let timeout = self.timeout;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                // 发送心跳
                for (member_id, info) in &mut members {
                    if let Err(_) = Self::send_heartbeat(member_id).await {
                        info.missed_heartbeats += 1;
                    } else {
                        info.last_heartbeat = Some(Instant::now());
                        info.missed_heartbeats = 0;
                        info.status = HeartbeatStatus::Healthy;
                    }
                }
                
                // 检查超时
                let now = Instant::now();
                for (_, info) in &mut members {
                    if let Some(last_heartbeat) = info.last_heartbeat {
                        let elapsed = now.duration_since(last_heartbeat);
                        
                        if elapsed > timeout {
                            info.status = HeartbeatStatus::Failed;
                        } else if elapsed > timeout / 2 {
                            info.status = HeartbeatStatus::Suspect;
                        }
                    }
                }
            }
        });
        
        self.detector_task = Some(handle);
        Ok(())
    }
    
    async fn send_heartbeat(member_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 实现心跳发送逻辑
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
    
    pub fn get_member_status(&self, member_id: &str) -> Option<HeartbeatStatus> {
        self.members.get(member_id).map(|info| info.status.clone())
    }
    
    pub fn get_failed_members(&self) -> Vec<String> {
        self.members.iter()
            .filter(|(_, info)| info.status == HeartbeatStatus::Failed)
            .map(|(id, _)| id.clone())
            .collect()
    }
}
```

### SWIM 协议实现

```rust
pub struct SWIMMembership {
    local_member: Member,
    membership_view: MembershipView,
    protocol_period: Duration,
    ping_timeout: Duration,
    indirect_ping_count: usize,
    suspicion_timeout: Duration,
    protocol_task: Option<tokio::task::JoinHandle<()>>,
}

impl SWIMMembership {
    pub fn new(
        local_member: Member,
        protocol_period: Duration,
        ping_timeout: Duration,
        indirect_ping_count: usize,
        suspicion_timeout: Duration,
    ) -> Self {
        let mut membership_view = MembershipView::new(local_member.id.clone());
        membership_view.add_member(local_member.clone());
        
        Self {
            local_member,
            membership_view,
            protocol_period,
            ping_timeout,
            indirect_ping_count,
            suspicion_timeout,
            protocol_task: None,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut membership_view = self.membership_view.clone();
        let protocol_period = self.protocol_period;
        let ping_timeout = self.ping_timeout;
        let indirect_ping_count = self.indirect_ping_count;
        let suspicion_timeout = self.suspicion_timeout;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(protocol_period);
            
            loop {
                interval.tick().await;
                
                // 选择随机成员进行探测
                if let Some(target_member) = Self::select_random_member(&membership_view) {
                    Self::probe_member(
                        &mut membership_view,
                        &target_member,
                        ping_timeout,
                        indirect_ping_count,
                        suspicion_timeout,
                    ).await;
                }
            }
        });
        
        self.protocol_task = Some(handle);
        Ok(())
    }
    
    async fn probe_member(
        membership_view: &mut MembershipView,
        target_member: &Member,
        ping_timeout: Duration,
        indirect_ping_count: usize,
        suspicion_timeout: Duration,
    ) {
        // 直接探测
        match Self::direct_probe(target_member, ping_timeout).await {
            Ok(_) => {
                // 探测成功，更新成员状态
                if let Some(member) = membership_view.get_member(&target_member.id) {
                    let mut updated_member = member.clone();
                    updated_member.status = MemberStatus::Active;
                    updated_member.update_last_seen();
                    membership_view.update_member(updated_member);
                }
                return;
            }
            Err(_) => {
                // 直接探测失败，尝试间接探测
            }
        }
        
        // 间接探测
        let indirect_success = Self::indirect_probe(
            membership_view,
            target_member,
            indirect_ping_count,
            ping_timeout,
        ).await;
        
        if indirect_success {
            // 间接探测成功
            if let Some(member) = membership_view.get_member(&target_member.id) {
                let mut updated_member = member.clone();
                updated_member.status = MemberStatus::Active;
                updated_member.update_last_seen();
                membership_view.update_member(updated_member);
            }
        } else {
            // 间接探测也失败，标记为可疑
            if let Some(member) = membership_view.get_member(&target_member.id) {
                let mut updated_member = member.clone();
                updated_member.status = MemberStatus::Suspicious;
                updated_member.increment_incarnation();
                membership_view.update_member(updated_member);
            }
        }
    }
    
    async fn direct_probe(member: &Member, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
        // 实现直接探测逻辑
        tokio::time::timeout(timeout, async {
            // 发送 ping 消息
            Self::send_ping(member).await?;
            
            // 等待 pong 响应
            Self::wait_for_pong(member).await?;
            
            Ok(())
        }).await??;
        
        Ok(())
    }
    
    async fn indirect_probe(
        membership_view: &MembershipView,
        target_member: &Member,
        count: usize,
        timeout: Duration,
    ) -> bool {
        let mut success_count = 0;
        let mut tasks = Vec::new();
        
        // 选择中间成员进行间接探测
        let intermediate_members = Self::select_intermediate_members(membership_view, target_member, count);
        
        for intermediate_member in intermediate_members {
            let target = target_member.clone();
            let task = tokio::spawn(async move {
                Self::indirect_ping_via_member(&intermediate_member, &target, timeout).await
            });
            tasks.push(task);
        }
        
        // 等待所有间接探测完成
        for task in tasks {
            if let Ok(Ok(true)) = task.await {
                success_count += 1;
            }
        }
        
        success_count > 0
    }
    
    fn select_random_member(membership_view: &MembershipView) -> Option<Member> {
        let alive_members: Vec<Member> = membership_view.get_alive_members()
            .into_iter()
            .cloned()
            .collect();
        
        if alive_members.is_empty() {
            None
        } else {
            let index = rand::random::<usize>() % alive_members.len();
            Some(alive_members[index].clone())
        }
    }
    
    fn select_intermediate_members(
        membership_view: &MembershipView,
        target_member: &Member,
        count: usize,
    ) -> Vec<Member> {
        membership_view.get_alive_members()
            .into_iter()
            .filter(|m| m.id != target_member.id)
            .take(count)
            .cloned()
            .collect()
    }
    
    async fn send_ping(member: &Member) -> Result<(), Box<dyn std::error::Error>> {
        // 实现 ping 发送逻辑
        Ok(())
    }
    
    async fn wait_for_pong(member: &Member) -> Result<(), Box<dyn std::error::Error>> {
        // 实现 pong 等待逻辑
        Ok(())
    }
    
    async fn indirect_ping_via_member(
        intermediate_member: &Member,
        target_member: &Member,
        timeout: Duration,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // 实现通过中间成员的间接探测
        Ok(true)
    }
}
```

## 🔍 服务发现

### 服务注册中心

```rust
pub struct ServiceRegistry {
    services: HashMap<String, ServiceInfo>,
    health_checker: HealthChecker,
    registry_task: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub service_id: String,
    pub service_name: String,
    pub address: String,
    pub port: u16,
    pub status: ServiceStatus,
    pub metadata: HashMap<String, String>,
    pub last_heartbeat: Instant,
    pub ttl: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            health_checker: HealthChecker::new(),
            registry_task: None,
        }
    }
    
    pub async fn register_service(&mut self, service_info: ServiceInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.services.insert(service_info.service_id.clone(), service_info);
        Ok(())
    }
    
    pub async fn deregister_service(&mut self, service_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.services.remove(service_id);
        Ok(())
    }
    
    pub async fn discover_services(&self, service_name: &str) -> Vec<&ServiceInfo> {
        self.services.values()
            .filter(|service| {
                service.service_name == service_name && service.status == ServiceStatus::Healthy
            })
            .collect()
    }
    
    pub async fn get_service(&self, service_id: &str) -> Option<&ServiceInfo> {
        self.services.get(service_id)
    }
    
    pub async fn update_heartbeat(&mut self, service_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(service) = self.services.get_mut(service_id) {
            service.last_heartbeat = Instant::now();
            service.status = ServiceStatus::Healthy;
        }
        Ok(())
    }
    
    pub async fn start_health_checking(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut services = self.services.clone();
        let health_checker = self.health_checker.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // 检查服务健康状态
                for (service_id, service_info) in &mut services {
                    let is_healthy = health_checker.check_health(service_info).await;
                    
                    if is_healthy {
                        service_info.status = ServiceStatus::Healthy;
                        service_info.last_heartbeat = Instant::now();
                    } else {
                        service_info.status = ServiceStatus::Unhealthy;
                    }
                }
                
                // 清理过期的服务
                let now = Instant::now();
                services.retain(|_, service| {
                    now.duration_since(service.last_heartbeat) < service.ttl
                });
            }
        });
        
        self.registry_task = Some(handle);
        Ok(())
    }
}
```

### 健康检查器

```rust
#[derive(Debug, Clone)]
pub struct HealthChecker {
    timeout: Duration,
    retry_count: usize,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            retry_count: 3,
        }
    }
    
    pub async fn check_health(&self, service_info: &ServiceInfo) -> bool {
        for _ in 0..self.retry_count {
            if let Ok(true) = self.perform_health_check(service_info).await {
                return true;
            }
        }
        
        false
    }
    
    async fn perform_health_check(&self, service_info: &ServiceInfo) -> Result<bool, Box<dyn std::error::Error>> {
        let url = format!("http://{}:{}/health", service_info.address, service_info.port);
        
        let client = reqwest::Client::new();
        let response = tokio::time::timeout(
            self.timeout,
            client.get(&url).send()
        ).await??;
        
        Ok(response.status().is_success())
    }
}
```

## 🔄 成员变更处理

### 成员变更事件

```rust
#[derive(Debug, Clone)]
pub enum MembershipEvent {
    MemberJoined(Member),
    MemberLeft(Member),
    MemberFailed(Member),
    MemberRecovered(Member),
    MembershipChanged(MembershipView),
}

pub struct MembershipEventBus {
    subscribers: Vec<mpsc::UnboundedSender<MembershipEvent>>,
}

impl MembershipEventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }
    
    pub fn subscribe(&mut self) -> mpsc::UnboundedReceiver<MembershipEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.push(tx);
        rx
    }
    
    pub async fn publish(&self, event: MembershipEvent) -> Result<(), Box<dyn std::error::Error>> {
        for subscriber in &self.subscribers {
            let _ = subscriber.send(event.clone());
        }
        Ok(())
    }
}
```

### 成员变更处理器

```rust
pub struct MembershipChangeHandler {
    event_bus: MembershipEventBus,
    membership_view: MembershipView,
    change_handlers: Vec<Box<dyn MembershipChangeHandler>>,
}

pub trait MembershipChangeHandler {
    async fn handle_member_joined(&mut self, member: &Member) -> Result<(), Box<dyn std::error::Error>>;
    async fn handle_member_left(&mut self, member: &Member) -> Result<(), Box<dyn std::error::Error>>;
    async fn handle_member_failed(&mut self, member: &Member) -> Result<(), Box<dyn std::error::Error>>;
    async fn handle_member_recovered(&mut self, member: &Member) -> Result<(), Box<dyn std::error::Error>>;
}

impl MembershipChangeHandler {
    pub fn new(event_bus: MembershipEventBus, membership_view: MembershipView) -> Self {
        Self {
            event_bus,
            membership_view,
            change_handlers: Vec::new(),
        }
    }
    
    pub fn add_handler(&mut self, handler: Box<dyn MembershipChangeHandler>) {
        self.change_handlers.push(handler);
    }
    
    pub async fn start_handling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut event_receiver = self.event_bus.subscribe();
        
        while let Some(event) = event_receiver.recv().await {
            match event {
                MembershipEvent::MemberJoined(member) => {
                    self.membership_view.add_member(member.clone());
                    for handler in &mut self.change_handlers {
                        let _ = handler.handle_member_joined(&member).await;
                    }
                }
                MembershipEvent::MemberLeft(member) => {
                    self.membership_view.remove_member(&member.id);
                    for handler in &mut self.change_handlers {
                        let _ = handler.handle_member_left(&member).await;
                    }
                }
                MembershipEvent::MemberFailed(member) => {
                    if let Some(existing) = self.membership_view.get_member(&member.id) {
                        let mut updated = existing.clone();
                        updated.status = MemberStatus::Failed;
                        self.membership_view.update_member(updated);
                    }
                    for handler in &mut self.change_handlers {
                        let _ = handler.handle_member_failed(&member).await;
                    }
                }
                MembershipEvent::MemberRecovered(member) => {
                    if let Some(existing) = self.membership_view.get_member(&member.id) {
                        let mut updated = existing.clone();
                        updated.status = MemberStatus::Active;
                        updated.update_last_seen();
                        self.membership_view.update_member(updated);
                    }
                    for handler in &mut self.change_handlers {
                        let _ = handler.handle_member_recovered(&member).await;
                    }
                }
                MembershipEvent::MembershipChanged(view) => {
                    self.membership_view.merge_view(&view);
                }
            }
        }
        
        Ok(())
    }
}
```

## 🧪 测试策略

### 成员管理测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_membership_view() {
        let mut view = MembershipView::new("local".to_string());
        
        let member1 = Member::new("node1".to_string(), "127.0.0.1".to_string(), 8080);
        let member2 = Member::new("node2".to_string(), "127.0.0.1".to_string(), 8081);
        
        view.add_member(member1);
        view.add_member(member2);
        
        assert_eq!(view.members.len(), 2);
        assert_eq!(view.version, 2);
        
        let alive_members = view.get_alive_members();
        assert_eq!(alive_members.len(), 2);
    }
    
    #[tokio::test]
    async fn test_swim_membership() {
        let local_member = Member::new("local".to_string(), "127.0.0.1".to_string(), 8080);
        let mut swim = SWIMMembership::new(
            local_member,
            Duration::from_millis(100),
            Duration::from_millis(50),
            3,
            Duration::from_secs(1),
        );
        
        // 添加远程成员
        let remote_member = Member::new("remote".to_string(), "127.0.0.1".to_string(), 8081);
        swim.membership_view.add_member(remote_member);
        
        // 启动 SWIM 协议
        swim.start().await.unwrap();
        
        // 等待一段时间
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // 检查成员状态
        let remote_member = swim.membership_view.get_member("remote");
        assert!(remote_member.is_some());
    }
    
    #[tokio::test]
    async fn test_service_registry() {
        let mut registry = ServiceRegistry::new();
        
        let service_info = ServiceInfo {
            service_id: "service1".to_string(),
            service_name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            status: ServiceStatus::Healthy,
            metadata: HashMap::new(),
            last_heartbeat: Instant::now(),
            ttl: Duration::from_secs(60),
        };
        
        registry.register_service(service_info).await.unwrap();
        
        let services = registry.discover_services("test-service").await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].service_id, "service1");
    }
}
```

## 📚 进一步阅读

- [故障处理](../failure/README.md) - 故障检测和处理
- [共识机制](../consensus/README.md) - 共识算法的成员管理
- [一致性模型](../consistency/README.md) - 一致性保证
- [性能优化](../performance/OPTIMIZATION.md) - 成员管理性能优化

## 🔗 相关文档

- [故障处理](../failure/README.md)
- [共识机制](../consensus/README.md)
- [一致性模型](../consistency/README.md)
- [性能优化](../performance/OPTIMIZATION.md)
- [测试策略](../testing/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

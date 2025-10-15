# 故障处理

> 分布式系统中的故障模型、检测机制和容错策略

## 目录

- [故障处理](#故障处理)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 故障模型](#-故障模型)
    - [故障类型分类](#故障类型分类)
    - [Fail-Stop 模型](#fail-stop-模型)
    - [拜占庭故障模型](#拜占庭故障模型)
  - [🔍 故障检测机制](#-故障检测机制)
    - [心跳检测](#心跳检测)
    - [SWIM 故障检测](#swim-故障检测)
  - [🛡️ 容错策略](#️-容错策略)
    - [冗余机制](#冗余机制)
    - [重试机制](#重试机制)
    - [熔断器模式](#熔断器模式)
  - [🔄 恢复机制](#-恢复机制)
    - [自动恢复](#自动恢复)
  - [🧪 测试策略](#-测试策略)
    - [故障注入测试](#故障注入测试)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

故障处理是分布式系统的核心能力，包括故障模型定义、故障检测机制、容错策略和恢复机制。

## 🎯 故障模型

### 故障类型分类

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FailureType {
    // 节点故障
    NodeCrash,           // 节点崩溃
    NodeByzantine,       // 拜占庭故障
    NodeSlow,            // 节点响应缓慢
    
    // 网络故障
    NetworkPartition,    // 网络分区
    NetworkDelay,        // 网络延迟
    MessageLoss,         // 消息丢失
    MessageDuplication,  // 消息重复
    
    // 存储故障
    StorageCorruption,   // 存储损坏
    StorageFull,         // 存储空间不足
    StorageSlow,         // 存储响应缓慢
    
    // 时钟故障
    ClockSkew,           // 时钟偏差
    ClockDrift,          // 时钟漂移
    ClockStop,           // 时钟停止
}

#[derive(Debug, Clone)]
pub struct FailureModel {
    pub failure_type: FailureType,
    pub probability: f64,
    pub duration: Duration,
    pub recovery_time: Duration,
}
```

### Fail-Stop 模型

```rust
pub struct FailStopModel {
    nodes: Vec<Node>,
    failed_nodes: HashSet<String>,
}

impl FailStopModel {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,
            failed_nodes: HashSet::new(),
        }
    }
    
    pub async fn simulate_node_failure(&mut self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 节点立即停止响应
        self.failed_nodes.insert(node_id.to_string());
        
        // 通知其他节点
        for node in &self.nodes {
            if node.id() != node_id {
                node.notify_node_failure(node_id).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn simulate_node_recovery(&mut self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 节点恢复
        self.failed_nodes.remove(node_id);
        
        // 通知其他节点
        for node in &self.nodes {
            if node.id() != node_id {
                node.notify_node_recovery(node_id).await?;
            }
        }
        
        Ok(())
    }
    
    pub fn is_node_failed(&self, node_id: &str) -> bool {
        self.failed_nodes.contains(node_id)
    }
}
```

### 拜占庭故障模型

```rust
pub struct ByzantineFailureModel {
    nodes: Vec<Node>,
    byzantine_nodes: HashSet<String>,
    failure_behavior: ByzantineBehavior,
}

#[derive(Debug, Clone)]
pub enum ByzantineBehavior {
    // 发送错误消息
    SendWrongMessage,
    // 不发送消息
    SendNoMessage,
    // 发送延迟消息
    SendDelayedMessage(Duration),
    // 发送重复消息
    SendDuplicateMessage,
}

impl ByzantineFailureModel {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,
            byzantine_nodes: HashSet::new(),
            failure_behavior: ByzantineBehavior::SendWrongMessage,
        }
    }
    
    pub async fn simulate_byzantine_failure(
        &mut self,
        node_id: &str,
        behavior: ByzantineBehavior,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.byzantine_nodes.insert(node_id.to_string());
        self.failure_behavior = behavior;
        
        // 根据故障行为修改节点行为
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id() == node_id) {
            node.set_byzantine_behavior(behavior.clone()).await?;
        }
        
        Ok(())
    }
    
    pub fn is_byzantine_node(&self, node_id: &str) -> bool {
        self.byzantine_nodes.contains(node_id)
    }
    
    pub fn get_byzantine_behavior(&self) -> &ByzantineBehavior {
        &self.failure_behavior
    }
}
```

## 🔍 故障检测机制

### 心跳检测

```rust
pub struct HeartbeatDetector {
    nodes: HashMap<String, NodeInfo>,
    heartbeat_interval: Duration,
    timeout: Duration,
    detector_task: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub last_heartbeat: Option<Instant>,
    pub status: NodeStatus,
    pub failure_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Healthy,
    Suspect,
    Failed,
}

impl HeartbeatDetector {
    pub fn new(heartbeat_interval: Duration, timeout: Duration) -> Self {
        Self {
            nodes: HashMap::new(),
            heartbeat_interval,
            timeout,
            detector_task: None,
        }
    }
    
    pub fn add_node(&mut self, node_id: String) {
        self.nodes.insert(node_id, NodeInfo {
            id: node_id,
            last_heartbeat: None,
            status: NodeStatus::Healthy,
            failure_count: 0,
        });
    }
    
    pub async fn start_detection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut nodes = self.nodes.clone();
        let heartbeat_interval = self.heartbeat_interval;
        let timeout = self.timeout;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                // 发送心跳
                for (node_id, node_info) in &mut nodes {
                    if let Err(_) = Self::send_heartbeat(node_id).await {
                        node_info.failure_count += 1;
                    } else {
                        node_info.last_heartbeat = Some(Instant::now());
                        node_info.failure_count = 0;
                    }
                }
                
                // 检查超时
                let now = Instant::now();
                for (node_id, node_info) in &mut nodes {
                    if let Some(last_heartbeat) = node_info.last_heartbeat {
                        if now.duration_since(last_heartbeat) > timeout {
                            node_info.status = NodeStatus::Failed;
                        } else if now.duration_since(last_heartbeat) > timeout / 2 {
                            node_info.status = NodeStatus::Suspect;
                        } else {
                            node_info.status = NodeStatus::Healthy;
                        }
                    }
                }
            }
        });
        
        self.detector_task = Some(handle);
        Ok(())
    }
    
    async fn send_heartbeat(node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 实现心跳发送逻辑
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
    
    pub fn get_node_status(&self, node_id: &str) -> Option<NodeStatus> {
        self.nodes.get(node_id).map(|info| info.status.clone())
    }
    
    pub fn get_failed_nodes(&self) -> Vec<String> {
        self.nodes.iter()
            .filter(|(_, info)| info.status == NodeStatus::Failed)
            .map(|(id, _)| id.clone())
            .collect()
    }
}
```

### SWIM 故障检测

```rust
pub struct SWIMFailureDetector {
    nodes: HashMap<String, SwimNodeInfo>,
    protocol_period: Duration,
    ping_timeout: Duration,
    indirect_ping_count: usize,
    suspicion_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct SwimNodeInfo {
    pub id: String,
    pub status: SwimNodeStatus,
    pub incarnation: u64,
    pub last_seen: Instant,
    pub suspicion_start: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwimNodeStatus {
    Alive,
    Suspect,
    Dead,
}

impl SWIMFailureDetector {
    pub fn new(
        protocol_period: Duration,
        ping_timeout: Duration,
        indirect_ping_count: usize,
        suspicion_timeout: Duration,
    ) -> Self {
        Self {
            nodes: HashMap::new(),
            protocol_period,
            ping_timeout,
            indirect_ping_count,
            suspicion_timeout,
        }
    }
    
    pub async fn start_protocol(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = tokio::time::interval(self.protocol_period);
        
        loop {
            interval.tick().await;
            
            // 选择随机节点进行探测
            if let Some(target_node) = self.select_random_node() {
                self.probe_node(target_node).await?;
            }
            
            // 处理超时的怀疑节点
            self.handle_suspicion_timeout().await?;
        }
    }
    
    async fn probe_node(&mut self, target_node: String) -> Result<(), Box<dyn std::error::Error>> {
        // 直接探测
        match self.direct_probe(&target_node).await {
            Ok(_) => {
                // 探测成功，更新节点状态
                if let Some(node_info) = self.nodes.get_mut(&target_node) {
                    node_info.status = SwimNodeStatus::Alive;
                    node_info.last_seen = Instant::now();
                    node_info.suspicion_start = None;
                }
                return Ok(());
            }
            Err(_) => {
                // 直接探测失败，尝试间接探测
            }
        }
        
        // 间接探测
        let indirect_success = self.indirect_probe(&target_node).await?;
        
        if indirect_success {
            // 间接探测成功
            if let Some(node_info) = self.nodes.get_mut(&target_node) {
                node_info.status = SwimNodeStatus::Alive;
                node_info.last_seen = Instant::now();
                node_info.suspicion_start = None;
            }
        } else {
            // 间接探测也失败，标记为怀疑
            if let Some(node_info) = self.nodes.get_mut(&target_node) {
                if node_info.status == SwimNodeStatus::Alive {
                    node_info.status = SwimNodeStatus::Suspect;
                    node_info.suspicion_start = Some(Instant::now());
                    node_info.incarnation += 1;
                }
            }
        }
        
        Ok(())
    }
    
    async fn direct_probe(&self, target_node: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 实现直接探测逻辑
        tokio::time::timeout(self.ping_timeout, async {
            // 发送 ping 消息
            self.send_ping(target_node).await?;
            
            // 等待 pong 响应
            self.wait_for_pong(target_node).await?;
            
            Ok(())
        }).await??;
        
        Ok(())
    }
    
    async fn indirect_probe(&self, target_node: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut success_count = 0;
        let mut tasks = Vec::new();
        
        // 选择多个中间节点进行间接探测
        let intermediate_nodes = self.select_intermediate_nodes(target_node, self.indirect_ping_count);
        
        for intermediate_node in intermediate_nodes {
            let target = target_node.to_string();
            let task = tokio::spawn(async move {
                // 通过中间节点探测目标节点
                Self::indirect_ping_via_node(&intermediate_node, &target).await
            });
            tasks.push(task);
        }
        
        // 等待所有间接探测完成
        for task in tasks {
            if let Ok(Ok(true)) = task.await {
                success_count += 1;
            }
        }
        
        // 如果至少有一个间接探测成功，则认为目标节点存活
        Ok(success_count > 0)
    }
    
    async fn handle_suspicion_timeout(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Instant::now();
        let mut nodes_to_remove = Vec::new();
        
        for (node_id, node_info) in &mut self.nodes {
            if let Some(suspicion_start) = node_info.suspicion_start {
                if now.duration_since(suspicion_start) > self.suspicion_timeout {
                    node_info.status = SwimNodeStatus::Dead;
                    nodes_to_remove.push(node_id.clone());
                }
            }
        }
        
        // 移除死亡节点
        for node_id in nodes_to_remove {
            self.nodes.remove(&node_id);
        }
        
        Ok(())
    }
    
    fn select_random_node(&self) -> Option<String> {
        let alive_nodes: Vec<String> = self.nodes.iter()
            .filter(|(_, info)| info.status == SwimNodeStatus::Alive)
            .map(|(id, _)| id.clone())
            .collect();
        
        if alive_nodes.is_empty() {
            None
        } else {
            let index = rand::random::<usize>() % alive_nodes.len();
            Some(alive_nodes[index].clone())
        }
    }
    
    fn select_intermediate_nodes(&self, target_node: &str, count: usize) -> Vec<String> {
        self.nodes.iter()
            .filter(|(id, info)| {
                *id != target_node && info.status == SwimNodeStatus::Alive
            })
            .take(count)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    async fn send_ping(&self, target_node: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 实现 ping 发送逻辑
        Ok(())
    }
    
    async fn wait_for_pong(&self, target_node: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 实现 pong 等待逻辑
        Ok(())
    }
    
    async fn indirect_ping_via_node(intermediate_node: &str, target_node: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // 实现通过中间节点的间接探测
        Ok(true)
    }
}
```

## 🛡️ 容错策略

### 冗余机制

```rust
pub struct RedundancyManager {
    primary_nodes: Vec<Node>,
    backup_nodes: Vec<Node>,
    redundancy_level: usize,
}

impl RedundancyManager {
    pub fn new(primary_nodes: Vec<Node>, backup_nodes: Vec<Node>, redundancy_level: usize) -> Self {
        Self {
            primary_nodes,
            backup_nodes,
            redundancy_level,
        }
    }
    
    pub async fn execute_with_redundancy<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        F: Fn(&Node) -> Fut + Copy,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let mut results = Vec::new();
        let mut tasks = Vec::new();
        
        // 在主要节点上执行操作
        for node in &self.primary_nodes {
            let task = tokio::spawn(async move {
                operation(node).await
            });
            tasks.push(task);
        }
        
        // 等待结果
        for task in tasks {
            if let Ok(result) = task.await {
                results.push(result);
            }
        }
        
        // 检查是否有足够的结果
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        
        if success_count >= self.redundancy_level {
            // 返回第一个成功的结果
            for result in results {
                if let Ok(value) = result {
                    return Ok(value);
                }
            }
        }
        
        Err("Insufficient successful operations".into())
    }
    
    pub async fn failover(&mut self, failed_node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 从主要节点中移除故障节点
        self.primary_nodes.retain(|node| node.id() != failed_node_id);
        
        // 从备份节点中提升一个节点到主要节点
        if let Some(backup_node) = self.backup_nodes.pop() {
            self.primary_nodes.push(backup_node);
        }
        
        Ok(())
    }
}
```

### 重试机制

```rust
pub struct RetryManager {
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    jitter: bool,
}

impl RetryManager {
    pub fn new(
        max_retries: usize,
        base_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
        jitter: bool,
    ) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            backoff_multiplier,
            jitter,
        }
    }
    
    pub async fn execute_with_retry<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.max_retries {
                        let delay = self.calculate_delay(attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| "Max retries exceeded".into()))
    }
    
    fn calculate_delay(&self, attempt: usize) -> Duration {
        let mut delay = self.base_delay.as_millis() as f64;
        
        // 指数退避
        for _ in 0..attempt {
            delay *= self.backoff_multiplier;
        }
        
        // 限制最大延迟
        delay = delay.min(self.max_delay.as_millis() as f64);
        
        // 添加抖动
        if self.jitter {
            let jitter = rand::random::<f64>() * delay * 0.1;
            delay += jitter;
        }
        
        Duration::from_millis(delay as u64)
    }
}
```

### 熔断器模式

```rust
pub struct CircuitBreaker {
    failure_threshold: usize,
    recovery_timeout: Duration,
    state: CircuitBreakerState,
    failure_count: usize,
    last_failure_time: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,    // 正常状态
    Open,      // 熔断状态
    HalfOpen,  // 半开状态
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure_time: None,
        }
    }
    
    pub async fn execute<F, Fut, T>(&mut self, operation: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error>>,
    {
        match self.state {
            CircuitBreakerState::Closed => {
                match operation().await {
                    Ok(result) => {
                        self.on_success();
                        Ok(result)
                    }
                    Err(e) => {
                        self.on_failure();
                        Err(e)
                    }
                }
            }
            CircuitBreakerState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitBreakerState::HalfOpen;
                    self.execute(operation).await
                } else {
                    Err("Circuit breaker is open".into())
                }
            }
            CircuitBreakerState::HalfOpen => {
                match operation().await {
                    Ok(result) => {
                        self.on_success();
                        Ok(result)
                    }
                    Err(e) => {
                        self.on_failure();
                        Err(e)
                    }
                }
            }
        }
    }
    
    fn on_success(&mut self) {
        self.failure_count = 0;
        self.last_failure_time = None;
        self.state = CircuitBreakerState::Closed;
    }
    
    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
    
    fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = self.last_failure_time {
            Instant::now().duration_since(last_failure) >= self.recovery_timeout
        } else {
            false
        }
    }
}
```

## 🔄 恢复机制

### 自动恢复

```rust
pub struct AutoRecoveryManager {
    recovery_strategies: HashMap<FailureType, Box<dyn RecoveryStrategy>>,
    recovery_queue: mpsc::UnboundedSender<RecoveryTask>,
}

#[derive(Debug, Clone)]
pub struct RecoveryTask {
    pub failure_type: FailureType,
    pub affected_nodes: Vec<String>,
    pub priority: RecoveryPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryPriority {
    Low,
    Medium,
    High,
    Critical,
}

pub trait RecoveryStrategy {
    async fn recover(&self, task: &RecoveryTask) -> Result<(), Box<dyn std::error::Error>>;
    fn can_handle(&self, failure_type: &FailureType) -> bool;
}

impl AutoRecoveryManager {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // 启动恢复处理器
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                // 处理恢复任务
                Self::process_recovery_task(task).await;
            }
        });
        
        Self {
            recovery_strategies: HashMap::new(),
            recovery_queue: tx,
        }
    }
    
    pub fn register_strategy(&mut self, failure_type: FailureType, strategy: Box<dyn RecoveryStrategy>) {
        self.recovery_strategies.insert(failure_type, strategy);
    }
    
    pub async fn schedule_recovery(&self, task: RecoveryTask) -> Result<(), Box<dyn std::error::Error>> {
        self.recovery_queue.send(task)?;
        Ok(())
    }
    
    async fn process_recovery_task(task: RecoveryTask) {
        // 实现恢复任务处理逻辑
        println!("Processing recovery task: {:?}", task);
    }
}
```

## 🧪 测试策略

### 故障注入测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_heartbeat_detection() {
        let mut detector = HeartbeatDetector::new(
            Duration::from_millis(100),
            Duration::from_millis(300),
        );
        
        detector.add_node("node1".to_string());
        detector.add_node("node2".to_string());
        
        detector.start_detection().await.unwrap();
        
        // 等待一段时间
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // 检查节点状态
        let status1 = detector.get_node_status("node1");
        let status2 = detector.get_node_status("node2");
        
        assert!(status1.is_some());
        assert!(status2.is_some());
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(1));
        
        // 模拟连续失败
        for _ in 0..3 {
            let result = circuit_breaker.execute(|| async {
                Err::<(), _>("Operation failed".into())
            }).await;
            
            assert!(result.is_err());
        }
        
        // 检查熔断器状态
        assert_eq!(circuit_breaker.state, CircuitBreakerState::Open);
        
        // 尝试执行操作（应该被拒绝）
        let result = circuit_breaker.execute(|| async {
            Ok(())
        }).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_retry_manager() {
        let retry_manager = RetryManager::new(
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
            2.0,
            false,
        );
        
        let mut attempt_count = 0;
        
        let result = retry_manager.execute_with_retry(|| async {
            attempt_count += 1;
            if attempt_count < 3 {
                Err("Temporary failure".into())
            } else {
                Ok("Success".to_string())
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempt_count, 3);
    }
}
```

## 📚 进一步阅读

- [成员管理](../membership/README.md) - 成员检测和管理
- [共识机制](../consensus/README.md) - 共识算法的容错
- [一致性模型](../consistency/README.md) - 一致性保证
- [性能优化](../performance/OPTIMIZATION.md) - 故障处理性能优化

## 🔗 相关文档

- [成员管理](../membership/README.md)
- [共识机制](../consensus/README.md)
- [一致性模型](../consistency/README.md)
- [性能优化](../performance/OPTIMIZATION.md)
- [测试策略](../testing/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

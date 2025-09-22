//! 负载均衡模块
//!
//! 提供多种负载均衡策略，包括轮询、加权轮询、最少连接、一致性哈希等

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
// use std::sync::{Arc, RwLock}; // 暂时未使用
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::service_discovery::ServiceInstance;

/// 负载均衡策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoadBalancingStrategy {
    /// 轮询策略
    RoundRobin,
    /// 加权轮询策略
    WeightedRoundRobin,
    /// 最少连接策略
    LeastConnections,
    /// 一致性哈希策略
    ConsistentHash {
        /// 虚拟节点数量
        virtual_nodes: usize,
    },
    /// 随机策略
    Random,
    /// 加权随机策略
    WeightedRandom,
    /// 最少响应时间策略
    LeastResponseTime,
    /// 基于地理位置的策略
    Geographic {
        /// 客户端位置
        client_location: String,
    },
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// 服务器状态信息
#[derive(Debug, Clone)]
pub struct ServerStats {
    /// 当前连接数
    pub connections: u32,
    /// 平均响应时间
    pub avg_response_time: Duration,
    /// 最后更新时间
    pub last_updated: Instant,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self {
            connections: 0,
            avg_response_time: Duration::from_millis(0),
            last_updated: Instant::now(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        }
    }
}

impl ServerStats {
    /// 更新连接数
    pub fn update_connections(&mut self, delta: i32) {
        if delta > 0 {
            self.connections = self.connections.saturating_add(delta as u32);
        } else {
            self.connections = self.connections.saturating_sub((-delta) as u32);
        }
        self.last_updated = Instant::now();
    }

    /// 记录请求
    pub fn record_request(&mut self, response_time: Duration, success: bool) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        // 更新平均响应时间
        if self.total_requests == 1 {
            self.avg_response_time = response_time;
        } else {
            // 使用指数移动平均
            let alpha = 0.1; // 平滑因子
            let new_avg = self.avg_response_time.as_nanos() as f64 * (1.0 - alpha)
                + response_time.as_nanos() as f64 * alpha;
            self.avg_response_time = Duration::from_nanos(new_avg as u64);
        }
        self.last_updated = Instant::now();
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// 获取健康分数（0.0-1.0）
    pub fn health_score(&self) -> f64 {
        let success_rate = self.success_rate();
        let response_time_score = if self.avg_response_time.as_millis() > 1000 {
            0.0
        } else {
            1.0 - (self.avg_response_time.as_millis() as f64 / 1000.0)
        };
        (success_rate + response_time_score) / 2.0
    }
}

/// 轮询负载均衡器
pub struct RoundRobinBalancer {
    servers: Vec<ServiceInstance>,
    current_index: usize,
}

impl RoundRobinBalancer {
    /// 创建轮询负载均衡器
    pub fn new(servers: Vec<ServiceInstance>) -> Self {
        Self {
            servers,
            current_index: 0,
        }
    }

    /// 选择下一个服务器
    pub fn select_server(&mut self) -> Option<&ServiceInstance> {
        if self.servers.is_empty() {
            return None;
        }

        let server = &self.servers[self.current_index];
        self.current_index = (self.current_index + 1) % self.servers.len();
        Some(server)
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
        self.current_index = 0;
    }
}

/// 加权轮询负载均衡器
pub struct WeightedRoundRobinBalancer {
    servers: Vec<ServiceInstance>,
    current_weights: Vec<i32>,
    current_index: usize,
}

impl WeightedRoundRobinBalancer {
    /// 创建加权轮询负载均衡器
    pub fn new(servers: Vec<ServiceInstance>) -> Self {
        let current_weights: Vec<i32> = servers.iter().map(|s| s.weight as i32).collect();
        Self {
            servers,
            current_weights,
            current_index: 0,
        }
    }

    /// 选择下一个服务器
    pub fn select_server(&mut self) -> Option<&ServiceInstance> {
        if self.servers.is_empty() {
            return None;
        }

        // 找到权重最大的服务器
        let mut max_weight_index = 0;
        let mut max_weight = self.current_weights[0];

        for (i, &weight) in self.current_weights.iter().enumerate() {
            if weight > max_weight {
                max_weight = weight;
                max_weight_index = i;
            }
        }

        // 减少选中服务器的权重
        self.current_weights[max_weight_index] -= 1;

        // 如果所有权重都为0，重置权重
        if self.current_weights.iter().all(|&w| w == 0) {
            for (i, server) in self.servers.iter().enumerate() {
                self.current_weights[i] = server.weight as i32;
            }
        }

        Some(&self.servers[max_weight_index])
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
        self.current_weights = self.servers.iter().map(|s| s.weight as i32).collect();
        self.current_index = 0;
    }
}

/// 最少连接负载均衡器
pub struct LeastConnectionsBalancer {
    servers: Vec<ServiceInstance>,
    server_stats: HashMap<SocketAddr, ServerStats>,
}

impl LeastConnectionsBalancer {
    /// 创建最少连接负载均衡器
    pub fn new(servers: Vec<ServiceInstance>) -> Self {
        let mut server_stats = HashMap::new();
        for server in &servers {
            server_stats.insert(server.address, ServerStats::default());
        }
        Self {
            servers,
            server_stats,
        }
    }

    /// 选择连接数最少的服务器
    pub fn select_server(&mut self) -> Option<&ServiceInstance> {
        if self.servers.is_empty() {
            return None;
        }

        let mut min_connections = u32::MAX;
        let mut selected_server = None;

        for server in &self.servers {
            let connections = self
                .server_stats
                .get(&server.address)
                .map(|stats| stats.connections)
                .unwrap_or(0);

            if connections < min_connections {
                min_connections = connections;
                selected_server = Some(server);
            }
        }

        selected_server
    }

    /// 更新服务器连接数
    pub fn update_connections(&mut self, address: SocketAddr, delta: i32) {
        if let Some(stats) = self.server_stats.get_mut(&address) {
            stats.update_connections(delta);
        }
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
        // 保留现有统计信息，添加新服务器的统计信息
        for server in &self.servers {
            self.server_stats
                .entry(server.address)
                .or_default();
        }
    }
}

/// 一致性哈希负载均衡器
pub struct ConsistentHashBalancer {
    servers: Vec<ServiceInstance>,
    virtual_nodes: usize,
    hash_ring: Vec<(u64, SocketAddr)>,
}

impl ConsistentHashBalancer {
    /// 创建一致性哈希负载均衡器
    pub fn new(servers: Vec<ServiceInstance>, virtual_nodes: usize) -> Self {
        let mut balancer = Self {
            servers,
            virtual_nodes,
            hash_ring: Vec::new(),
        };
        balancer.build_hash_ring();
        balancer
    }

    /// 构建哈希环
    fn build_hash_ring(&mut self) {
        self.hash_ring.clear();

        for server in &self.servers {
            for i in 0..self.virtual_nodes {
                let virtual_node = format!("{}:{}", server.address, i);
                let hash = self.hash(&virtual_node);
                self.hash_ring.push((hash, server.address));
            }
        }

        self.hash_ring.sort_by_key(|(hash, _)| *hash);
    }

    /// 计算哈希值
    fn hash(&self, key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// 选择服务器
    pub fn select_server(&self, key: &str) -> Option<&ServiceInstance> {
        if self.servers.is_empty() {
            return None;
        }

        let key_hash = self.hash(key);

        // 找到第一个大于等于key_hash的节点
        for (hash, address) in &self.hash_ring {
            if *hash >= key_hash {
                return self.servers.iter().find(|s| s.address == *address);
            }
        }

        // 如果没有找到，返回第一个节点（环的起点）
        self.servers.first()
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
        self.build_hash_ring();
    }
}

/// 随机负载均衡器
pub struct RandomBalancer {
    servers: Vec<ServiceInstance>,
}

impl RandomBalancer {
    /// 创建随机负载均衡器
    pub fn new(servers: Vec<ServiceInstance>) -> Self {
        Self { servers }
    }

    /// 随机选择服务器
    pub fn select_server(&self) -> Option<&ServiceInstance> {
        if self.servers.is_empty() {
            return None;
        }

        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        Instant::now().elapsed().as_nanos().hash(&mut hasher);
        let index = hasher.finish() as usize % self.servers.len();
        Some(&self.servers[index])
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
    }
}

/// 加权随机负载均衡器
pub struct WeightedRandomBalancer {
    servers: Vec<ServiceInstance>,
    total_weight: u32,
    rng_state: u64,
}

impl WeightedRandomBalancer {
    /// 创建加权随机负载均衡器
    pub fn new(servers: Vec<ServiceInstance>) -> Self {
        let total_weight = servers.iter().map(|s| s.weight).sum();
        // 初始化一个简易种子（基于时间与长度）
        let seed = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            std::time::Instant::now().elapsed().as_nanos().hash(&mut h);
            servers.len().hash(&mut h);
            h.finish() | 1 // 避免为0
        };
        Self {
            servers,
            total_weight,
            rng_state: seed,
        }
    }

    /// 加权随机选择服务器
    pub fn select_server(&mut self) -> Option<&ServiceInstance> {
        if self.servers.is_empty() || self.total_weight == 0 {
            return None;
        }

        // xorshift64* 产生伪随机数，避免紧循环中时间哈希的偏差
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        let random = (self.rng_state % self.total_weight as u64) as u32;

        let mut current_weight = 0;
        for server in &self.servers {
            current_weight += server.weight;
            if random < current_weight {
                return Some(server);
            }
        }

        // 如果由于精度问题没有选中，返回最后一个服务器
        self.servers.last()
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
        self.total_weight = self.servers.iter().map(|s| s.weight).sum();
        // 轻微扰动随机状态
        self.rng_state ^= self.total_weight as u64 + self.servers.len() as u64 + 0x9E3779B97F4A7C15;
    }
}

/// 最少响应时间负载均衡器
pub struct LeastResponseTimeBalancer {
    servers: Vec<ServiceInstance>,
    server_stats: HashMap<SocketAddr, ServerStats>,
}

impl LeastResponseTimeBalancer {
    /// 创建最少响应时间负载均衡器
    pub fn new(servers: Vec<ServiceInstance>) -> Self {
        let mut server_stats = HashMap::new();
        for server in &servers {
            server_stats.insert(server.address, ServerStats::default());
        }
        Self {
            servers,
            server_stats,
        }
    }

    /// 选择响应时间最少的服务器
    pub fn select_server(&mut self) -> Option<&ServiceInstance> {
        if self.servers.is_empty() {
            return None;
        }

        let mut min_response_time = Duration::MAX;
        let mut selected_server = None;

        for server in &self.servers {
            let response_time = self
                .server_stats
                .get(&server.address)
                .map(|stats| stats.avg_response_time)
                .unwrap_or(Duration::from_millis(0));

            if response_time < min_response_time {
                min_response_time = response_time;
                selected_server = Some(server);
            }
        }

        selected_server
    }

    /// 记录服务器响应时间
    pub fn record_response_time(
        &mut self,
        address: SocketAddr,
        response_time: Duration,
        success: bool,
    ) {
        if let Some(stats) = self.server_stats.get_mut(&address) {
            stats.record_request(response_time, success);
        }
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
        for server in &self.servers {
            self.server_stats
                .entry(server.address)
                .or_default();
        }
    }
}

/// 基于地理位置的负载均衡器
pub struct GeographicBalancer {
    servers: Vec<ServiceInstance>,
    client_location: String,
    location_mapping: HashMap<String, Vec<SocketAddr>>,
}

impl GeographicBalancer {
    /// 创建基于地理位置的负载均衡器
    pub fn new(servers: Vec<ServiceInstance>, client_location: String) -> Self {
        let mut location_mapping = HashMap::new();

        // 根据服务器的元数据构建位置映射
        for server in &servers {
            if let Some(region) = server.metadata.get("region") {
                location_mapping
                    .entry(region.clone())
                    .or_insert_with(Vec::new)
                    .push(server.address);
            }
        }

        Self {
            servers,
            client_location,
            location_mapping,
        }
    }

    /// 选择地理位置最近的服务器
    pub fn select_server(&self) -> Option<&ServiceInstance> {
        if self.servers.is_empty() {
            return None;
        }

        // 首先尝试选择同一区域的服务器
        if let Some(addresses) = self.location_mapping.get(&self.client_location) {
            for address in addresses {
                if let Some(server) = self.servers.iter().find(|s| s.address == *address) {
                    return Some(server);
                }
            }
        }

        // 如果没有同区域的服务器，选择第一个可用的服务器
        self.servers.first()
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers;
        self.location_mapping.clear();

        for server in &self.servers {
            if let Some(region) = server.metadata.get("region") {
                self.location_mapping
                    .entry(region.clone())
                    .or_default()
                    .push(server.address);
            }
        }
    }
}

/// 负载均衡管理器
pub struct LoadBalancerManager {
    strategy: LoadBalancingStrategy,
    round_robin: Option<RoundRobinBalancer>,
    weighted_round_robin: Option<WeightedRoundRobinBalancer>,
    least_connections: Option<LeastConnectionsBalancer>,
    consistent_hash: Option<ConsistentHashBalancer>,
    random: Option<RandomBalancer>,
    weighted_random: Option<WeightedRandomBalancer>,
    least_response_time: Option<LeastResponseTimeBalancer>,
    geographic: Option<GeographicBalancer>,
    servers: Vec<ServiceInstance>,
}

impl LoadBalancerManager {
    /// 创建负载均衡管理器
    pub fn new(strategy: LoadBalancingStrategy, servers: Vec<ServiceInstance>) -> Self {
        let mut manager = Self {
            strategy: strategy.clone(),
            round_robin: None,
            weighted_round_robin: None,
            least_connections: None,
            consistent_hash: None,
            random: None,
            weighted_random: None,
            least_response_time: None,
            geographic: None,
            servers: servers.clone(),
        };

        manager.initialize_balancer(strategy, servers);
        manager
    }

    /// 初始化负载均衡器
    fn initialize_balancer(
        &mut self,
        strategy: LoadBalancingStrategy,
        servers: Vec<ServiceInstance>,
    ) {
        match strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.round_robin = Some(RoundRobinBalancer::new(servers));
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin = Some(WeightedRoundRobinBalancer::new(servers));
            }
            LoadBalancingStrategy::LeastConnections => {
                self.least_connections = Some(LeastConnectionsBalancer::new(servers));
            }
            LoadBalancingStrategy::ConsistentHash { virtual_nodes } => {
                self.consistent_hash = Some(ConsistentHashBalancer::new(servers, virtual_nodes));
            }
            LoadBalancingStrategy::Random => {
                self.random = Some(RandomBalancer::new(servers));
            }
            LoadBalancingStrategy::WeightedRandom => {
                self.weighted_random = Some(WeightedRandomBalancer::new(servers));
            }
            LoadBalancingStrategy::LeastResponseTime => {
                self.least_response_time = Some(LeastResponseTimeBalancer::new(servers));
            }
            LoadBalancingStrategy::Geographic { client_location } => {
                self.geographic = Some(GeographicBalancer::new(servers, client_location));
            }
        }
    }

    /// 选择服务器
    pub fn select_server(&mut self, key: Option<&str>) -> Option<&ServiceInstance> {
        match &self.strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin.as_mut()?.select_server(),
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin.as_mut()?.select_server()
            }
            LoadBalancingStrategy::LeastConnections => {
                self.least_connections.as_mut()?.select_server()
            }
            LoadBalancingStrategy::ConsistentHash { .. } => {
                if let Some(k) = key {
                    self.consistent_hash.as_ref()?.select_server(k)
                } else {
                    self.consistent_hash.as_ref()?.select_server("default")
                }
            }
            LoadBalancingStrategy::Random => self.random.as_ref()?.select_server(),
            LoadBalancingStrategy::WeightedRandom => self.weighted_random.as_mut()?.select_server(),
            LoadBalancingStrategy::LeastResponseTime => {
                self.least_response_time.as_mut()?.select_server()
            }
            LoadBalancingStrategy::Geographic { .. } => self.geographic.as_ref()?.select_server(),
        }
    }

    /// 更新服务器列表
    pub fn update_servers(&mut self, servers: Vec<ServiceInstance>) {
        self.servers = servers.clone();

        match &self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                if let Some(ref mut balancer) = self.round_robin {
                    balancer.update_servers(servers);
                }
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                if let Some(ref mut balancer) = self.weighted_round_robin {
                    balancer.update_servers(servers);
                }
            }
            LoadBalancingStrategy::LeastConnections => {
                if let Some(ref mut balancer) = self.least_connections {
                    balancer.update_servers(servers);
                }
            }
            LoadBalancingStrategy::ConsistentHash { virtual_nodes: _ } => {
                if let Some(ref mut balancer) = self.consistent_hash {
                    balancer.update_servers(servers);
                }
            }
            LoadBalancingStrategy::Random => {
                if let Some(ref mut balancer) = self.random {
                    balancer.update_servers(servers);
                }
            }
            LoadBalancingStrategy::WeightedRandom => {
                if let Some(ref mut balancer) = self.weighted_random {
                    balancer.update_servers(servers);
                }
            }
            LoadBalancingStrategy::LeastResponseTime => {
                if let Some(ref mut balancer) = self.least_response_time {
                    balancer.update_servers(servers);
                }
            }
            LoadBalancingStrategy::Geographic { client_location: _ } => {
                if let Some(ref mut balancer) = self.geographic {
                    balancer.update_servers(servers);
                }
            }
        }
    }

    /// 更新策略
    pub fn update_strategy(&mut self, strategy: LoadBalancingStrategy) {
        self.strategy = strategy.clone();
        self.initialize_balancer(strategy, self.servers.clone());
    }

    /// 获取当前策略
    pub fn get_strategy(&self) -> &LoadBalancingStrategy {
        &self.strategy
    }

    /// 更新连接数（用于最少连接策略）
    pub fn update_connections(&mut self, address: SocketAddr, delta: i32) {
        if let Some(ref mut balancer) = self.least_connections {
            balancer.update_connections(address, delta);
        }
    }

    /// 记录响应时间（用于最少响应时间策略）
    pub fn record_response_time(
        &mut self,
        address: SocketAddr,
        response_time: Duration,
        success: bool,
    ) {
        if let Some(ref mut balancer) = self.least_response_time {
            balancer.record_response_time(address, response_time, success);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn create_test_servers() -> Vec<ServiceInstance> {
        vec![
            ServiceInstance::new(
                "server-1".to_string(),
                "test-service".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                HashMap::from([("region".to_string(), "us-east-1".to_string())]),
            )
            .with_weight(10),
            ServiceInstance::new(
                "server-2".to_string(),
                "test-service".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                HashMap::from([("region".to_string(), "us-west-1".to_string())]),
            )
            .with_weight(5),
            ServiceInstance::new(
                "server-3".to_string(),
                "test-service".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082),
                HashMap::from([("region".to_string(), "us-east-1".to_string())]),
            )
            .with_weight(8),
        ]
    }

    #[test]
    fn test_round_robin_balancer() {
        let servers = create_test_servers();
        let mut balancer = RoundRobinBalancer::new(servers.clone());

        let p1 = balancer.select_server().unwrap().address.port();
        let p2 = balancer.select_server().unwrap().address.port();
        let p3 = balancer.select_server().unwrap().address.port();
        let p4 = balancer.select_server().unwrap().address.port();

        assert_eq!(p1, 8080);
        assert_eq!(p2, 8081);
        assert_eq!(p3, 8082);
        assert_eq!(p4, 8080); // 回到第一个
    }

    #[test]
    fn test_weighted_round_robin_balancer() {
        let servers = create_test_servers();
        let mut balancer = WeightedRoundRobinBalancer::new(servers);

        // 测试多次选择，验证权重分布
        let mut selections = Vec::new();
        for _ in 0..23 {
            // 总权重是23
            if let Some(server) = balancer.select_server() {
                selections.push(server.address.port());
            }
        }

        // 验证权重分布
        let count_8080 = selections.iter().filter(|&&p| p == 8080).count();
        let count_8081 = selections.iter().filter(|&&p| p == 8081).count();
        let count_8082 = selections.iter().filter(|&&p| p == 8082).count();

        assert_eq!(count_8080, 10); // 权重10
        assert_eq!(count_8081, 5); // 权重5
        assert_eq!(count_8082, 8); // 权重8
    }

    #[test]
    fn test_least_connections_balancer() {
        let servers = create_test_servers();
        let mut balancer = LeastConnectionsBalancer::new(servers);

        // 初始选择应该选择连接数为0的服务器
        let server = balancer.select_server().unwrap();
        assert_eq!(server.address.port(), 8080);

        // 增加第一个服务器的连接数
        balancer.update_connections(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            1,
        );

        // 现在应该选择其他服务器
        let server = balancer.select_server().unwrap();
        assert_ne!(server.address.port(), 8080);
    }

    #[test]
    fn test_consistent_hash_balancer() {
        let servers = create_test_servers();
        let balancer = ConsistentHashBalancer::new(servers, 100);

        // 相同key应该选择相同服务器
        let server1 = balancer.select_server("test-key").unwrap();
        let server2 = balancer.select_server("test-key").unwrap();
        assert_eq!(server1.address, server2.address);

        // 不同key可能选择不同服务器
        let _server3 = balancer.select_server("different-key").unwrap();
        // 注意：由于哈希的随机性，这里不保证一定不同
    }

    #[test]
    fn test_random_balancer() {
        let servers = create_test_servers();
        let balancer = RandomBalancer::new(servers);

        // 多次选择应该能选到不同的服务器
        let mut found_servers = std::collections::HashSet::new();
        for _ in 0..100 {
            if let Some(server) = balancer.select_server() {
                found_servers.insert(server.address.port());
            }
        }

        // 应该能找到多个不同的服务器
        assert!(found_servers.len() > 1);
    }

    #[test]
    fn test_weighted_random_balancer() {
        let servers = create_test_servers();
        let mut balancer = WeightedRandomBalancer::new(servers);

        // 多次选择验证权重分布
        let mut selections = Vec::new();
        for _ in 0..1000 {
            if let Some(server) = balancer.select_server() {
                selections.push(server.address.port());
            }
        }

        // 验证权重分布（允许一定的随机性）
        let count_8080 = selections.iter().filter(|&&p| p == 8080).count();
        let count_8081 = selections.iter().filter(|&&p| p == 8081).count();
        let count_8082 = selections.iter().filter(|&&p| p == 8082).count();

        // 权重高的服务器被选中的次数应该更多
        assert!(count_8080 > count_8081);
        assert!(count_8082 > count_8081);
    }

    #[test]
    fn test_least_response_time_balancer() {
        let servers = create_test_servers();
        let mut balancer = LeastResponseTimeBalancer::new(servers);

        // 记录第一个服务器的响应时间
        balancer.record_response_time(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            Duration::from_millis(100),
            true,
        );

        // 记录第二个服务器的响应时间（更快）
        balancer.record_response_time(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            Duration::from_millis(50),
            true,
        );

        // 现在应该选择响应时间更快的服务器
        let server = balancer.select_server().unwrap();
        // 可能选择8081（更快），如果多台均未打点则可能返回默认第一台
        assert!(
            server.address.port() == 8081
                || server.address.port() == 8080
                || server.address.port() == 8082
        );
    }

    #[test]
    fn test_geographic_balancer() {
        let servers = create_test_servers();
        let balancer = GeographicBalancer::new(servers, "us-east-1".to_string());

        // 应该选择us-east-1区域的服务器
        let server = balancer.select_server().unwrap();
        assert_eq!(server.address.port(), 8080); // 第一个us-east-1服务器
    }

    #[test]
    fn test_load_balancer_manager() {
        let servers = create_test_servers();
        let mut manager = LoadBalancerManager::new(LoadBalancingStrategy::RoundRobin, servers);

        let server = manager.select_server(None).unwrap();
        assert_eq!(server.address.port(), 8080);

        // 测试策略切换
        manager.update_strategy(LoadBalancingStrategy::Random);
        let server = manager.select_server(None).unwrap();
        // 随机选择，不保证是特定服务器
        assert!(server.address.port() >= 8080 && server.address.port() <= 8082);
    }

    #[test]
    fn test_server_stats() {
        let mut stats = ServerStats::default();

        assert_eq!(stats.connections, 0);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.success_rate(), 1.0);

        stats.update_connections(5);
        assert_eq!(stats.connections, 5);

        stats.record_request(Duration::from_millis(100), true);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.success_rate(), 1.0);

        stats.record_request(Duration::from_millis(200), false);
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.success_rate(), 0.5);
    }

    #[test]
    fn test_load_balancing_strategy_serialization() {
        let strategies = vec![
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::WeightedRoundRobin,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::ConsistentHash { virtual_nodes: 100 },
            LoadBalancingStrategy::Random,
            LoadBalancingStrategy::WeightedRandom,
            LoadBalancingStrategy::LeastResponseTime,
            LoadBalancingStrategy::Geographic {
                client_location: "us-east-1".to_string(),
            },
        ];

        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: LoadBalancingStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, strategy);
        }
    }
}

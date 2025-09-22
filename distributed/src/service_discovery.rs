//! 服务发现模块
//!
//! 提供基于DNS和配置的服务发现机制，支持多种服务发现策略

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 服务实例信息
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceInstance {
    /// 服务ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务地址
    pub address: SocketAddr,
    /// 服务元数据
    pub metadata: HashMap<String, String>,
    /// 健康检查URL
    pub health_check_url: Option<String>,
    /// 权重（用于负载均衡）
    pub weight: u32,
    /// 最后更新时间
    pub last_updated: Instant,
    /// 是否健康
    pub is_healthy: bool,
}

impl ServiceInstance {
    /// 创建新的服务实例
    pub fn new(
        id: String,
        name: String,
        address: SocketAddr,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            name,
            address,
            metadata,
            health_check_url: None,
            weight: 1,
            last_updated: Instant::now(),
            is_healthy: true,
        }
    }

    /// 设置健康检查URL
    pub fn with_health_check_url(mut self, url: String) -> Self {
        self.health_check_url = Some(url);
        self
    }

    /// 设置权重
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// 更新健康状态
    pub fn update_health(&mut self, is_healthy: bool) {
        self.is_healthy = is_healthy;
        self.last_updated = Instant::now();
    }

    /// 检查实例是否过期
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.last_updated.elapsed() > ttl
    }
}

/// 服务发现策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryStrategy {
    /// DNS服务发现
    Dns {
        /// DNS服务器地址
        dns_server: String,
        /// 查询间隔
        query_interval: Duration,
    },
    /// 配置服务发现
    Config {
        /// 配置文件路径
        config_path: String,
        /// 重载间隔
        reload_interval: Duration,
    },
    /// 注册中心服务发现
    Registry {
        /// 注册中心地址
        registry_url: String,
        /// 心跳间隔
        heartbeat_interval: Duration,
    },
    /// 混合模式
    Hybrid {
        /// 主要策略
        primary: Box<DiscoveryStrategy>,
        /// 备用策略
        fallback: Box<DiscoveryStrategy>,
    },
}

/// 服务发现配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// 服务发现策略
    pub strategy: DiscoveryStrategy,
    /// 服务TTL
    pub service_ttl: Duration,
    /// 健康检查间隔
    pub health_check_interval: Duration,
    /// 最大重试次数
    pub max_retries: usize,
    /// 超时时间
    pub timeout: Duration,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            strategy: DiscoveryStrategy::Config {
                config_path: "services.json".to_string(),
                reload_interval: Duration::from_secs(30),
            },
            service_ttl: Duration::from_secs(300), // 5分钟
            health_check_interval: Duration::from_secs(30),
            max_retries: 3,
            timeout: Duration::from_secs(5),
        }
    }
}

/// DNS服务发现器
pub struct DnsServiceDiscovery {
    _dns_server: String,
    query_interval: Duration,
    last_query: Instant,
}

impl DnsServiceDiscovery {
    /// 创建DNS服务发现器
    pub fn new(dns_server: String, query_interval: Duration) -> Self {
        Self {
            _dns_server: dns_server,
            query_interval,
            last_query: Instant::now(),
        }
    }

    /// 查询DNS记录
    pub fn query_dns(&mut self, service_name: &str) -> Result<Vec<SocketAddr>, String> {
        // 模拟DNS查询
        if self.last_query.elapsed() < self.query_interval {
            return Ok(vec![]);
        }

        self.last_query = Instant::now();

        // 模拟DNS解析结果
        match service_name {
            "user-service" => Ok(vec![
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8080),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8081),
            ]),
            "order-service" => Ok(vec![
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8082),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8083),
            ]),
            _ => Ok(vec![]),
        }
    }

    /// 强制查询DNS记录（忽略时间间隔）
    pub fn force_query_dns(&mut self, service_name: &str) -> Result<Vec<SocketAddr>, String> {
        self.last_query = Instant::now();

        // 模拟DNS解析结果
        match service_name {
            "user-service" => Ok(vec![
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8080),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8081),
            ]),
            "order-service" => Ok(vec![
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8082),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8083),
            ]),
            _ => Ok(vec![]),
        }
    }
}

/// 配置服务发现器
pub struct ConfigServiceDiscovery {
    _config_path: String,
    reload_interval: Duration,
    last_reload: Instant,
    services: HashMap<String, Vec<ServiceInstance>>,
}

impl ConfigServiceDiscovery {
    /// 创建配置服务发现器
    pub fn new(config_path: String, reload_interval: Duration) -> Self {
        Self {
            _config_path: config_path,
            reload_interval,
            last_reload: Instant::now(),
            services: HashMap::new(),
        }
    }

    /// 从配置文件加载服务
    pub fn load_services(&mut self) -> Result<(), String> {
        if self.last_reload.elapsed() < self.reload_interval {
            return Ok(());
        }

        self.last_reload = Instant::now();

        // 模拟从配置文件加载服务
        let mut services = HashMap::new();

        // 模拟用户服务
        let user_services = vec![
            ServiceInstance::new(
                "user-1".to_string(),
                "user-service".to_string(),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8080),
                HashMap::from([
                    ("version".to_string(), "1.0.0".to_string()),
                    ("region".to_string(), "us-east-1".to_string()),
                ]),
            )
            .with_weight(10),
            ServiceInstance::new(
                "user-2".to_string(),
                "user-service".to_string(),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8081),
                HashMap::from([
                    ("version".to_string(), "1.0.0".to_string()),
                    ("region".to_string(), "us-west-1".to_string()),
                ]),
            )
            .with_weight(5),
        ];
        services.insert("user-service".to_string(), user_services);

        // 模拟订单服务
        let order_services = vec![
            ServiceInstance::new(
                "order-1".to_string(),
                "order-service".to_string(),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8082),
                HashMap::from([
                    ("version".to_string(), "2.0.0".to_string()),
                    ("region".to_string(), "us-east-1".to_string()),
                ]),
            )
            .with_weight(8),
        ];
        services.insert("order-service".to_string(), order_services);

        self.services = services;
        Ok(())
    }

    /// 强制从配置文件加载服务（忽略时间间隔）
    pub fn force_load_services(&mut self) -> Result<(), String> {
        self.last_reload = Instant::now();

        // 模拟从配置文件加载服务
        let mut services = HashMap::new();

        // 模拟用户服务
        let user_services = vec![
            ServiceInstance::new(
                "user-1".to_string(),
                "user-service".to_string(),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8080),
                HashMap::from([
                    ("version".to_string(), "1.0.0".to_string()),
                    ("region".to_string(), "us-east-1".to_string()),
                ]),
            )
            .with_weight(10),
            ServiceInstance::new(
                "user-2".to_string(),
                "user-service".to_string(),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8081),
                HashMap::from([
                    ("version".to_string(), "1.0.0".to_string()),
                    ("region".to_string(), "us-west-1".to_string()),
                ]),
            )
            .with_weight(5),
        ];
        services.insert("user-service".to_string(), user_services);

        // 模拟订单服务
        let order_services = vec![
            ServiceInstance::new(
                "order-1".to_string(),
                "order-service".to_string(),
                SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 8082),
                HashMap::from([
                    ("version".to_string(), "2.0.0".to_string()),
                    ("region".to_string(), "us-east-1".to_string()),
                ]),
            )
            .with_weight(8),
        ];
        services.insert("order-service".to_string(), order_services);

        self.services = services;
        Ok(())
    }

    /// 获取服务实例
    pub fn get_services(&self, service_name: &str) -> Vec<ServiceInstance> {
        self.services.get(service_name).cloned().unwrap_or_default()
    }
}

/// 注册中心服务发现器
pub struct RegistryServiceDiscovery {
    _registry_url: String,
    heartbeat_interval: Duration,
    last_heartbeat: Instant,
    registered_services: HashMap<String, Vec<ServiceInstance>>,
}

impl RegistryServiceDiscovery {
    /// 创建注册中心服务发现器
    pub fn new(registry_url: String, heartbeat_interval: Duration) -> Self {
        Self {
            _registry_url: registry_url,
            heartbeat_interval,
            last_heartbeat: Instant::now(),
            registered_services: HashMap::new(),
        }
    }

    /// 注册服务
    pub fn register_service(&mut self, instance: ServiceInstance) -> Result<(), String> {
        let service_name = instance.name.clone();
        self.registered_services
            .entry(service_name)
            .or_default()
            .push(instance);
        Ok(())
    }

    /// 注销服务
    pub fn unregister_service(
        &mut self,
        service_name: &str,
        instance_id: &str,
    ) -> Result<(), String> {
        if let Some(instances) = self.registered_services.get_mut(service_name) {
            instances.retain(|instance| instance.id != instance_id);
            if instances.is_empty() {
                self.registered_services.remove(service_name);
            }
        }
        Ok(())
    }

    /// 发送心跳
    pub fn send_heartbeat(&mut self) -> Result<(), String> {
        if self.last_heartbeat.elapsed() < self.heartbeat_interval {
            return Ok(());
        }

        self.last_heartbeat = Instant::now();

        // 模拟心跳发送
        for instances in self.registered_services.values_mut() {
            for instance in instances.iter_mut() {
                instance.update_health(true);
            }
        }
        Ok(())
    }

    /// 强制发送心跳（忽略时间间隔）
    pub fn force_send_heartbeat(&mut self) -> Result<(), String> {
        self.last_heartbeat = Instant::now();

        // 模拟心跳发送
        for instances in self.registered_services.values_mut() {
            for instance in instances.iter_mut() {
                instance.update_health(true);
            }
        }
        Ok(())
    }

    /// 获取服务实例
    pub fn get_services(&self, service_name: &str) -> Vec<ServiceInstance> {
        self.registered_services
            .get(service_name)
            .cloned()
            .unwrap_or_default()
    }
}

/// 服务发现管理器
pub struct ServiceDiscoveryManager {
    config: ServiceDiscoveryConfig,
    dns_discovery: Option<DnsServiceDiscovery>,
    config_discovery: Option<ConfigServiceDiscovery>,
    registry_discovery: Option<RegistryServiceDiscovery>,
    service_cache: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    health_checker: HealthChecker,
}

/// 健康检查器
pub struct HealthChecker {
    check_interval: Duration,
    last_check: Instant,
}

impl HealthChecker {
    /// 创建健康检查器
    pub fn new(check_interval: Duration) -> Self {
        Self {
            check_interval,
            last_check: Instant::now(),
        }
    }

    /// 检查服务健康状态
    pub fn check_health(&mut self, instances: &mut [ServiceInstance]) {
        if self.last_check.elapsed() < self.check_interval {
            return;
        }

        self.last_check = Instant::now();

        for instance in instances.iter_mut() {
            // 模拟健康检查
            let is_healthy = self.simulate_health_check(instance);
            instance.update_health(is_healthy);
        }
    }

    /// 强制检查服务健康状态（忽略时间间隔）
    pub fn force_check_health(&mut self, instances: &mut [ServiceInstance]) {
        self.last_check = Instant::now();

        for instance in instances.iter_mut() {
            // 模拟健康检查
            let is_healthy = self.simulate_health_check(instance);
            instance.update_health(is_healthy);
        }
    }

    /// 模拟健康检查
    fn simulate_health_check(&self, instance: &ServiceInstance) -> bool {
        // 模拟健康检查逻辑
        // 在实际实现中，这里会发送HTTP请求到健康检查URL
        match instance.name.as_str() {
            "user-service" => true,
            "order-service" => true,
            _ => false,
        }
    }
}

impl ServiceDiscoveryManager {
    /// 创建服务发现管理器
    pub fn new(config: ServiceDiscoveryConfig) -> Self {
        let mut manager = Self {
            config: config.clone(),
            dns_discovery: None,
            config_discovery: None,
            registry_discovery: None,
            service_cache: Arc::new(RwLock::new(HashMap::new())),
            health_checker: HealthChecker::new(config.health_check_interval),
        };

        // 根据策略初始化相应的发现器
        match config.strategy {
            DiscoveryStrategy::Dns {
                dns_server,
                query_interval,
            } => {
                manager.dns_discovery = Some(DnsServiceDiscovery::new(dns_server, query_interval));
            }
            DiscoveryStrategy::Config {
                config_path,
                reload_interval,
            } => {
                manager.config_discovery =
                    Some(ConfigServiceDiscovery::new(config_path, reload_interval));
            }
            DiscoveryStrategy::Registry {
                registry_url,
                heartbeat_interval,
            } => {
                manager.registry_discovery = Some(RegistryServiceDiscovery::new(
                    registry_url,
                    heartbeat_interval,
                ));
            }
            DiscoveryStrategy::Hybrid {
                primary,
                fallback: _,
            } => {
                // 初始化主要和备用发现器
                match *primary {
                    DiscoveryStrategy::Dns {
                        dns_server,
                        query_interval,
                    } => {
                        manager.dns_discovery =
                            Some(DnsServiceDiscovery::new(dns_server, query_interval));
                    }
                    DiscoveryStrategy::Config {
                        config_path,
                        reload_interval,
                    } => {
                        manager.config_discovery =
                            Some(ConfigServiceDiscovery::new(config_path, reload_interval));
                    }
                    DiscoveryStrategy::Registry {
                        registry_url,
                        heartbeat_interval,
                    } => {
                        manager.registry_discovery = Some(RegistryServiceDiscovery::new(
                            registry_url,
                            heartbeat_interval,
                        ));
                    }
                    _ => {}
                }
                // 这里可以添加备用发现器的初始化逻辑
            }
        }

        manager
    }

    /// 发现服务实例
    pub fn discover_services(
        &mut self,
        service_name: &str,
    ) -> Result<Vec<ServiceInstance>, String> {
        // 首先检查缓存
        {
            let cache = self.service_cache.read().unwrap();
            if let Some(cached_instances) = cache.get(service_name) {
                // 检查缓存是否过期
                let valid_instances: Vec<ServiceInstance> = cached_instances
                    .iter()
                    .filter(|instance| !instance.is_expired(self.config.service_ttl))
                    .cloned()
                    .collect();

                if !valid_instances.is_empty() {
                    return Ok(valid_instances);
                }
            }
        }

        // 缓存过期或不存在，重新发现
        let mut instances = Vec::new();

        // 根据策略发现服务
        match &self.config.strategy {
            DiscoveryStrategy::Dns { .. } => {
                if let Some(ref mut dns) = self.dns_discovery {
                    let addresses = dns.force_query_dns(service_name)?;
                    for (i, address) in addresses.iter().enumerate() {
                        let instance = ServiceInstance::new(
                            format!("{}-{}", service_name, i),
                            service_name.to_string(),
                            *address,
                            HashMap::new(),
                        );
                        instances.push(instance);
                    }
                }
            }
            DiscoveryStrategy::Config { .. } => {
                if let Some(ref mut config) = self.config_discovery {
                    config.force_load_services()?;
                    instances = config.get_services(service_name);
                }
            }
            DiscoveryStrategy::Registry { .. } => {
                if let Some(ref registry) = self.registry_discovery {
                    instances = registry.get_services(service_name);
                }
            }
            DiscoveryStrategy::Hybrid { primary, .. } => {
                // 尝试主要策略
                let primary_result = match **primary {
                    DiscoveryStrategy::Dns { .. } => {
                        if let Some(ref mut dns) = self.dns_discovery {
                            dns.force_query_dns(service_name)
                        } else {
                            Err("DNS discovery not initialized".to_string())
                        }
                    }
                    DiscoveryStrategy::Config { .. } => {
                        if let Some(ref mut config) = self.config_discovery {
                            config.force_load_services()?;
                            Ok(config
                                .get_services(service_name)
                                .into_iter()
                                .map(|i| i.address)
                                .collect())
                        } else {
                            Err("Config discovery not initialized".to_string())
                        }
                    }
                    DiscoveryStrategy::Registry { .. } => {
                        if let Some(ref registry) = self.registry_discovery {
                            Ok(registry
                                .get_services(service_name)
                                .into_iter()
                                .map(|i| i.address)
                                .collect())
                        } else {
                            Err("Registry discovery not initialized".to_string())
                        }
                    }
                    _ => Err("Unsupported primary strategy".to_string()),
                };

                match primary_result {
                    Ok(addresses) => {
                        for (i, address) in addresses.iter().enumerate() {
                            let instance = ServiceInstance::new(
                                format!("{}-{}", service_name, i),
                                service_name.to_string(),
                                *address,
                                HashMap::new(),
                            );
                            instances.push(instance);
                        }
                    }
                    Err(_) => {
                        // 主要策略失败，使用备用策略
                        // 这里可以添加备用策略的逻辑
                    }
                }
            }
        }

        // 执行健康检查
        self.health_checker.check_health(&mut instances);

        // 更新缓存
        {
            let mut cache = self.service_cache.write().unwrap();
            cache.insert(service_name.to_string(), instances.clone());
        }

        Ok(instances)
    }

    /// 注册服务实例
    pub fn register_service(&mut self, instance: ServiceInstance) -> Result<(), String> {
        if let Some(ref mut registry) = self.registry_discovery {
            registry.register_service(instance.clone())?;
        }

        // 更新缓存
        {
            let mut cache = self.service_cache.write().unwrap();
            let service_name = instance.name.clone();
            cache
                .entry(service_name)
                .or_default()
                .push(instance);
        }

        Ok(())
    }

    /// 注销服务实例
    pub fn unregister_service(
        &mut self,
        service_name: &str,
        instance_id: &str,
    ) -> Result<(), String> {
        if let Some(ref mut registry) = self.registry_discovery {
            registry.unregister_service(service_name, instance_id)?;
        }

        // 从缓存中移除
        {
            let mut cache = self.service_cache.write().unwrap();
            if let Some(instances) = cache.get_mut(service_name) {
                instances.retain(|instance| instance.id != instance_id);
                if instances.is_empty() {
                    cache.remove(service_name);
                }
            }
        }

        Ok(())
    }

    /// 获取所有服务
    pub fn get_all_services(&self) -> HashMap<String, Vec<ServiceInstance>> {
        self.service_cache.read().unwrap().clone()
    }

    /// 设置缓存直接写入（替换或合并）
    pub fn set_cache_for(
        &self,
        service_name: &str,
        instances: Vec<ServiceInstance>,
        replace: bool,
    ) {
        let mut cache = self.service_cache.write().unwrap();
        if replace {
            cache.insert(service_name.to_string(), instances);
        } else {
            let entry = cache
                .entry(service_name.to_string())
                .or_default();
            entry.extend(instances);
        }
    }

    /// 清空指定服务的缓存
    pub fn clear_cache_for(&self, service_name: &str) {
        let mut cache = self.service_cache.write().unwrap();
        cache.remove(service_name);
    }

    /// 清理过期服务
    pub fn cleanup_expired_services(&mut self) {
        let mut cache = self.service_cache.write().unwrap();
        for (_service_name, instances) in cache.iter_mut() {
            instances.retain(|instance| !instance.is_expired(self.config.service_ttl));
        }
        cache.retain(|_, instances| !instances.is_empty());
    }

    /// 获取配置
    pub fn get_config(&self) -> &ServiceDiscoveryConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ServiceDiscoveryConfig) {
        self.config = config;
        // 这里可以添加重新初始化发现器的逻辑
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_service_instance_creation() {
        let instance = ServiceInstance::new(
            "test-1".to_string(),
            "test-service".to_string(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            HashMap::new(),
        );

        assert_eq!(instance.id, "test-1");
        assert_eq!(instance.name, "test-service");
        assert_eq!(instance.weight, 1);
        assert!(instance.is_healthy);
    }

    #[test]
    fn test_service_instance_with_options() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());

        let instance = ServiceInstance::new(
            "test-1".to_string(),
            "test-service".to_string(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            metadata,
        )
        .with_health_check_url("http://localhost:8080/health".to_string())
        .with_weight(10);

        assert_eq!(instance.weight, 10);
        assert_eq!(
            instance.health_check_url,
            Some("http://localhost:8080/health".to_string())
        );
    }

    #[test]
    fn test_dns_service_discovery() {
        let mut dns = DnsServiceDiscovery::new("8.8.8.8".to_string(), Duration::from_secs(30));

        let result = dns.force_query_dns("user-service");
        assert!(result.is_ok());
        let addresses = result.unwrap();
        assert!(!addresses.is_empty());
    }

    #[test]
    fn test_config_service_discovery() {
        let mut config =
            ConfigServiceDiscovery::new("services.json".to_string(), Duration::from_secs(30));

        let result = config.force_load_services();
        assert!(result.is_ok());

        let services = config.get_services("user-service");
        assert!(!services.is_empty());
        assert_eq!(services[0].name, "user-service");
    }

    #[test]
    fn test_registry_service_discovery() {
        let mut registry = RegistryServiceDiscovery::new(
            "http://localhost:8500".to_string(),
            Duration::from_secs(30),
        );

        let instance = ServiceInstance::new(
            "test-1".to_string(),
            "test-service".to_string(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            HashMap::new(),
        );

        let result = registry.register_service(instance);
        assert!(result.is_ok());

        let services = registry.get_services("test-service");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "test-1");
    }

    #[test]
    fn test_service_discovery_manager() {
        let config = ServiceDiscoveryConfig {
            strategy: DiscoveryStrategy::Config {
                config_path: "services.json".to_string(),
                reload_interval: Duration::from_secs(30),
            },
            service_ttl: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            max_retries: 3,
            timeout: Duration::from_secs(5),
        };

        let mut manager = ServiceDiscoveryManager::new(config);

        let result = manager.discover_services("user-service");
        assert!(result.is_ok());
        let instances = result.unwrap();
        assert!(!instances.is_empty());
    }

    #[test]
    fn test_health_checker() {
        let mut checker = HealthChecker::new(Duration::from_secs(1));

        let mut instances = vec![
            ServiceInstance::new(
                "test-1".to_string(),
                "user-service".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                HashMap::new(),
            ),
            ServiceInstance::new(
                "test-2".to_string(),
                "unknown-service".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                HashMap::new(),
            ),
        ];

        checker.force_check_health(&mut instances);

        assert!(instances[0].is_healthy);
        assert!(!instances[1].is_healthy);
    }
}

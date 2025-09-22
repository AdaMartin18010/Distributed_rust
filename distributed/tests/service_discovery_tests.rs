//! 服务发现模块测试

use c20_distributed::{
    ConfigServiceDiscovery, DiscoveryStrategy, DnsServiceDiscovery,
    RegistryServiceDiscovery, ServiceDiscoveryConfig, ServiceDiscoveryManager, ServiceInstance,
};
use c20_distributed::service_discovery::HealthChecker;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

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
fn test_service_instance_health_update() {
    let mut instance = ServiceInstance::new(
        "test-1".to_string(),
        "test-service".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        HashMap::new(),
    );

    assert!(instance.is_healthy);

    instance.update_health(false);
    assert!(!instance.is_healthy);

    instance.update_health(true);
    assert!(instance.is_healthy);
}

#[test]
fn test_service_instance_expiration() {
    let instance = ServiceInstance::new(
        "test-1".to_string(),
        "test-service".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        HashMap::new(),
    );

    // 新创建的实例不应该过期
    assert!(!instance.is_expired(Duration::from_secs(60)));

    // 等待一小段时间后检查
    std::thread::sleep(Duration::from_millis(10));
    assert!(!instance.is_expired(Duration::from_secs(60)));
}

#[test]
fn test_dns_service_discovery() {
    let mut dns = DnsServiceDiscovery::new("8.8.8.8".to_string(), Duration::from_secs(30));

    let result = dns.force_query_dns("user-service");
    assert!(result.is_ok());
    let addresses = result.unwrap();
    assert!(!addresses.is_empty());
    assert_eq!(addresses.len(), 2);

    // 测试不存在的服务
    let result = dns.force_query_dns("non-existent-service");
    assert!(result.is_ok());
    let addresses = result.unwrap();
    assert!(addresses.is_empty());
}

#[test]
fn test_dns_service_discovery_interval() {
    let mut dns = DnsServiceDiscovery::new("8.8.8.8".to_string(), Duration::from_secs(1));

    // 第一次查询应该成功
    let result1 = dns.query_dns("user-service");
    assert!(result1.is_ok());

    // 立即再次查询应该返回空（因为还在间隔期内）
    let result2 = dns.query_dns("user-service");
    assert!(result2.is_ok());
    let addresses = result2.unwrap();
    assert!(addresses.is_empty());
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
    assert_eq!(services[0].weight, 10);

    let services = config.get_services("order-service");
    assert!(!services.is_empty());
    assert_eq!(services[0].name, "order-service");
    assert_eq!(services[0].weight, 8);
}

#[test]
fn test_config_service_discovery_nonexistent() {
    let mut config =
        ConfigServiceDiscovery::new("services.json".to_string(), Duration::from_secs(30));

    let result = config.force_load_services();
    assert!(result.is_ok());

    let services = config.get_services("non-existent-service");
    assert!(services.is_empty());
}

#[test]
fn test_registry_service_discovery() {
    let mut registry =
        RegistryServiceDiscovery::new("http://localhost:8500".to_string(), Duration::from_secs(30));

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
fn test_registry_service_discovery_unregister() {
    let mut registry =
        RegistryServiceDiscovery::new("http://localhost:8500".to_string(), Duration::from_secs(30));

    let instance = ServiceInstance::new(
        "test-1".to_string(),
        "test-service".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        HashMap::new(),
    );

    registry.register_service(instance).unwrap();

    let services = registry.get_services("test-service");
    assert_eq!(services.len(), 1);

    let result = registry.unregister_service("test-service", "test-1");
    assert!(result.is_ok());

    let services = registry.get_services("test-service");
    assert!(services.is_empty());
}

#[test]
fn test_registry_service_discovery_heartbeat() {
    let mut registry =
        RegistryServiceDiscovery::new("http://localhost:8500".to_string(), Duration::from_secs(1));

    let mut instance = ServiceInstance::new(
        "test-1".to_string(),
        "test-service".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        HashMap::new(),
    );
    instance.update_health(false);

    registry.register_service(instance).unwrap();

    let services = registry.get_services("test-service");
    assert_eq!(services.len(), 1);
    assert!(!services[0].is_healthy);

    // 发送心跳
    let result = registry.force_send_heartbeat();
    assert!(result.is_ok());

    let services = registry.get_services("test-service");
    assert_eq!(services.len(), 1);
    assert!(services[0].is_healthy);
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

    // 初始状态
    assert!(instances[0].is_healthy);
    assert!(instances[1].is_healthy);

    checker.force_check_health(&mut instances);

    // 检查后状态
    assert!(instances[0].is_healthy);
    assert!(!instances[1].is_healthy);
}

#[test]
fn test_health_checker_interval() {
    let mut checker = HealthChecker::new(Duration::from_secs(1));

    let mut instances = vec![ServiceInstance::new(
        "test-1".to_string(),
        "user-service".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        HashMap::new(),
    )];

    // 第一次检查
    checker.check_health(&mut instances);
    assert!(instances[0].is_healthy);

    // 立即再次检查（应该不会执行，因为还在间隔期内）
    instances[0].update_health(false);
    checker.check_health(&mut instances);
    assert!(!instances[0].is_healthy); // 状态没有改变
}

#[test]
fn test_service_discovery_manager_config_strategy() {
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
    assert_eq!(instances[0].name, "user-service");
}

#[test]
fn test_service_discovery_manager_dns_strategy() {
    let config = ServiceDiscoveryConfig {
        strategy: DiscoveryStrategy::Dns {
            dns_server: "8.8.8.8".to_string(),
            query_interval: Duration::from_secs(30),
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
    assert_eq!(instances[0].name, "user-service");
}

#[test]
fn test_service_discovery_manager_registry_strategy() {
    let config = ServiceDiscoveryConfig {
        strategy: DiscoveryStrategy::Registry {
            registry_url: "http://localhost:8500".to_string(),
            heartbeat_interval: Duration::from_secs(30),
        },
        service_ttl: Duration::from_secs(300),
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(5),
    };

    let mut manager = ServiceDiscoveryManager::new(config);

    // 注册一个服务
    let instance = ServiceInstance::new(
        "test-1".to_string(),
        "test-service".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        HashMap::new(),
    );

    let result = manager.register_service(instance);
    assert!(result.is_ok());

    let result = manager.discover_services("test-service");
    assert!(result.is_ok());
    let instances = result.unwrap();
    assert!(!instances.is_empty());
    assert_eq!(instances[0].name, "test-service");
}

#[test]
fn test_service_discovery_manager_cache() {
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

    // 第一次发现
    let result1 = manager.discover_services("user-service");
    assert!(result1.is_ok());
    let instances1 = result1.unwrap();
    assert!(!instances1.is_empty());

    // 第二次发现应该从缓存返回
    let result2 = manager.discover_services("user-service");
    assert!(result2.is_ok());
    let instances2 = result2.unwrap();
    assert!(!instances2.is_empty());
    assert_eq!(instances1.len(), instances2.len());
}

#[test]
fn test_service_discovery_manager_unregister() {
    let config = ServiceDiscoveryConfig {
        strategy: DiscoveryStrategy::Registry {
            registry_url: "http://localhost:8500".to_string(),
            heartbeat_interval: Duration::from_secs(30),
        },
        service_ttl: Duration::from_secs(300),
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(5),
    };

    let mut manager = ServiceDiscoveryManager::new(config);

    // 注册服务
    let instance = ServiceInstance::new(
        "test-1".to_string(),
        "test-service".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        HashMap::new(),
    );

    manager.register_service(instance).unwrap();

    let result = manager.discover_services("test-service");
    assert!(result.is_ok());
    let instances = result.unwrap();
    assert_eq!(instances.len(), 1);

    // 注销服务
    let result = manager.unregister_service("test-service", "test-1");
    assert!(result.is_ok());

    let result = manager.discover_services("test-service");
    assert!(result.is_ok());
    let instances = result.unwrap();
    assert!(instances.is_empty());
}

#[test]
fn test_service_discovery_manager_get_all_services() {
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

    // 发现一些服务
    manager.discover_services("user-service").unwrap();
    manager.discover_services("order-service").unwrap();

    let all_services = manager.get_all_services();
    assert!(all_services.contains_key("user-service"));
    assert!(all_services.contains_key("order-service"));
    assert!(!all_services.get("user-service").unwrap().is_empty());
    assert!(!all_services.get("order-service").unwrap().is_empty());
}

#[test]
fn test_service_discovery_manager_cleanup_expired() {
    let config = ServiceDiscoveryConfig {
        strategy: DiscoveryStrategy::Config {
            config_path: "services.json".to_string(),
            reload_interval: Duration::from_secs(30),
        },
        service_ttl: Duration::from_millis(1), // 很短的TTL
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(5),
    };

    let mut manager = ServiceDiscoveryManager::new(config);

    // 发现服务
    manager.discover_services("user-service").unwrap();

    let all_services = manager.get_all_services();
    assert!(all_services.contains_key("user-service"));

    // 等待TTL过期
    std::thread::sleep(Duration::from_millis(10));

    // 清理过期服务
    manager.cleanup_expired_services();

    let all_services = manager.get_all_services();
    assert!(!all_services.contains_key("user-service"));
}

#[test]
fn test_discovery_strategy_serialization() {
    let dns_strategy = DiscoveryStrategy::Dns {
        dns_server: "8.8.8.8".to_string(),
        query_interval: Duration::from_secs(30),
    };

    let config_strategy = DiscoveryStrategy::Config {
        config_path: "services.json".to_string(),
        reload_interval: Duration::from_secs(30),
    };

    let registry_strategy = DiscoveryStrategy::Registry {
        registry_url: "http://localhost:8500".to_string(),
        heartbeat_interval: Duration::from_secs(30),
    };

    // 测试序列化
    let dns_json = serde_json::to_string(&dns_strategy);
    assert!(dns_json.is_ok());

    let config_json = serde_json::to_string(&config_strategy);
    assert!(config_json.is_ok());

    let registry_json = serde_json::to_string(&registry_strategy);
    assert!(registry_json.is_ok());

    // 测试反序列化
    let dns_deserialized: DiscoveryStrategy = serde_json::from_str(&dns_json.unwrap()).unwrap();
    assert_eq!(dns_deserialized, dns_strategy);

    let config_deserialized: DiscoveryStrategy =
        serde_json::from_str(&config_json.unwrap()).unwrap();
    assert_eq!(config_deserialized, config_strategy);

    let registry_deserialized: DiscoveryStrategy =
        serde_json::from_str(&registry_json.unwrap()).unwrap();
    assert_eq!(registry_deserialized, registry_strategy);
}

#[test]
fn test_service_discovery_config_default() {
    let config = ServiceDiscoveryConfig::default();

    match config.strategy {
        DiscoveryStrategy::Config {
            config_path,
            reload_interval,
        } => {
            assert_eq!(config_path, "services.json");
            assert_eq!(reload_interval, Duration::from_secs(30));
        }
        _ => panic!("Default strategy should be Config"),
    }

    assert_eq!(config.service_ttl, Duration::from_secs(300));
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.timeout, Duration::from_secs(5));
}

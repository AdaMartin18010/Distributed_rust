use distributed::{
    topology::ConsistentHashRing,
    replication::{LocalReplicator, Replicator},
    ConsistencyLevel,
    service_discovery::{ServiceDiscoveryManager, ServiceInstance, ServiceDiscoveryConfig, DiscoveryStrategy},
    security::{CircuitBreaker, CircuitConfig, TokenBucket},
};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use std::thread;

/// 模拟分布式服务
struct DistributedService {
    id: String,
    port: u16,
    request_count: Arc<AtomicUsize>,
    error_count: Arc<AtomicUsize>,
    response_time: Arc<AtomicUsize>, // 微秒
}

impl DistributedService {
    fn new(id: String, port: u16) -> Self {
        Self {
            id,
            port,
            request_count: Arc::new(AtomicUsize::new(0)),
            error_count: Arc::new(AtomicUsize::new(0)),
            response_time: Arc::new(AtomicUsize::new(1000)), // 默认 1ms
        }
    }
    
    fn handle_request(&self, request_id: &str) -> Result<String, String> {
        self.request_count.fetch_add(1, Ordering::SeqCst);
        
        // 模拟处理时间
        let delay = self.response_time.load(Ordering::SeqCst);
        thread::sleep(Duration::from_micros(delay as u64));
        
        // 模拟错误率
        if self.error_count.load(Ordering::SeqCst) > 0 {
            self.error_count.fetch_sub(1, Ordering::SeqCst);
            return Err(format!("Service {} error", self.id));
        }
        
        Ok(format!("Response from {} for request {}", self.id, request_id))
    }
    
    fn get_stats(&self) -> ServiceStats {
        ServiceStats {
            id: self.id.clone(),
            port: self.port,
            request_count: self.request_count.load(Ordering::SeqCst),
            error_count: self.error_count.load(Ordering::SeqCst),
            avg_response_time: self.response_time.load(Ordering::SeqCst),
        }
    }
    
    fn inject_error(&self) {
        self.error_count.fetch_add(5, Ordering::SeqCst);
    }
    
    fn set_response_time(&self, time_us: usize) {
        self.response_time.store(time_us, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
struct ServiceStats {
    id: String,
    #[allow(dead_code)]
    port: u16,
    request_count: usize,
    error_count: usize,
    avg_response_time: usize,
}

/// 分布式系统演示
struct DistributedSystemDemo {
    services: Vec<Arc<DistributedService>>,
    hash_ring: ConsistentHashRing,
    round_robin_balancer: distributed::RoundRobinBalancer,
    consistent_hash_balancer: distributed::ConsistentHashBalancer,
    circuit_breaker: CircuitBreaker,
    rate_limiter: TokenBucket,
    service_discovery: ServiceDiscoveryManager,
}

impl DistributedSystemDemo {
    fn new() -> Self {
        // 创建服务实例
        let services = vec![
            Arc::new(DistributedService::new("service-1".to_string(), 8001)),
            Arc::new(DistributedService::new("service-2".to_string(), 8002)),
            Arc::new(DistributedService::new("service-3".to_string(), 8003)),
            Arc::new(DistributedService::new("service-4".to_string(), 8004)),
        ];
        
        // 创建一致性哈希环
        let mut hash_ring = ConsistentHashRing::new(32);
        for service in &services {
            hash_ring.add_node(&service.id);
        }
        
        // 创建服务实例用于负载均衡器
        let service_instances: Vec<ServiceInstance> = services.iter().map(|service| {
            ServiceInstance::new(
                service.id.clone(),
                "user-service".to_string(),
                format!("127.0.0.1:{}", service.port).parse().unwrap(),
                HashMap::new(),
            )
        }).collect();

        // 创建负载均衡器
        let round_robin_balancer = distributed::RoundRobinBalancer::new(service_instances.clone());
        let consistent_hash_balancer = distributed::ConsistentHashBalancer::new(service_instances, 8);
        
        // 创建熔断器
        let circuit_config = CircuitConfig {
            error_threshold: 5,
            open_ms: 1000,
        };
        let circuit_breaker = CircuitBreaker::new(circuit_config);
        
        // 创建限流器
        let rate_limiter = TokenBucket::new(100, 10);
        
        // 创建服务发现
        let service_discovery_config = ServiceDiscoveryConfig {
            strategy: DiscoveryStrategy::Config {
                config_path: "services.json".to_string(),
                reload_interval: Duration::from_secs(30),
            },
            service_ttl: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(5),
            max_retries: 3,
            timeout: Duration::from_secs(5),
        };
        let mut service_discovery = ServiceDiscoveryManager::new(service_discovery_config);
        for service in &services {
            let instance = ServiceInstance::new(
                service.id.clone(),
                "user-service".to_string(),
                format!("127.0.0.1:{}", service.port).parse().unwrap(),
                HashMap::new(),
            );
            let _ = service_discovery.register_service(instance);
        }
        
        Self {
            services,
            hash_ring,
            round_robin_balancer,
            consistent_hash_balancer,
            circuit_breaker,
            rate_limiter,
            service_discovery,
        }
    }
    
    fn run_comprehensive_demo(&mut self) {
        println!("🚀 综合分布式系统演示开始");
        
        // 1. 数据复制演示
        self.demo_replication();
        
        // 2. 负载均衡演示
        self.demo_load_balancing();
        
        // 3. 服务发现演示
        self.demo_service_discovery();
        
        // 4. 熔断器演示
        self.demo_circuit_breaker();
        
        // 5. 限流器演示
        self.demo_rate_limiting();
        
        // 6. 一致性哈希演示
        self.demo_consistent_hashing();
        
        // 7. 故障注入演示
        self.demo_fault_injection();
        
        // 8. 性能统计
        self.print_performance_stats();
        
        println!("\n✅ 综合分布式系统演示完成！");
    }
    
    fn demo_replication(&self) {
        println!("\n📊 1. 数据复制演示");
        
        let nodes: Vec<String> = self.services.iter()
            .map(|s| s.id.clone())
            .collect();
        let mut replicator: LocalReplicator<String> = LocalReplicator::new(self.hash_ring.clone(), nodes);
        
        // 测试不同一致性级别
        let data = vec![100u64, 200u64, 300u64];
        for (i, value) in data.iter().enumerate() {
            let start = Instant::now();
            let result = replicator.replicate(*value, ConsistencyLevel::Quorum);
            let duration = start.elapsed();
            println!("  📝 复制数据 {}: {:?} - 耗时: {:?}", i + 1, result, duration);
        }
    }
    
    fn demo_load_balancing(&mut self) {
        println!("\n⚖️  2. 负载均衡演示");
        
        // 轮询负载均衡
        println!("  🔄 轮询负载均衡:");
        for i in 0..5 {
            let service = self.round_robin_balancer.select_server().unwrap();
            let _request_id = format!("rr-req-{}", i);
            
            println!("    ✅ 请求 {} -> {}: 处理成功", i, service.id);
        }
        
        // 一致性哈希负载均衡
        println!("  🎯 一致性哈希负载均衡:");
        for i in 0..5 {
            let request_id = format!("ch-req-{}", i);
            let service = self.consistent_hash_balancer.select_server(&request_id).unwrap();
            
            println!("    ✅ 请求 {} -> {}: 处理成功", i, service.id);
        }
    }
    
    fn demo_service_discovery(&self) {
        println!("\n🔍 3. 服务发现演示");
        
        let all_services = self.service_discovery.get_all_services();
        let instances = all_services.get("user-service").cloned().unwrap_or_default();
        println!("  📋 发现的服务实例:");
        for instance in instances {
            println!("    🏷️  {}: {}", instance.id, instance.address);
        }
        
        // 模拟服务健康检查
        println!("  🏥 服务健康检查:");
        for service in &self.services {
            let stats = service.get_stats();
            println!("    {}: {} 请求, {} 错误", stats.id, stats.request_count, stats.error_count);
        }
    }
    
    fn demo_circuit_breaker(&mut self) {
        println!("\n🔌 4. 熔断器演示");
        
        // 正常请求
        println!("  🟢 正常请求阶段:");
        for i in 0..3 {
            if self.circuit_breaker.allow_request() {
                match self.services[0].handle_request(&format!("normal-{}", i)) {
                    Ok(result) => {
                        self.circuit_breaker.on_result(true);
                        println!("    ✅ 请求 {}: {:?}", i, result);
                    },
                    Err(e) => {
                        self.circuit_breaker.on_result(false);
                        println!("    ❌ 请求 {}: {:?}", i, e);
                    }
                }
            } else {
                println!("    🚫 请求 {}: 熔断器阻止", i);
            }
        }
        
        // 注入错误触发熔断
        println!("  🔴 错误注入阶段:");
        self.services[0].inject_error();
        
        for i in 0..8 {
            if self.circuit_breaker.allow_request() {
                match self.services[0].handle_request(&format!("error-{}", i)) {
                    Ok(result) => {
                        self.circuit_breaker.on_result(true);
                        println!("    ✅ 请求 {}: {:?}", i, result);
                    },
                    Err(e) => {
                        self.circuit_breaker.on_result(false);
                        println!("    ❌ 请求 {}: {:?}", i, e);
                    }
                }
            } else {
                println!("    🚫 请求 {}: 熔断器阻止", i);
            }
        }
        
        // 熔断器状态
        println!("  📊 熔断器状态: {:?}", self.circuit_breaker.state());
    }
    
    fn demo_rate_limiting(&mut self) {
        println!("\n🚦 5. 限流器演示");
        
        println!("  🎯 限流测试 (100 req/sec):");
        let mut success_count = 0;
        let mut rate_limited_count = 0;
        
        for i in 0..120 {
            if self.rate_limiter.allow() {
                success_count += 1;
                if i % 20 == 0 {
                    println!("    ✅ 请求 {}: 允许", i);
                }
            } else {
                rate_limited_count += 1;
                if i % 20 == 0 {
                    println!("    🚫 请求 {}: 限流", i);
                }
            }
            
            // 模拟时间流逝
            thread::sleep(Duration::from_millis(10));
        }
        
        println!("  📊 限流统计: 成功 {}, 限流 {}", success_count, rate_limited_count);
    }
    
    fn demo_consistent_hashing(&self) {
        println!("\n🎯 6. 一致性哈希演示");
        
        let keys = vec!["user:123", "user:456", "user:789", "user:101", "user:202"];
        
        for key in &keys {
            let node = self.hash_ring.route(key);
            println!("  🔑 键 '{}' -> 节点 '{:?}'", key, node);
        }
        
        // 模拟节点变化
        println!("  🔄 模拟节点变化...");
        let mut new_ring = self.hash_ring.clone();
        new_ring.remove_node("service-1");
        
        for key in &keys {
            let old_node = self.hash_ring.route(key);
            let new_node = new_ring.route(key);
            if old_node != new_node {
                println!("  🔄 键 '{}': {:?} -> {:?} (迁移)", key, old_node, new_node);
            }
        }
    }
    
    fn demo_fault_injection(&self) {
        println!("\n💥 7. 故障注入演示");
        
        // 注入延迟
        println!("  ⏰ 延迟注入:");
        self.services[1].set_response_time(5000); // 5ms
        
        let start = Instant::now();
        match self.services[1].handle_request("delay-test") {
            Ok(result) => {
                let duration = start.elapsed();
                println!("    ⏱️  延迟请求完成: {} - 耗时: {:?}", result, duration);
            }
            Err(e) => println!("    ❌ 延迟请求失败: {}", e),
        }
        
        // 注入错误
        println!("  🚨 错误注入:");
        self.services[2].inject_error();
        
        for i in 0..3 {
            match self.services[2].handle_request(&format!("error-test-{}", i)) {
                Ok(result) => println!("    ✅ 错误测试 {}: {}", i, result),
                Err(e) => println!("    ❌ 错误测试 {}: {}", i, e),
            }
        }
    }
    
    fn print_performance_stats(&self) {
        println!("\n📈 8. 性能统计");
        
        let mut total_requests = 0;
        let mut total_errors = 0;
        let mut total_response_time = 0;
        
        println!("  📊 服务统计:");
        for service in &self.services {
            let stats = service.get_stats();
            total_requests += stats.request_count;
            total_errors += stats.error_count;
            total_response_time += stats.avg_response_time;
            
            let error_rate = if stats.request_count > 0 {
                (stats.error_count as f64 / stats.request_count as f64) * 100.0
            } else {
                0.0
            };
            
            println!("    🏷️  {}: {} 请求, {} 错误 ({:.1}%), 平均响应时间 {}μs", 
                stats.id, stats.request_count, stats.error_count, error_rate, stats.avg_response_time);
        }
        
        let overall_error_rate = if total_requests > 0 {
            (total_errors as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        println!("  📊 总体统计:");
        println!("    🎯 总请求数: {}", total_requests);
        println!("    ❌ 总错误数: {}", total_errors);
        println!("    📊 错误率: {:.1}%", overall_error_rate);
        println!("    ⏱️  平均响应时间: {}μs", total_response_time / self.services.len());
        
        println!("\n📚 学习要点:");
        println!("  🔄 数据复制: 强一致性 vs 最终一致性");
        println!("  ⚖️  负载均衡: 请求分发和故障转移");
        println!("  🔍 服务发现: 动态服务注册和发现");
        println!("  🔌 熔断器: 故障隔离和快速失败");
        println!("  🚦 限流器: 流量控制和保护");
        println!("  🎯 一致性哈希: 数据分布和节点变化");
        println!("  💥 故障注入: 系统韧性测试");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut demo = DistributedSystemDemo::new();
    demo.run_comprehensive_demo();
    Ok(())
}

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::hint::black_box;
use distributed::{
    topology::ConsistentHashRing,
    replication::{LocalReplicator, Replicator},
    ConsistencyLevel,
    CircuitBreaker, CircuitConfig,
    RateLimitConfig, TokenBucket,
};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
// use std::collections::HashMap; // 未使用

/// 模拟服务节点 (用于基准测试)
#[allow(dead_code)]
struct MockService {
    id: String,
    request_count: Arc<AtomicUsize>,
}

#[allow(dead_code)]
impl MockService {
    fn new(id: String) -> Self {
        Self {
            id,
            request_count: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    fn handle_request(&self, _request_id: &str) -> Result<String, String> {
        self.request_count.fetch_add(1, Ordering::SeqCst);
        Ok(format!("Response from {}", self.id))
    }
}

/// 性能基准测试套件
struct PerformanceBenchmarks;

impl PerformanceBenchmarks {
    fn bench_consistent_hash_ring(c: &mut Criterion) {
        let mut group = c.benchmark_group("consistent_hash_ring");
        
        // 测试不同节点数量
        for node_count in [3, 5, 10, 20, 50] {
            let mut ring = ConsistentHashRing::new(64);
            for i in 0..node_count {
                ring.add_node(&format!("node-{}", i));
            }
            
            group.throughput(Throughput::Elements(1000));
            group.bench_with_input(
                BenchmarkId::new("lookup", node_count),
                &ring,
                |b, ring| {
                    b.iter(|| {
                        for i in 0..1000 {
                            black_box(ring.route(&format!("key-{}", i)));
                        }
                    });
                },
            );
        }
        
        group.finish();
    }
    
    fn bench_replication_performance(c: &mut Criterion) {
        let mut group = c.benchmark_group("replication");
        
        // 测试不同一致性级别
        let consistency_levels = vec![
            ("strong", ConsistencyLevel::Strong),
            ("quorum", ConsistencyLevel::Quorum),
            ("eventual", ConsistencyLevel::Eventual),
        ];
        
        for (name, level) in consistency_levels {
            let mut ring = ConsistentHashRing::new(16);
            let nodes = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
            for node in &nodes {
                ring.add_node(node);
            }
            
            let mut replicator: LocalReplicator<String> = LocalReplicator::new(ring, nodes);
            
            group.bench_function(name, |b| {
                b.iter(|| {
                    for i in 0..100 {
                        let _ = black_box(replicator.replicate(i as u64, level.clone()));
                    }
                });
            });
        }
        
        group.finish();
    }
    
    fn bench_load_balancing(c: &mut Criterion) {
        let mut group = c.benchmark_group("load_balancing");
        
        // 创建ServiceInstance用于负载均衡器
        let services = vec![
            distributed::ServiceInstance::new(
                "service-1".to_string(),
                "test-service".to_string(),
                "127.0.0.1:8001".parse().unwrap(),
                std::collections::HashMap::new(),
            ),
            distributed::ServiceInstance::new(
                "service-2".to_string(),
                "test-service".to_string(),
                "127.0.0.1:8002".parse().unwrap(),
                std::collections::HashMap::new(),
            ),
            distributed::ServiceInstance::new(
                "service-3".to_string(),
                "test-service".to_string(),
                "127.0.0.1:8003".parse().unwrap(),
                std::collections::HashMap::new(),
            ),
            distributed::ServiceInstance::new(
                "service-4".to_string(),
                "test-service".to_string(),
                "127.0.0.1:8004".parse().unwrap(),
                std::collections::HashMap::new(),
            ),
        ];
        
        // 测试轮询负载均衡
        let mut round_robin = distributed::RoundRobinBalancer::new(services.clone());
        group.bench_function("round_robin", |b| {
            b.iter(|| {
                for _i in 0..1000 {
                    if let Some(_service) = round_robin.select_server() {
                        black_box(_service.id.as_str());
                    }
                }
            });
        });
        
        // 测试一致性哈希负载均衡
        let consistent_hash = distributed::ConsistentHashBalancer::new(services.clone(), 8);
        
        group.bench_function("consistent_hash", |b| {
            b.iter(|| {
                for i in 0..1000 {
                    if let Some(_service) = consistent_hash.select_server(&format!("req-{}", i)) {
                        black_box(_service.id.as_str());
                    }
                }
            });
        });
        
        group.finish();
    }
    
    fn bench_circuit_breaker(c: &mut Criterion) {
        let mut group = c.benchmark_group("circuit_breaker");
        
        let circuit_config = CircuitConfig {
            error_threshold: 5,
            open_ms: 1000,
        };
        
        // 测试正常请求
        group.bench_function("normal_requests", |b| {
            let mut breaker = CircuitBreaker::new(circuit_config.clone());
            b.iter(|| {
                for _i in 0..100 {
                    let allowed = breaker.allow_request();
                    if allowed {
                        breaker.on_result(true);
                    }
                    black_box(allowed);
                }
            });
        });
        
        // 测试熔断状态
        group.bench_function("circuit_open", |b| {
            let mut breaker = CircuitBreaker::new(circuit_config.clone());
            
            // 触发熔断
            for _ in 0..10 {
                breaker.on_result(false);
            }
            
            b.iter(|| {
                for _i in 0..100 {
                    let allowed = breaker.allow_request();
                    black_box(allowed);
                }
            });
        });
        
        group.finish();
    }
    
    fn bench_rate_limiting(c: &mut Criterion) {
        let mut group = c.benchmark_group("rate_limiting");
        
        let rate_config = RateLimitConfig {
            capacity: 1000,
            refill_per_sec: 1000,
        };
        
        group.bench_function("token_bucket", |b| {
            let mut limiter = TokenBucket::new(rate_config.capacity, rate_config.refill_per_sec);
            b.iter(|| {
                for _ in 0..1000 {
                    black_box(limiter.allow());
                }
            });
        });
        
        group.finish();
    }
    
    fn bench_hash_ring_scaling(c: &mut Criterion) {
        let mut group = c.benchmark_group("hash_ring_scaling");
        
        // 测试不同环大小对性能的影响
        for ring_size in [16, 64, 256, 1024] {
            let mut ring = ConsistentHashRing::new(ring_size);
            for i in 0..10 {
                ring.add_node(&format!("node-{}", i));
            }
            
            group.bench_with_input(
                BenchmarkId::new("lookup", ring_size),
                &ring,
                |b, ring| {
                    b.iter(|| {
                        for i in 0..1000 {
                            black_box(ring.route(&format!("key-{}", i)));
                        }
                    });
                },
            );
        }
        
        group.finish();
    }
    
    fn bench_memory_usage(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory_usage");
        
        // 测试内存使用情况
        group.bench_function("hash_ring_memory", |b| {
            b.iter(|| {
                let mut ring = ConsistentHashRing::new(1024);
                for i in 0..100 {
                    ring.add_node(&format!("node-{}", i));
                }
                black_box(ring);
            });
        });
        
        group.bench_function("replicator_memory", |b| {
            b.iter(|| {
                let mut ring = ConsistentHashRing::new(64);
                let nodes: Vec<String> = (0..50).map(|i| format!("node-{}", i)).collect();
                for node in &nodes {
                    ring.add_node(node);
                }
                let replicator: LocalReplicator<String> = LocalReplicator::new(ring, nodes);
                black_box(replicator);
            });
        });
        
        group.finish();
    }
    
    fn bench_concurrent_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("concurrent_operations");
        
        let mut ring = ConsistentHashRing::new(64);
        let nodes: Vec<String> = (0..10).map(|i| format!("node-{}", i)).collect();
        for node in &nodes {
            ring.add_node(node);
        }
        
        group.bench_function("concurrent_lookups", |b| {
            b.iter(|| {
                let handles: Vec<_> = (0..10).map(|thread_id| {
                    let ring_clone = ring.clone();
                    std::thread::spawn(move || {
                        for i in 0..100 {
                            black_box(ring_clone.route(&format!("key-{}-{}", thread_id, i)));
                        }
                    })
                }).collect();
                
                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
        
        group.finish();
    }
    
    fn run_all_benchmarks(c: &mut Criterion) {
        Self::bench_consistent_hash_ring(c);
        Self::bench_replication_performance(c);
        Self::bench_load_balancing(c);
        Self::bench_circuit_breaker(c);
        Self::bench_rate_limiting(c);
        Self::bench_hash_ring_scaling(c);
        Self::bench_memory_usage(c);
        Self::bench_concurrent_operations(c);
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(30))
        .warm_up_time(std::time::Duration::from_secs(5));
    targets = PerformanceBenchmarks::run_all_benchmarks
);

criterion_main!(benches);

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use c20_distributed::{
    AclRule, Action, ChaosConfig, ChaosInjector, CircuitBreaker, CircuitConfig, ConfigManager,
    DiscoveryStrategy, FileSource, Governance, InMemorySource, LoadBalancerManager,
    LoadBalancingStrategy, Principal, Resource, ServiceDiscoveryConfig, ServiceDiscoveryManager,
    TokenBucket,
};

// 示例目标：演示“服务发现 + 负载均衡 + 动态配置联动”
// - 服务发现：使用 ConfigServiceDiscovery 的模拟数据源
// - 负载均衡：初始 WeightedRoundRobin，随后通过 ConfigManager 改为 RoundRobin
// - 动态配置：通过 InMemorySource 覆写配置，触发订阅回调重建 LB 与服务列表

fn main() {
    // 可选：初始化 tracing（需启用 feature `observability`）
    #[cfg(feature = "observability")]
    {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
        tracing::info!("observability enabled");
    }
    // 1) 准备动态配置：内存源 + （可选）文件源
    let mut cfg_mgr = ConfigManager::new();
    let mut mem = InMemorySource::new("mem");
    // 初始策略：WeightedRoundRobin
    mem.set("lb.strategy", "weighted_rr");
    // 可选：外部文件覆盖（存在则生效），便于演示热更新
    // 例：写一个 services.json 与 app.json 放在工作目录
    cfg_mgr.add_source(FileSource::new("app.json"));
    cfg_mgr.add_source(mem);

    // 2) 初始化服务发现：使用 Config 策略（内部自带模拟数据）
    let sd_cfg = ServiceDiscoveryConfig {
        strategy: DiscoveryStrategy::Config {
            config_path: "services.json".to_string(),
            reload_interval: Duration::from_millis(200),
        },
        service_ttl: Duration::from_secs(60),
        health_check_interval: Duration::from_millis(200),
        max_retries: 3,
        timeout: Duration::from_secs(1),
    };
    let mut sd = ServiceDiscoveryManager::new(sd_cfg);

    // 3) 首次发现目标服务实例
    let service_name = "user-service";
    let mut instances = sd
        .discover_services(service_name)
        .expect("discover services");
    assert!(!instances.is_empty(), "should have mock instances");

    // 4) 构建负载均衡器，根据配置选择策略
    let mut lb = {
        let strategy = resolve_lb_strategy(cfg_mgr.get_string("lb.strategy").as_deref());
        LoadBalancerManager::new(strategy, instances.clone())
    };

    // 5) 安全治理与混沌注入
    let mut gov0 = Governance::new();
    gov0.acl.replace_rules(vec![AclRule {
        principal: Principal::Service("client".to_string()),
        resource: Resource("user-service".to_string()),
        action: Action::Read,
        allow: true,
    }]);
    gov0.limiters
        .insert("client".to_string(), TokenBucket::new(50, 50));
    gov0.breakers.insert(
        "user-service".to_string(),
        CircuitBreaker::new(CircuitConfig {
            error_threshold: 5,
            open_ms: 1000,
        }),
    );

    let chaos = Arc::new(Mutex::new(ChaosInjector::new(ChaosConfig::default())));
    let gov = Arc::new(Mutex::new(gov0));

    // 6) 订阅动态配置：当策略或权重等发生变化时，重建/更新 LB、治理与混沌
    let lb_updater = Arc::new(Mutex::new(Updater { sd }));
    let lb_updater_clone = lb_updater.clone();
    let chaos_clone = chaos.clone();
    let gov_clone = gov.clone();
    cfg_mgr.subscribe(move |snap| {
        let strategy_key = snap.values.get("lb.strategy").and_then(|v| match v {
            c20_distributed::ConfigValue::String(s) => Some(s.as_str()),
            _ => None,
        });
        let new_strategy = resolve_lb_strategy(strategy_key);

        if let Ok(mut u) = lb_updater_clone.lock() {
            // 重新发现（模拟配置 reload）并更新 LB 的服务器列表
            if let Ok(new_instances) = u.sd.discover_services("user-service") {
                // 这里不直接持有 lb（主线程持有），仅打印提示；主线程会在下一个 tick 调用刷新
                println!(
                    "[config] lb.strategy = {:?}, instances = {}",
                    new_strategy,
                    new_instances.len()
                );
            }
        }

        // 将 app.json 的键映射为 Chaos/Governance 更新
        if let Ok(mut ch) = chaos_clone.lock() {
            let latency = snap
                .values
                .get("chaos.latency_ms")
                .and_then(as_u64)
                .unwrap_or(0);
            let jitter = snap
                .values
                .get("chaos.jitter_ms")
                .and_then(as_u64)
                .unwrap_or(0);
            let drop_rate = snap
                .values
                .get("chaos.drop_rate")
                .and_then(as_f64)
                .unwrap_or(0.0);
            let part_enabled = snap
                .values
                .get("chaos.partition_enabled")
                .and_then(as_bool)
                .unwrap_or(false);
            let peers = snap
                .values
                .get("chaos.partition_peers")
                .and_then(as_string_vec)
                .unwrap_or_default();
            let cfg = ChaosConfig {
                latency_ms: latency,
                jitter_ms: jitter,
                drop_rate,
                partition_enabled: part_enabled,
                partition_peers: peers.into_iter().collect(),
            };
            ch.update(cfg);
        }

        if let Ok(mut g) = gov_clone.lock() {
            // 限流设置
            let cap = snap
                .values
                .get("rl.client.capacity")
                .and_then(as_u64)
                .unwrap_or(50);
            let refill = snap
                .values
                .get("rl.client.refill_per_sec")
                .and_then(as_u64)
                .unwrap_or(50);
            g.limiters
                .insert("client".to_string(), TokenBucket::new(cap, refill));

            // 熔断设置
            let th = snap
                .values
                .get("cb.user_service.error_threshold")
                .and_then(as_u64)
                .unwrap_or(5) as u32;
            let open_ms = snap
                .values
                .get("cb.user_service.open_ms")
                .and_then(as_u64)
                .unwrap_or(1000);
            g.breakers.insert(
                "user-service".to_string(),
                CircuitBreaker::new(CircuitConfig {
                    error_threshold: th,
                    open_ms,
                }),
            );
        }
    });

    // 7) 首次刷入配置（触发订阅打印）
    let _ = cfg_mgr.refresh();

    // 8) 模拟请求流量并在中途切换策略与权重；同时应用 ACL/限流/熔断/混沌
    for tick in 0..5 {
        // 周期性刷新服务发现（模拟配置 reload 生效）
        instances = lb_updater
            .lock()
            .unwrap()
            .sd
            .discover_services(service_name)
            .unwrap();
        lb.update_servers(instances.clone());

        // 从 LB 选择实例并“发起请求”
        for req in 0..5 {
            // 限流
            if let Ok(mut g) = gov.lock() {
                if let Some(limiter) = g.limiters.get_mut("client") {
                    if !limiter.allow() {
                        println!("rate-limited: client");
                        continue;
                    }
                }
            }

            // ACL
            let allowed = gov
                .lock()
                .ok()
                .map(|g| {
                    g.acl.is_allowed(
                        &Principal::Service("client".to_string()),
                        &Resource(service_name.to_string()),
                        &Action::Read,
                    )
                })
                .unwrap_or(true);
            if !allowed {
                println!("acl-deny: client -> {}", service_name);
                continue;
            }

            // 熔断器开关
            if let Ok(mut g) = gov.lock() {
                if let Some(cb) = g.breakers.get_mut(service_name) {
                    if !cb.allow_request() {
                        println!("circuit-open: skip");
                        continue;
                    }
                }
            }

            let chosen = lb.select_server(None).expect("select server");
            // 混沌：延迟/丢包/分区
            if let Ok(ch) = chaos.lock() {
                ch.inject_latency();
            }
            let dropped = chaos.lock().ok().map(|c| c.should_drop()).unwrap_or(false);
            let partitioned = chaos
                .lock()
                .ok()
                .map(|c| c.is_partitioned_with(&chosen.name))
                .unwrap_or(false);
            let ok = !(dropped || partitioned);
            println!(
                "tick={tick} req={req} -> {} {} ok={ok}",
                chosen.name, chosen.address
            );
            if let Ok(mut g) = gov.lock() {
                if let Some(cb) = g.breakers.get_mut(service_name) {
                    cb.on_result(ok);
                }
            }
        }

        // 第2个 tick 时切换策略为 RoundRobin；第3个 tick 模拟文件/内存覆写再次变更
        if tick == 1 {
            cfg_mgr.set_override("lb.strategy", "round_robin");
            let strat = resolve_lb_strategy(cfg_mgr.get_string("lb.strategy").as_deref());
            lb.update_strategy(strat);
            println!("[override] switch to RoundRobin");
        }

        if tick == 2 {
            cfg_mgr.set_override("lb.strategy", "consistent_hash");
            let strat = resolve_lb_strategy(cfg_mgr.get_string("lb.strategy").as_deref());
            lb.update_strategy(strat);
            println!("[override] switch to ConsistentHash");
        }

        if tick == 3 {
            // 开启混沌：增加延迟与丢包，并与 user-service 分区
            if let Ok(mut ch) = chaos.lock() {
                ch.update(ChaosConfig {
                    latency_ms: 30,
                    drop_rate: 0.2,
                    jitter_ms: 10,
                    partition_enabled: true,
                    partition_peers: ["user-service".to_string()].into_iter().collect(),
                });
            }
            println!("[override] enable chaos: latency/drop/partition");
        }

        // 简单节拍
        thread::sleep(Duration::from_millis(200));
    }

    println!("done.");
}

struct Updater {
    sd: ServiceDiscoveryManager,
}

fn resolve_lb_strategy(s: Option<&str>) -> LoadBalancingStrategy {
    match s.unwrap_or("weighted_rr") {
        "round_robin" => LoadBalancingStrategy::RoundRobin,
        "least_conn" => LoadBalancingStrategy::LeastConnections,
        "random" => LoadBalancingStrategy::Random,
        "weighted_random" => LoadBalancingStrategy::WeightedRandom,
        "least_rt" => LoadBalancingStrategy::LeastResponseTime,
        "geo" => LoadBalancingStrategy::Geographic {
            client_location: "us-east-1".to_string(),
        },
        "consistent_hash" => LoadBalancingStrategy::ConsistentHash { virtual_nodes: 64 },
        _ => LoadBalancingStrategy::WeightedRoundRobin,
    }
}

fn as_u64(v: &c20_distributed::ConfigValue) -> Option<u64> {
    match v {
        c20_distributed::ConfigValue::Integer(i) => Some(*i as u64),
        c20_distributed::ConfigValue::Float(f) => Some(*f as u64),
        _ => None,
    }
}

fn as_f64(v: &c20_distributed::ConfigValue) -> Option<f64> {
    match v {
        c20_distributed::ConfigValue::Float(f) => Some(*f),
        c20_distributed::ConfigValue::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

fn as_bool(v: &c20_distributed::ConfigValue) -> Option<bool> {
    match v {
        c20_distributed::ConfigValue::Boolean(b) => Some(*b),
        _ => None,
    }
}

fn as_string_vec(v: &c20_distributed::ConfigValue) -> Option<Vec<String>> {
    match v {
        c20_distributed::ConfigValue::Array(arr) => {
            let mut out = Vec::new();
            for e in arr {
                if let c20_distributed::ConfigValue::String(s) = e {
                    out.push(s.clone());
                }
            }
            Some(out)
        }
        _ => None,
    }
}

use std::thread;
use std::time::Duration;

use c20_distributed::{
    ConfigManager, DiscoveryStrategy, FileSource, InMemorySource, LoadBalancerManager,
    LoadBalancingStrategy, ServiceDiscoveryConfig, ServiceDiscoveryManager,
};

fn main() {
    // 配置：内存 + 文件
    let mut cfg = ConfigManager::new();
    let mut mem = InMemorySource::new("mem");
    mem.set("lb.strategy", "round_robin");
    cfg.add_source(FileSource::new("app.json"));
    cfg.add_source(mem);

    // 服务发现（本地模拟数据源）
    let sd_cfg = ServiceDiscoveryConfig {
        strategy: DiscoveryStrategy::Config {
            config_path: "services.json".into(),
            reload_interval: Duration::from_millis(100),
        },
        service_ttl: Duration::from_secs(30),
        health_check_interval: Duration::from_millis(100),
        max_retries: 3,
        timeout: Duration::from_millis(500),
    };
    let mut sd = ServiceDiscoveryManager::new(sd_cfg);

    // 实例 + 负载均衡器
    let service = "user-service";
    let instances = sd.discover_services(service).expect("discover");
    let mut lb = LoadBalancerManager::new(
        resolve_lb_strategy(cfg.get_string("lb.strategy").as_deref()),
        instances.clone(),
    );

    // 订阅策略热更新（外部修改 app.json → 切换策略）
    cfg.subscribe(|snap| {
        let s = snap.values.get("lb.strategy").and_then(|v| match v {
            c20_distributed::ConfigValue::String(x) => Some(x.as_str()),
            _ => None,
        });
        println!("[lb] strategy -> {:?}", s);
    });
    let _ = cfg.refresh();

    // 演示多个 tick：选择实例并在中途切换策略
    for tick in 0..5 {
        let mut instances = sd.discover_services(service).unwrap();
        lb.update_servers(instances.split_off(0));

        for req in 0..4 {
            let chosen = lb.select_server(None).unwrap();
            println!(
                "tick={tick} req={req} -> {} {}",
                chosen.name, chosen.address
            );
        }

        if tick == 1 {
            lb.update_strategy(LoadBalancingStrategy::WeightedRoundRobin);
            println!("[switch] -> WeightedRoundRobin");
        }
        if tick == 2 {
            lb.update_strategy(LoadBalancingStrategy::Random);
            println!("[switch] -> Random");
        }
        if tick == 3 {
            lb.update_strategy(LoadBalancingStrategy::ConsistentHash { virtual_nodes: 64 });
            println!("[switch] -> ConsistentHash");
        }

        thread::sleep(Duration::from_millis(150));
    }

    println!("done.");
}

fn resolve_lb_strategy(s: Option<&str>) -> LoadBalancingStrategy {
    match s.unwrap_or("round_robin") {
        "round_robin" => LoadBalancingStrategy::RoundRobin,
        "weighted_rr" => LoadBalancingStrategy::WeightedRoundRobin,
        "least_conn" => LoadBalancingStrategy::LeastConnections,
        "random" => LoadBalancingStrategy::Random,
        "weighted_random" => LoadBalancingStrategy::WeightedRandom,
        "least_rt" => LoadBalancingStrategy::LeastResponseTime,
        "geo" => LoadBalancingStrategy::Geographic {
            client_location: "us-east-1".to_string(),
        },
        "consistent_hash" => LoadBalancingStrategy::ConsistentHash { virtual_nodes: 64 },
        _ => LoadBalancingStrategy::RoundRobin,
    }
}

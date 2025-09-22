use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use c20_distributed::{
    ChaosConfig, ChaosInjector, ConfigManager, ConfigValue, FileSource, InMemorySource,
};

fn main() {
    // 配置：内存 + 文件
    let mut cfg = ConfigManager::new();
    let mut mem = InMemorySource::new("mem");
    mem.set("chaos.latency_ms", 10u64);
    mem.set("chaos.drop_rate", 0.1f64);
    mem.set("chaos.jitter_ms", 5u64);
    mem.set("chaos.partition_enabled", false);
    cfg.add_source(FileSource::new("app.json"));
    cfg.add_source(mem);

    let chaos = Arc::new(Mutex::new(ChaosInjector::new(ChaosConfig::default())));
    let sub = chaos.clone();
    cfg.subscribe(move |snap| {
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
        let partition_enabled = snap
            .values
            .get("chaos.partition_enabled")
            .and_then(as_bool)
            .unwrap_or(false);
        if let Ok(mut ch) = sub.lock() {
            ch.update(ChaosConfig {
                latency_ms: latency,
                jitter_ms: jitter,
                drop_rate,
                partition_enabled,
                partition_peers: Default::default(),
            });
        }
        println!(
            "[chaos] latency={}ms jitter={}ms drop_rate={} partition={}",
            latency, jitter, drop_rate, partition_enabled
        );
    });

    let _ = cfg.refresh();

    for i in 0..20 {
        if let Ok(ch) = chaos.lock() {
            ch.inject_latency();
            if ch.should_drop() {
                println!("{}: dropped", i);
                continue;
            }
        }
        println!("{}: ok", i);
        thread::sleep(Duration::from_millis(50));
    }
}

fn as_u64(v: &ConfigValue) -> Option<u64> {
    match v {
        ConfigValue::Integer(i) => Some(*i as u64),
        ConfigValue::Float(f) => Some(*f as u64),
        _ => None,
    }
}
fn as_f64(v: &ConfigValue) -> Option<f64> {
    match v {
        ConfigValue::Float(f) => Some(*f),
        ConfigValue::Integer(i) => Some(*i as f64),
        _ => None,
    }
}
fn as_bool(v: &ConfigValue) -> Option<bool> {
    match v {
        ConfigValue::Boolean(b) => Some(*b),
        _ => None,
    }
}

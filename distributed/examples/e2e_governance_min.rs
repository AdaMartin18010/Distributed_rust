use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use c20_distributed::{
    AclRule, Action, CircuitBreaker, CircuitConfig, ConfigManager, ConfigValue, FileSource,
    Governance, InMemorySource, Principal, Resource, TokenBucket,
};

fn main() {
    // 配置：内存 + 文件
    let mut cfg = ConfigManager::new();
    let mut mem = InMemorySource::new("mem");
    mem.set("rl.client.capacity", 50u64);
    mem.set("rl.client.refill_per_sec", 50u64);
    mem.set("cb.user_service.error_threshold", 5u64);
    mem.set("cb.user_service.open_ms", 1000u64);
    cfg.add_source(FileSource::new("app.json"));
    cfg.add_source(mem);

    // 治理对象，支持热更新
    let gov = Arc::new(Mutex::new(Governance::new()));
    {
        let mut g = gov.lock().unwrap();
        g.acl.replace_rules(vec![AclRule {
            principal: Principal::Service("client".into()),
            resource: Resource("user-service".into()),
            action: Action::Read,
            allow: true,
        }]);
        g.limiters.insert("client".into(), TokenBucket::new(50, 50));
        g.breakers.insert(
            "user-service".into(),
            CircuitBreaker::new(CircuitConfig {
                error_threshold: 5,
                open_ms: 1000,
            }),
        );
    }

    // 订阅配置热更新 → 应用到限流/熔断
    let gov_sub = gov.clone();
    cfg.subscribe(move |snap| {
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
        if let Ok(mut g) = gov_sub.lock() {
            g.limiters
                .insert("client".into(), TokenBucket::new(cap, refill));
            g.breakers.insert(
                "user-service".into(),
                CircuitBreaker::new(CircuitConfig {
                    error_threshold: th,
                    open_ms,
                }),
            );
            // 可选：从配置载入 ACL 规则数组
            if let Some(rules) = snap.values.get("acl.rules").and_then(as_acl_rules) {
                g.acl.replace_rules(rules);
            }
        }
        println!(
            "[governance] rl(cap={},refill={}), cb(th={},open_ms={})",
            cap, refill, th, open_ms
        );
    });

    let _ = cfg.refresh();

    // 简单演示：发起若干请求，展示限流与熔断状态
    for i in 0..30 {
        // 限流
        let allowed = {
            let mut g = gov.lock().unwrap();
            match g.limiters.get_mut("client") {
                Some(b) => b.allow(),
                None => true,
            }
        };
        if !allowed {
            println!("{}: rate-limited", i);
            thread::sleep(Duration::from_millis(50));
            continue;
        }

        // 模拟一次结果并喂给熔断器（这里让每5次失败一次）
        let ok = i % 5 != 0;
        {
            let mut g = gov.lock().unwrap();
            if let Some(cb) = g.breakers.get_mut("user-service") {
                if !cb.allow_request() {
                    println!("{}: circuit-open", i);
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                cb.on_result(ok);
            }
        }
        println!("{}: result={}", i, ok);
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

fn as_string(v: &ConfigValue) -> Option<String> {
    match v {
        ConfigValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn as_bool(v: &ConfigValue) -> Option<bool> {
    match v {
        ConfigValue::Boolean(b) => Some(*b),
        _ => None,
    }
}

fn as_acl_rules(v: &ConfigValue) -> Option<Vec<AclRule>> {
    use c20_distributed::{Action, Principal, Resource};
    match v {
        ConfigValue::Array(arr) => {
            let mut out = Vec::new();
            for item in arr {
                if let ConfigValue::Object(m) = item {
                    let principal = match m.get("principal").and_then(as_string) {
                        Some(s) if s.starts_with("user:") => {
                            Principal::User(s.trim_start_matches("user:").to_string())
                        }
                        Some(s) if s.starts_with("service:") => {
                            Principal::Service(s.trim_start_matches("service:").to_string())
                        }
                        Some(s) if s.starts_with("role:") => {
                            Principal::Role(s.trim_start_matches("role:").to_string())
                        }
                        Some(s) => Principal::Service(s),
                        None => continue,
                    };
                    let resource = m.get("resource").and_then(as_string).map(Resource);
                    let action = match m.get("action").and_then(as_string).as_deref() {
                        Some("read") => Some(Action::Read),
                        Some("write") => Some(Action::Write),
                        Some("admin") => Some(Action::Admin),
                        _ => None,
                    };
                    let allow = m.get("allow").and_then(as_bool).unwrap_or(false);
                    if let (Some(res), Some(act)) = (resource, action) {
                        out.push(AclRule {
                            principal,
                            resource: res,
                            action: act,
                            allow,
                        });
                    }
                }
            }
            Some(out)
        }
        _ => None,
    }
}

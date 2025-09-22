//! 安全与治理模块
//!
//! 提供基于内存热更新的 ACL、审计日志、限流与熔断策略。

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};

// --- 访问控制（ACL） ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Principal {
    User(String),
    Service(String),
    Role(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Resource(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Action {
    Read,
    Write,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AclRule {
    pub principal: Principal,
    pub resource: Resource,
    pub action: Action,
    pub allow: bool,
}

#[derive(Debug, Default, Clone)]
pub struct AclManager {
    rules: Vec<AclRule>,
}

impl AclManager {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    pub fn replace_rules(&mut self, rs: Vec<AclRule>) {
        self.rules = rs;
    }
    pub fn is_allowed(&self, principal: &Principal, resource: &Resource, action: &Action) -> bool {
        for r in &self.rules {
            if &r.principal == principal && &r.resource == resource && &r.action == action {
                return r.allow;
            }
        }
        false
    }
}

// --- 审计日志 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub ts: SystemTime,
    pub principal: Principal,
    pub resource: Resource,
    pub action: Action,
    pub allowed: bool,
}

#[derive(Debug, Default)]
pub struct Auditor {
    events: VecDeque<AuditEvent>,
    max: usize,
}

impl Auditor {
    pub fn new(max: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max,
        }
    }
    pub fn record(&mut self, e: AuditEvent) {
        if self.events.len() >= self.max {
            self.events.pop_front();
        }
        self.events.push_back(e);
    }
    pub fn recent(&self, n: usize) -> Vec<AuditEvent> {
        self.events.iter().rev().take(n).cloned().collect()
    }
}

// --- 限流（令牌桶） ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub capacity: u64,
    pub refill_per_sec: u64,
}

#[derive(Debug, Clone)]
pub struct TokenBucket {
    cap: u64,
    refill: u64,
    tokens: u64,
    last: Instant,
}

impl TokenBucket {
    pub fn new(cap: u64, refill: u64) -> Self {
        Self {
            cap,
            refill,
            tokens: cap,
            last: Instant::now(),
        }
    }
    pub fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        let add = (elapsed * self.refill as f64) as u64;
        if add > 0 {
            self.tokens = (self.tokens + add).min(self.cap);
            self.last = now;
        }
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }
}

// --- 熔断器（半开） ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitConfig {
    pub error_threshold: u32,
    pub open_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    cfg: CircuitConfig,
    state: CircuitState,
    errors: u32,
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(cfg: CircuitConfig) -> Self {
        Self {
            cfg,
            state: CircuitState::Closed,
            errors: 0,
            opened_at: None,
        }
    }
    pub fn on_result(&mut self, ok: bool) {
        match self.state {
            CircuitState::Closed => {
                if ok {
                    self.errors = 0;
                } else {
                    self.errors += 1;
                    if self.errors >= self.cfg.error_threshold {
                        self.state = CircuitState::Open;
                        self.opened_at = Some(Instant::now());
                    }
                }
            }
            CircuitState::Open => {
                if let Some(t0) = self.opened_at
                    && t0.elapsed() >= Duration::from_millis(self.cfg.open_ms) {
                        self.state = CircuitState::HalfOpen;
                        self.errors = 0;
                    }
            }
            CircuitState::HalfOpen => {
                if ok {
                    self.state = CircuitState::Closed;
                    self.errors = 0;
                } else {
                    self.state = CircuitState::Open;
                    self.opened_at = Some(Instant::now());
                }
            }
        }
    }
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(t0) = self.opened_at {
                    if t0.elapsed() >= Duration::from_millis(self.cfg.open_ms) {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    pub fn state(&self) -> CircuitState {
        self.state
    }
}

// --- 汇总策略门面 ---

#[derive(Debug, Default)]
pub struct Governance {
    pub acl: AclManager,
    pub auditor: Auditor,
    pub limiters: HashMap<String, TokenBucket>,
    pub breakers: HashMap<String, CircuitBreaker>,
}

impl Governance {
    pub fn new() -> Self {
        Self {
            acl: AclManager::new(),
            auditor: Auditor::new(1024),
            limiters: HashMap::new(),
            breakers: HashMap::new(),
        }
    }
}

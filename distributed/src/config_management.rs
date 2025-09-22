//! 配置管理模块
//!
//! 提供分层配置（内存/文件/环境）、动态更新、订阅通知与快照导出能力。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// 配置值类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

impl From<&str> for ConfigValue {
    fn from(v: &str) -> Self {
        ConfigValue::String(v.to_string())
    }
}
impl From<String> for ConfigValue {
    fn from(v: String) -> Self {
        ConfigValue::String(v)
    }
}
impl From<i64> for ConfigValue {
    fn from(v: i64) -> Self {
        ConfigValue::Integer(v)
    }
}
impl From<u64> for ConfigValue {
    fn from(v: u64) -> Self {
        ConfigValue::Integer(v as i64)
    }
}
impl From<f64> for ConfigValue {
    fn from(v: f64) -> Self {
        ConfigValue::Float(v)
    }
}
impl From<bool> for ConfigValue {
    fn from(v: bool) -> Self {
        ConfigValue::Boolean(v)
    }
}

/// 配置源接口
pub trait ConfigSource {
    fn load(&mut self) -> Result<HashMap<String, ConfigValue>, String>;
    fn name(&self) -> &str;
}

/// 内存配置源（最高优先级，常用于动态覆写）
pub struct InMemorySource {
    name: String,
    kv: HashMap<String, ConfigValue>,
}

impl InMemorySource {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kv: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<ConfigValue>) {
        self.kv.insert(key.into(), value.into());
    }
    pub fn remove(&mut self, key: &str) {
        self.kv.remove(key);
    }
}

impl ConfigSource for InMemorySource {
    fn load(&mut self) -> Result<HashMap<String, ConfigValue>, String> {
        Ok(self.kv.clone())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// 文件配置源（JSON/YAML 支持 JSON，YAML 可后续扩展）
pub struct FileSource {
    name: String,
    path: PathBuf,
    last_loaded: Option<Instant>,
    min_reload_interval: Duration,
}

impl FileSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            name: "file".to_string(),
            path: path.into(),
            last_loaded: None,
            min_reload_interval: Duration::from_millis(200),
        }
    }

    pub fn with_min_reload_interval(mut self, interval: Duration) -> Self {
        self.min_reload_interval = interval;
        self
    }
}

impl ConfigSource for FileSource {
    fn load(&mut self) -> Result<HashMap<String, ConfigValue>, String> {
        if let Some(ts) = self.last_loaded
            && ts.elapsed() < self.min_reload_interval {
                return Ok(HashMap::new());
            }
        self.last_loaded = Some(Instant::now());

        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }

        // 优先JSON
        let json: Result<HashMap<String, ConfigValue>, _> = serde_json::from_str(&content);
        if let Ok(map) = json {
            return Ok(map);
        }

        // 简单的 key=value 行解析（备用）
        let mut map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                map.insert(
                    k.trim().to_string(),
                    ConfigValue::String(v.trim().to_string()),
                );
            }
        }
        Ok(map)
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// 环境变量配置源（指定前缀）
pub struct EnvSource {
    name: String,
    prefix: String,
}

impl EnvSource {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            name: "env".to_string(),
            prefix: prefix.into(),
        }
    }
}

impl ConfigSource for EnvSource {
    fn load(&mut self) -> Result<HashMap<String, ConfigValue>, String> {
        let mut map = HashMap::new();
        for (k, v) in std::env::vars() {
            if k.starts_with(&self.prefix) {
                let key = k[self.prefix.len()..]
                    .trim_matches('_')
                    .to_ascii_lowercase()
                    .replace("__", ".");
                map.insert(key, ConfigValue::String(v));
            }
        }
        Ok(map)
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// 配置快照
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigSnapshot {
    pub values: HashMap<String, ConfigValue>,
    pub version: u64,
}

/// 订阅者回调类型
type Subscriber = Box<dyn Fn(&ConfigSnapshot) + Send + Sync + 'static>;

/// 配置管理器
pub struct ConfigManager {
    sources: Vec<Box<dyn ConfigSource + Send + Sync>>,
    store: HashMap<String, ConfigValue>,
    version: u64,
    subscribers: Vec<Subscriber>,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            store: HashMap::new(),
            version: 0,
            subscribers: Vec::new(),
        }
    }

    pub fn add_source<S: ConfigSource + Send + Sync + 'static>(&mut self, source: S) {
        self.sources.push(Box::new(source));
    }

    /// 合并并刷新配置，后添加的源优先级更高（覆写）
    pub fn refresh(&mut self) -> Result<ConfigSnapshot, String> {
        let mut merged: HashMap<String, ConfigValue> = HashMap::new();
        for src in &mut self.sources {
            let kv = src.load()?;
            for (k, v) in kv {
                merged.insert(k, v);
            }
        }
        if merged != self.store {
            self.store = merged.clone();
            self.version += 1;
            let snap = self.snapshot();
            self.notify(&snap);
            return Ok(snap);
        }
        Ok(self.snapshot())
    }

    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.store.get(key)
    }
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.store.get(key).and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
    }
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.store.get(key).and_then(|v| match v {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        })
    }
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.store.get(key).and_then(|v| match v {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        })
    }
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.store.get(key).and_then(|v| match v {
            ConfigValue::Float(f) => Some(*f),
            _ => None,
        })
    }

    pub fn set_override(&mut self, key: impl Into<String>, value: impl Into<ConfigValue>) {
        self.store.insert(key.into(), value.into());
        self.version += 1;
        let snap = self.snapshot();
        self.notify(&snap);
    }

    pub fn subscribe<F>(&mut self, f: F)
    where
        F: Fn(&ConfigSnapshot) + Send + Sync + 'static,
    {
        self.subscribers.push(Box::new(f));
    }

    fn notify(&self, snap: &ConfigSnapshot) {
        for s in &self.subscribers {
            s(snap);
        }
    }

    pub fn snapshot(&self) -> ConfigSnapshot {
        ConfigSnapshot {
            values: self.store.clone(),
            version: self.version,
        }
    }

    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.snapshot()).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inmemory_source() {
        let mut src = InMemorySource::new("mem");
        src.set("a.b", 1_i64);
        let map = src.load().unwrap();
        assert_eq!(map.get("a.b"), Some(&ConfigValue::Integer(1)));
    }

    #[test]
    fn test_file_source_json_and_reload_interval() {
        let mut p = std::env::temp_dir();
        let unique = format!(
            "cfg_{}_{}.json",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        );
        p.push(unique);
        fs::write(&p, "{\"k1\":\"v1\"}").unwrap();
        let mut src = FileSource::new(&p).with_min_reload_interval(Duration::from_secs(1));
        let m1 = src.load().unwrap();
        assert_eq!(m1.get("k1"), Some(&ConfigValue::String("v1".to_string())));
        // within interval returns empty
        let m2 = src.load().unwrap();
        assert!(m2.is_empty());
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn test_env_source_prefix() {
        // 使用一个不太可能存在的前缀，验证不会崩溃且结果为空
        let mut src = EnvSource::new("__APP_TEST_PREFIX__");
        let map = src.load().unwrap();
        assert!(map.is_empty() || !map.contains_key("db.url"));
    }

    #[test]
    fn test_config_manager_merge_priority_and_subscribe() {
        let mut p = std::env::temp_dir();
        let unique = format!(
            "base_{}_{}.json",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        );
        p.push(unique);
        fs::write(&p, "{\"k\":\"base\", \"x\": 1}").unwrap();

        let mut mgr = ConfigManager::new();
        mgr.add_source(FileSource::new(&p));
        mgr.add_source(EnvSource::new("APP_"));
        let notified = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let notified_cloned = notified.clone();
        mgr.subscribe(move |snap| {
            let _ = &snap.version;
            notified_cloned.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let snap1 = mgr.refresh().unwrap();
        assert_eq!(
            snap1.values.get("k"),
            Some(&ConfigValue::String("base".to_string()))
        );
        assert!(notified.load(std::sync::atomic::Ordering::SeqCst) >= 1);

        mgr.set_override("k", "override");
        let snap2 = mgr.snapshot();
        assert_eq!(
            snap2.values.get("k"),
            Some(&ConfigValue::String("override".to_string()))
        );
        assert!(snap2.version >= snap1.version);
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn test_export_json() {
        let mut mgr = ConfigManager::new();
        mgr.set_override("feature.flag", true);
        let json = mgr.export_json().unwrap();
        assert!(json.contains("feature.flag"));
    }
}

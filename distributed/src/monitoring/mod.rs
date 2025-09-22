//! 分布式系统监控和指标收集模块
//! 
//! 提供性能监控、健康检查、指标收集等功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// 计数器 - 单调递增的数值
    Counter,
    /// 仪表盘 - 可增可减的数值
    Gauge,
    /// 直方图 - 数值分布统计
    Histogram,
    /// 摘要 - 分位数统计
    Summary,
}

/// 指标标签
pub type MetricLabels = HashMap<String, String>;

/// 指标值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(f64),
    Gauge(f64),
    Histogram(HistogramData),
    Summary(SummaryData),
}

/// 直方图数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramData {
    pub count: u64,
    pub sum: f64,
    pub buckets: Vec<HistogramBucket>,
}

/// 直方图桶
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// 摘要数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryData {
    pub count: u64,
    pub sum: f64,
    pub quantiles: Vec<Quantile>,
}

/// 分位数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    pub quantile: f64,
    pub value: f64,
}

/// 指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub labels: MetricLabels,
    pub value: MetricValue,
    pub timestamp: u64,
}

/// 指标收集器
#[allow(dead_code)]
pub struct MetricCollector {
    metrics: HashMap<String, Arc<dyn MetricImpl + Send + Sync>>,
    registry: Arc<MetricRegistry>,
}

/// 指标实现 trait
pub trait MetricImpl {
    fn get_metric(&self) -> Metric;
    fn get_type(&self) -> MetricType;
}

/// 计数器实现
#[derive(Debug)]
pub struct Counter {
    name: String,
    labels: MetricLabels,
    value: AtomicU64,
}

impl Counter {
    pub fn new(name: String, labels: MetricLabels) -> Self {
        Self {
            name,
            labels,
            value: AtomicU64::new(0),
        }
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl MetricImpl for Counter {
    fn get_metric(&self) -> Metric {
        Metric {
            name: self.name.clone(),
            metric_type: MetricType::Counter,
            labels: self.labels.clone(),
            value: MetricValue::Counter(self.get() as f64),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    fn get_type(&self) -> MetricType {
        MetricType::Counter
    }
}

/// 仪表盘实现
#[derive(Debug)]
pub struct Gauge {
    name: String,
    labels: MetricLabels,
    value: AtomicU64,
}

impl Gauge {
    pub fn new(name: String, labels: MetricLabels) -> Self {
        Self {
            name,
            labels,
            value: AtomicU64::new(0),
        }
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl MetricImpl for Gauge {
    fn get_metric(&self) -> Metric {
        Metric {
            name: self.name.clone(),
            metric_type: MetricType::Gauge,
            labels: self.labels.clone(),
            value: MetricValue::Gauge(self.get() as f64),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    fn get_type(&self) -> MetricType {
        MetricType::Gauge
    }
}

/// 直方图实现
pub struct Histogram {
    name: String,
    labels: MetricLabels,
    buckets: Vec<f64>,
    counts: Vec<AtomicU64>,
    sum: AtomicU64,
    count: AtomicU64,
}

impl std::fmt::Debug for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Histogram")
            .field("name", &self.name)
            .field("labels", &self.labels)
            .field("buckets", &self.buckets)
            .field("count", &self.count.load(Ordering::Relaxed))
            .field("sum", &self.sum.load(Ordering::Relaxed))
            .finish()
    }
}

impl Histogram {
    pub fn new(name: String, labels: MetricLabels, buckets: Vec<f64>) -> Self {
        let mut counts = Vec::with_capacity(buckets.len());
        for _ in 0..buckets.len() {
            counts.push(AtomicU64::new(0));
        }
        Self {
            name,
            labels,
            buckets,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    pub fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        // 将浮点数转换为整数存储（乘以1000保持精度）
        let scaled_value = (value * 1000.0) as u64;
        self.sum.fetch_add(scaled_value, Ordering::Relaxed);

        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }

    pub fn get_data(&self) -> HistogramData {
        let buckets: Vec<HistogramBucket> = self
            .buckets
            .iter()
            .zip(self.counts.iter())
            .map(|(&upper_bound, count)| HistogramBucket {
                upper_bound,
                count: count.load(Ordering::Relaxed),
            })
            .collect();

        HistogramData {
            count: self.count.load(Ordering::Relaxed),
            sum: (self.sum.load(Ordering::Relaxed) as f64) / 1000.0,
            buckets,
        }
    }
}

impl MetricImpl for Histogram {
    fn get_metric(&self) -> Metric {
        Metric {
            name: self.name.clone(),
            metric_type: MetricType::Histogram,
            labels: self.labels.clone(),
            value: MetricValue::Histogram(self.get_data()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    fn get_type(&self) -> MetricType {
        MetricType::Histogram
    }
}

/// 指标注册表
pub struct MetricRegistry {
    metrics: HashMap<String, Arc<dyn MetricImpl + Send + Sync>>,
}

impl MetricRegistry {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn register_counter(&mut self, name: String, labels: MetricLabels) -> Arc<Counter> {
        let counter = Arc::new(Counter::new(name.clone(), labels));
        self.metrics.insert(name, counter.clone());
        counter
    }

    pub fn register_gauge(&mut self, name: String, labels: MetricLabels) -> Arc<Gauge> {
        let gauge = Arc::new(Gauge::new(name.clone(), labels));
        self.metrics.insert(name, gauge.clone());
        gauge
    }

    pub fn register_histogram(
        &mut self,
        name: String,
        labels: MetricLabels,
        buckets: Vec<f64>,
    ) -> Arc<Histogram> {
        let histogram = Arc::new(Histogram::new(name.clone(), labels, buckets));
        self.metrics.insert(name, histogram.clone());
        histogram
    }

    pub fn get_all_metrics(&self) -> Vec<Metric> {
        self.metrics
            .values()
            .map(|metric| metric.get_metric())
            .collect()
    }

    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        self.metrics.get(name).map(|metric| metric.get_metric())
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            registry: Arc::new(MetricRegistry::new()),
        }
    }

    pub fn counter(&mut self, name: &str, labels: MetricLabels) -> Arc<Counter> {
        let registry = Arc::get_mut(&mut self.registry).unwrap();
        registry.register_counter(name.to_string(), labels)
    }

    pub fn gauge(&mut self, name: &str, labels: MetricLabels) -> Arc<Gauge> {
        let registry = Arc::get_mut(&mut self.registry).unwrap();
        registry.register_gauge(name.to_string(), labels)
    }

    pub fn histogram(&mut self, name: &str, labels: MetricLabels, buckets: Vec<f64>) -> Arc<Histogram> {
        let registry = Arc::get_mut(&mut self.registry).unwrap();
        registry.register_histogram(name.to_string(), labels, buckets)
    }

    pub fn get_all_metrics(&self) -> Vec<Metric> {
        self.registry.get_all_metrics()
    }

    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        let metrics = self.get_all_metrics();

        for metric in metrics {
            match metric.value {
                MetricValue::Counter(value) => {
                    output.push_str(&format!(
                        "# TYPE {} counter\n",
                        metric.name
                    ));
                    output.push_str(&format!(
                        "{}{} {}\n",
                        metric.name,
                        format_labels(&metric.labels),
                        value
                    ));
                }
                MetricValue::Gauge(value) => {
                    output.push_str(&format!(
                        "# TYPE {} gauge\n",
                        metric.name
                    ));
                    output.push_str(&format!(
                        "{}{} {}\n",
                        metric.name,
                        format_labels(&metric.labels),
                        value
                    ));
                }
                MetricValue::Histogram(data) => {
                    output.push_str(&format!(
                        "# TYPE {} histogram\n",
                        metric.name
                    ));
                    for bucket in &data.buckets {
                        output.push_str(&format!(
                            "{}_bucket{} {} {}\n",
                            metric.name,
                            format_labels(&metric.labels),
                            bucket.count,
                            bucket.upper_bound
                        ));
                    }
                    output.push_str(&format!(
                        "{}_count{} {}\n",
                        metric.name,
                        format_labels(&metric.labels),
                        data.count
                    ));
                    output.push_str(&format!(
                        "{}_sum{} {}\n",
                        metric.name,
                        format_labels(&metric.labels),
                        data.sum
                    ));
                }
                MetricValue::Summary(_) => {
                    // 摘要格式实现
                }
            }
        }

        output
    }
}

impl Default for MetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 格式化标签为 Prometheus 格式
fn format_labels(labels: &MetricLabels) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let label_pairs: Vec<String> = labels
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect();

    format!("{{{}}}", label_pairs.join(","))
}

/// 健康检查状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub duration_ms: u64,
}

/// 健康检查器
pub struct SystemHealthChecker {
    checks: HashMap<String, Box<dyn Fn() -> HealthCheck + Send + Sync>>,
}

impl SystemHealthChecker {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }

    pub fn register_check<F>(&mut self, name: String, check: F)
    where
        F: Fn() -> HealthCheck + Send + Sync + 'static,
    {
        self.checks.insert(name, Box::new(check));
    }

    pub fn run_all_checks(&self) -> Vec<HealthCheck> {
        self.checks
            .values()
            .map(|check| check())
            .collect()
    }

    pub fn run_check(&self, name: &str) -> Option<HealthCheck> {
        self.checks.get(name).map(|check| check())
    }

    pub fn get_overall_status(&self) -> HealthStatus {
        let results = self.run_all_checks();
        if results.is_empty() {
            return HealthStatus::Healthy;
        }

        let has_unhealthy = results.iter().any(|r| r.status == HealthStatus::Unhealthy);
        let has_degraded = results.iter().any(|r| r.status == HealthStatus::Degraded);

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

impl Default for SystemHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    request_count: Arc<Counter>,
    request_duration: Arc<Histogram>,
    error_count: Arc<Counter>,
    active_connections: Arc<Gauge>,
}

impl std::fmt::Debug for PerformanceMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceMonitor")
            .field("request_count", &self.request_count.get())
            .field("error_count", &self.error_count.get())
            .field("active_connections", &self.active_connections.get())
            .finish()
    }
}

impl PerformanceMonitor {
    pub fn new(collector: &mut MetricCollector) -> Self {
        let request_count = collector.counter(
            "requests_total",
            HashMap::new(),
        );
        let request_duration = collector.histogram(
            "request_duration_seconds",
            HashMap::new(),
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0],
        );
        let error_count = collector.counter(
            "errors_total",
            HashMap::new(),
        );
        let active_connections = collector.gauge(
            "active_connections",
            HashMap::new(),
        );

        Self {
            request_count,
            request_duration,
            error_count,
            active_connections,
        }
    }

    pub fn record_request(&self, duration: Duration) {
        self.request_count.inc();
        self.request_duration.observe(duration.as_secs_f64());
    }

    pub fn record_error(&self) {
        self.error_count.inc();
    }

    pub fn set_active_connections(&self, count: u64) {
        self.active_connections.set(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter".to_string(), HashMap::new());
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge".to_string(), HashMap::new());
        assert_eq!(gauge.get(), 0);
        gauge.set(10);
        assert_eq!(gauge.get(), 10);
        gauge.inc();
        assert_eq!(gauge.get(), 11);
        gauge.dec();
        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new(
            "test_histogram".to_string(),
            HashMap::new(),
            vec![0.1, 0.5, 1.0, 5.0],
        );
        histogram.observe(0.2);
        histogram.observe(0.8);
        histogram.observe(2.0);

        let data = histogram.get_data();
        assert_eq!(data.count, 3);
        assert_eq!(data.sum, 3.0);
    }

    #[test]
    fn test_health_checker() {
        let mut checker = SystemHealthChecker::new();
        checker.register_check("test_check".to_string(), || HealthCheck {
            name: "test_check".to_string(),
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            timestamp: 0,
            duration_ms: 1,
        });

        let results = checker.run_all_checks();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, HealthStatus::Healthy);
    }
}

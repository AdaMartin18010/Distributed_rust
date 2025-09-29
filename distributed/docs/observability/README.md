# 监控与可观测性指南

> 关键不变量：RED/USE 指标分层落地；追踪与日志上下文关联；采样优先保留错误/慢调用。

本文档提供了分布式系统监控与可观测性的全面指南，包括指标收集、日志聚合、链路追踪和告警系统。

## 🎯 可观测性三大支柱

### 1. 指标 (Metrics)

- **RED 指标**: 请求率 (Rate)、错误率 (Error)、持续时间 (Duration)
- **USE 指标**: 利用率 (Utilization)、饱和度 (Saturation)、错误率 (Error)

> 选择指引：面向外部接口以 RED 为主、面向系统资源以 USE 为主；两者互补。

#### 分层指标建议

- RPC 层：`rpc.requests`, `rpc.errors`, `rpc.latency.ms{op}`
- 共识层：`raft.append.entries`, `raft.commit.index`, `raft.leader.elections`
- 复制层：`replica.acks`, `replica.lag.ms`, `replica.repair.count`
- 存储层：`wal.fsync.ms`, `snapshot.bytes`, `segment.crc.errors`
- 补偿层：`saga.steps`, `saga.compensate.count`, `saga.failure.matrix{kind}`

仪表设计建议：

- 为 P50/P95/P99 延迟使用直方图桶，区分读取/写入/共识路径；标签维度受控，避免高基数。

### 2. 日志 (Logs)

- **结构化日志**: 使用 JSON 格式，便于解析和查询
- **日志级别**: Trace, Debug, Info, Warn, Error, Fatal
- **上下文信息**: 包含 trace_id, span_id, user_id 等
  - 与追踪关联：在日志条目中注入 trace/span 上下文，便于跨系统排障。

### 3. 链路追踪 (Tracing)

- **分布式追踪**: 跨服务调用链追踪
- **性能分析**: 识别性能瓶颈和热点
- **错误定位**: 快速定位错误根源
  - 采样策略：优先保留错误与慢调用，降低低价值流量；导出与存储遵循数据保留策略。

## 📊 指标收集

### 1. 基础指标收集器

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    pub gauges: Arc<RwLock<HashMap<String, AtomicU64>>>,
    pub histograms: Arc<RwLock<HashMap<String, Histogram>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write().await;
        counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(value, Ordering::SeqCst);
    }

    pub async fn set_gauge(&self, name: &str, value: u64) {
        let mut gauges = self.gauges.write().await;
        gauges
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(value, Ordering::SeqCst);
    }

    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().await;
        histograms
            .entry(name.to_string())
            .or_insert_with(|| Histogram::new())
            .record(value);
    }
}
```

### 2. RED 指标实现

```rust
#[derive(Debug)]
pub struct RedMetrics {
    metrics: Metrics,
}

impl RedMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Metrics::new(),
        }
    }

    // Rate: 请求率
    pub async fn record_request(&self, endpoint: &str, method: &str) {
        let metric_name = format!("requests_total{{endpoint=\"{}\",method=\"{}\"}}", endpoint, method);
        self.metrics.increment_counter(&metric_name, 1).await;
    }

    // Error: 错误率
    pub async fn record_error(&self, endpoint: &str, method: &str, status_code: u16) {
        let metric_name = format!("errors_total{{endpoint=\"{}\",method=\"{}\",status=\"{}\"}}", 
                                 endpoint, method, status_code);
        self.metrics.increment_counter(&metric_name, 1).await;
    }

    // Duration: 持续时间
    pub async fn record_duration(&self, endpoint: &str, method: &str, duration: Duration) {
        let metric_name = format!("request_duration_seconds{{endpoint=\"{}\",method=\"{}\"}}", 
                                 endpoint, method);
        self.metrics.record_histogram(&metric_name, duration.as_secs_f64()).await;
    }
}
```

### 3. USE 指标实现

```rust
#[derive(Debug)]
pub struct UseMetrics {
    metrics: Metrics,
}

impl UseMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Metrics::new(),
        }
    }

    // Utilization: 利用率
    pub async fn record_cpu_utilization(&self, node_id: &str, utilization: f64) {
        let metric_name = format!("cpu_utilization{{node=\"{}\"}}", node_id);
        self.metrics.set_gauge(&metric_name, (utilization * 100.0) as u64).await;
    }

    pub async fn record_memory_utilization(&self, node_id: &str, utilization: f64) {
        let metric_name = format!("memory_utilization{{node=\"{}\"}}", node_id);
        self.metrics.set_gauge(&metric_name, (utilization * 100.0) as u64).await;
    }

    // Saturation: 饱和度
    pub async fn record_queue_length(&self, queue_name: &str, length: usize) {
        let metric_name = format!("queue_length{{queue=\"{}\"}}", queue_name);
        self.metrics.set_gauge(&metric_name, length as u64).await;
    }

    // Error: 错误率
    pub async fn record_system_error(&self, component: &str, error_type: &str) {
        let metric_name = format!("system_errors_total{{component=\"{}\",type=\"{}\"}}", 
                                 component, error_type);
        self.metrics.increment_counter(&metric_name, 1).await;
    }
}
```

## 📝 日志聚合

### 1. 结构化日志配置

```rust
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(false)
        )
        .init();
}

// 使用示例
pub async fn process_request(request: &Request) -> Result<Response, Error> {
    let span = tracing::info_span!("process_request", 
                                  request_id = %request.id,
                                  user_id = %request.user_id);
    let _enter = span.enter();

    info!("开始处理请求");
    
    match validate_request(request).await {
        Ok(_) => {
            debug!("请求验证通过");
            let response = execute_request(request).await?;
            info!("请求处理完成", duration_ms = response.duration.as_millis());
            Ok(response)
        }
        Err(e) => {
            warn!("请求验证失败", error = %e);
            Err(e)
        }
    }
}
```

### 2. 日志收集器

```rust
use std::sync::mpsc;
use std::thread;

pub struct LogCollector {
    sender: mpsc::Sender<LogEntry>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
    pub span_context: Option<SpanContext>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
}

impl LogCollector {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        // 启动日志处理线程
        thread::spawn(move || {
            Self::process_logs(receiver);
        });
        
        Self { sender }
    }

    fn process_logs(receiver: mpsc::Receiver<LogEntry>) {
        let mut batch = Vec::new();
        let mut last_flush = std::time::Instant::now();
        
        for log_entry in receiver {
            batch.push(log_entry);
            
            // 批量处理或定时刷新
            if batch.len() >= 100 || last_flush.elapsed() > Duration::from_secs(5) {
                Self::flush_logs(&mut batch);
                last_flush = std::time::Instant::now();
            }
        }
    }

    fn flush_logs(batch: &mut Vec<LogEntry>) {
        if batch.is_empty() {
            return;
        }

        // 发送到日志聚合系统
        for log_entry in batch.drain(..) {
            Self::send_to_aggregator(log_entry);
        }
    }

    fn send_to_aggregator(log_entry: LogEntry) {
        // 发送到 Elasticsearch, Splunk, 或其他日志聚合系统
        let json = serde_json::to_string(&log_entry).unwrap();
        println!("LOG: {}", json);
    }

    pub fn collect_log(&self, entry: LogEntry) {
        self.sender.send(entry).unwrap_or_else(|_| {
            eprintln!("日志收集器已关闭");
        });
    }
}
```

## 🔍 链路追踪

### 1. 分布式追踪器

```rust
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DistributedTracer {
    service_name: String,
    collector_endpoint: String,
    spans: Arc<RwLock<Vec<Span>>>,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
}

#[derive(Debug, Clone)]
pub struct SpanLog {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub fields: HashMap<String, String>,
}

impl DistributedTracer {
    pub fn new(service_name: String, collector_endpoint: String) -> Self {
        Self {
            service_name,
            collector_endpoint,
            spans: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn start_span(&self, operation_name: &str, parent_span: Option<&Span>) -> Span {
        let trace_id = parent_span
            .map(|p| p.trace_id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let span_id = Uuid::new_v4().to_string();
        let parent_span_id = parent_span.map(|p| p.span_id.clone());

        Span {
            trace_id,
            span_id,
            parent_span_id,
            operation_name: operation_name.to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
        }
    }

    pub fn finish_span(&self, mut span: Span) {
        span.end_time = Some(chrono::Utc::now());
        
        let spans = self.spans.clone();
        tokio::spawn(async move {
            let mut spans = spans.write().await;
            spans.push(span);
        });
    }

    pub fn add_tag(&self, span: &mut Span, key: &str, value: &str) {
        span.tags.insert(key.to_string(), value.to_string());
    }

    pub fn add_log(&self, span: &mut Span, fields: HashMap<String, String>) {
        span.logs.push(SpanLog {
            timestamp: chrono::Utc::now(),
            fields,
        });
    }

    pub async fn export_spans(&self) -> Result<(), Error> {
        let spans = {
            let mut spans = self.spans.write().await;
            spans.drain(..).collect::<Vec<_>>()
        };

        if spans.is_empty() {
            return Ok(());
        }

        // 发送到追踪收集器
        self.send_to_collector(spans).await?;
        Ok(())
    }

    async fn send_to_collector(&self, spans: Vec<Span>) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let payload = serde_json::to_string(&spans)?;
        
        let response = client
            .post(&self.collector_endpoint)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::CollectorError(format!("HTTP {}", response.status())));
        }

        Ok(())
    }
}
```

## 🚨 告警系统

### 1. 告警规则引擎

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub cooldown: Duration,
    pub notification_channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    CounterThreshold {
        metric_name: String,
        threshold: u64,
        operator: ComparisonOperator,
        time_window: Duration,
    },
    GaugeThreshold {
        metric_name: String,
        threshold: f64,
        operator: ComparisonOperator,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

pub struct AlertEngine {
    rules: HashMap<String, AlertRule>,
    metrics: Metrics,
    notification_service: NotificationService,
    alert_history: AlertHistory,
}

impl AlertEngine {
    pub fn new(metrics: Metrics, notification_service: NotificationService) -> Self {
        Self {
            rules: HashMap::new(),
            metrics,
            notification_service,
            alert_history: AlertHistory::new(),
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    pub async fn evaluate_rules(&self) -> Result<(), Error> {
        for rule in self.rules.values() {
            if self.should_evaluate_rule(rule).await {
                match self.evaluate_condition(&rule.condition).await {
                    Ok(true) => {
                        self.trigger_alert(rule).await?;
                    }
                    Ok(false) => {
                        // 条件不满足，重置告警状态
                        self.reset_alert_state(rule).await;
                    }
                    Err(e) => {
                        tracing::error!("评估告警规则失败: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn should_evaluate_rule(&self, rule: &AlertRule) -> bool {
        if let Some(last_alert) = self.alert_history.get_last_alert(&rule.id).await {
            return last_alert.timestamp + rule.cooldown < chrono::Utc::now();
        }
        true
    }

    async fn evaluate_condition(&self, condition: &AlertCondition) -> Result<bool, Error> {
        match condition {
            AlertCondition::CounterThreshold { metric_name, threshold, operator, time_window } => {
                let value = self.metrics.get_counter(metric_name).await
                    .ok_or_else(|| Error::MetricNotFound(metric_name.clone()))?;
                Ok(self.compare_values(value as f64, *threshold as f64, operator))
            }
            AlertCondition::GaugeThreshold { metric_name, threshold, operator } => {
                let value = self.metrics.get_gauge(metric_name).await
                    .ok_or_else(|| Error::MetricNotFound(metric_name.clone()))?;
                Ok(self.compare_values(value as f64, *threshold, operator))
            }
        }
    }

    fn compare_values(&self, value: f64, threshold: f64, operator: &ComparisonOperator) -> bool {
        match operator {
            ComparisonOperator::GreaterThan => value > threshold,
            ComparisonOperator::LessThan => value < threshold,
            ComparisonOperator::GreaterThanOrEqual => value >= threshold,
            ComparisonOperator::LessThanOrEqual => value <= threshold,
            ComparisonOperator::Equal => (value - threshold).abs() < f64::EPSILON,
        }
    }

    async fn trigger_alert(&self, rule: &AlertRule) -> Result<(), Error> {
        let alert = Alert {
            id: Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            severity: rule.severity.clone(),
            message: format!("告警触发: {}", rule.name),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        // 记录告警历史
        self.alert_history.record_alert(alert.clone()).await;

        // 发送通知
        for channel in &rule.notification_channels {
            self.notification_service.send_alert(channel, &alert).await?;
        }

        tracing::warn!("告警触发", rule_id = %rule.id, severity = ?rule.severity);
        Ok(())
    }
}
```

## 📈 仪表板配置

### 1. Prometheus 集成

```rust
use prometheus::{Counter, Histogram, Gauge, Registry, TextEncoder, Encoder};

pub struct PrometheusMetrics {
    pub registry: Registry,
    pub request_counter: Counter,
    pub request_duration: Histogram,
    pub active_connections: Gauge,
}

impl PrometheusMetrics {
    pub fn new() -> Result<Self, Error> {
        let registry = Registry::new();
        
        let request_counter = Counter::new(
            "http_requests_total",
            "Total number of HTTP requests"
        )?;
        
        let request_duration = Histogram::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        )?;
        
        let active_connections = Gauge::new(
            "active_connections",
            "Number of active connections"
        )?;

        registry.register(Box::new(request_counter.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;

        Ok(Self {
            registry,
            request_counter,
            request_duration,
            active_connections,
        })
    }

    pub fn record_request(&self, method: &str, endpoint: &str, status_code: u16, duration: Duration) {
        self.request_counter
            .with_label_values(&[method, endpoint, &status_code.to_string()])
            .inc();
        
        self.request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }

    pub fn set_active_connections(&self, count: i64) {
        self.active_connections.set(count);
    }

    pub fn export_metrics(&self) -> Result<String, Error> {
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        
        Ok(String::from_utf8(buffer)?)
    }
}
```

### 2. Grafana 仪表板配置

```json
{
  "dashboard": {
    "title": "分布式系统监控仪表板",
    "panels": [
      {
        "title": "请求率",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{endpoint}}"
          }
        ]
      },
      {
        "title": "响应时间",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "P95 响应时间"
          },
          {
            "expr": "histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "P50 响应时间"
          }
        ]
      },
      {
        "title": "错误率",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m]) / rate(http_requests_total[5m]) * 100",
            "legendFormat": "错误率 (%)"
          }
        ]
      }
      ,
      {
        "title": "Raft 选举次数",
        "type": "stat",
        "targets": [
          { "expr": "rate(raft_leader_elections_total[5m])", "legendFormat": "选举/秒" }
        ]
      },
      {
        "title": "副本落后 (ms)",
        "type": "graph",
        "targets": [
          { "expr": "replica_lag_ms", "legendFormat": "{{node}}" }
        ]
      }
    ]
  }
}
```

## 🔗 相关资源

- [快速开始指南](../QUICKSTART.md)
- [系统设计最佳实践](../design/BEST_PRACTICES.md)
- [性能优化技巧](../performance/OPTIMIZATION.md)
- [测试策略](../testing/README.md)
- [常见陷阱与调试](../PITFALLS.md)

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/c20_distributed/issues)
- **Discussions**: [讨论交流](https://github.com/your-org/c20_distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/c20-distributed)

---

**全面监控！** 🚀 建立完善的可观测性体系，确保分布式系统的稳定运行和快速故障定位。

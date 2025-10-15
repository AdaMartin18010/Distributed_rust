# 可观测性

> 分布式系统中的监控、追踪、日志和指标收集

## 目录

- [可观测性](#可观测性)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [可观测性三大支柱](#可观测性三大支柱)
    - [指标类型](#指标类型)
  - [📊 指标收集](#-指标收集)
    - [指标收集器](#指标收集器)
    - [计数器实现](#计数器实现)
    - [直方图实现](#直方图实现)
  - [🔍 分布式追踪](#-分布式追踪)
    - [追踪上下文](#追踪上下文)
    - [跨度 (Span) 实现](#跨度-span-实现)
    - [追踪器实现](#追踪器实现)
  - [📝 日志管理](#-日志管理)
    - [结构化日志](#结构化日志)
    - [JSON 格式化器](#json-格式化器)
  - [🔍 健康检查](#-健康检查)
    - [健康检查器](#健康检查器)
    - [具体健康检查实现](#具体健康检查实现)
  - [🧪 测试策略](#-测试策略)
    - [可观测性测试](#可观测性测试)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

可观测性是分布式系统的重要特性，通过监控、追踪、日志和指标收集，帮助开发者理解系统行为、诊断问题和优化性能。

## 🎯 核心概念

### 可观测性三大支柱

```rust
#[derive(Debug, Clone)]
pub enum ObservabilityType {
    Metrics,    // 指标
    Traces,     // 追踪
    Logs,       // 日志
}

pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub logging_enabled: bool,
    pub metrics_port: u16,
    pub tracing_endpoint: String,
    pub log_level: String,
}
```

### 指标类型

```rust
#[derive(Debug, Clone)]
pub enum MetricType {
    Counter,    // 计数器
    Gauge,      // 仪表盘
    Histogram,  // 直方图
    Summary,    // 摘要
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}
```

## 📊 指标收集

### 指标收集器

```rust
pub struct MetricsCollector {
    metrics: HashMap<String, Box<dyn Metric>>,
    registry: MetricRegistry,
}

pub trait Metric {
    fn get_name(&self) -> &str;
    fn get_type(&self) -> MetricType;
    fn get_value(&self) -> f64;
    fn get_labels(&self) -> &HashMap<String, String>;
    fn update(&mut self, value: f64);
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            registry: MetricRegistry::new(),
        }
    }
    
    pub fn register_metric(&mut self, name: String, metric: Box<dyn Metric>) {
        self.metrics.insert(name, metric);
    }
    
    pub fn increment_counter(&mut self, name: &str, labels: HashMap<String, String>) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.update(1.0);
        }
    }
    
    pub fn set_gauge(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.update(value);
        }
    }
    
    pub fn record_histogram(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.update(value);
        }
    }
    
    pub fn get_metrics(&self) -> Vec<Metric> {
        self.metrics.values()
            .map(|m| Metric {
                name: m.get_name().to_string(),
                metric_type: m.get_type(),
                value: m.get_value(),
                labels: m.get_labels().clone(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            })
            .collect()
    }
}
```

### 计数器实现

```rust
pub struct Counter {
    name: String,
    value: f64,
    labels: HashMap<String, String>,
}

impl Counter {
    pub fn new(name: String, labels: HashMap<String, String>) -> Self {
        Self {
            name,
            value: 0.0,
            labels,
        }
    }
    
    pub fn inc(&mut self) {
        self.value += 1.0;
    }
    
    pub fn inc_by(&mut self, amount: f64) {
        self.value += amount;
    }
    
    pub fn get_value(&self) -> f64 {
        self.value
    }
}

impl Metric for Counter {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_type(&self) -> MetricType {
        MetricType::Counter
    }
    
    fn get_value(&self) -> f64 {
        self.value
    }
    
    fn get_labels(&self) -> &HashMap<String, String> {
        &self.labels
    }
    
    fn update(&mut self, value: f64) {
        self.value += value;
    }
}
```

### 直方图实现

```rust
pub struct Histogram {
    name: String,
    buckets: Vec<f64>,
    counts: Vec<u64>,
    sum: f64,
    count: u64,
    labels: HashMap<String, String>,
}

impl Histogram {
    pub fn new(name: String, buckets: Vec<f64>, labels: HashMap<String, String>) -> Self {
        let counts = vec![0; buckets.len()];
        
        Self {
            name,
            buckets,
            counts,
            sum: 0.0,
            count: 0,
            labels,
        }
    }
    
    pub fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
        
        // 更新桶计数
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                self.counts[i] += 1;
            }
        }
    }
    
    pub fn get_percentile(&self, percentile: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        
        let target_count = (self.count as f64 * percentile / 100.0) as u64;
        let mut current_count = 0;
        
        for (i, &count) in self.counts.iter().enumerate() {
            current_count += count;
            if current_count >= target_count {
                return self.buckets[i];
            }
        }
        
        self.buckets.last().unwrap_or(&0.0).clone()
    }
}

impl Metric for Histogram {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_type(&self) -> MetricType {
        MetricType::Histogram
    }
    
    fn get_value(&self) -> f64 {
        self.sum
    }
    
    fn get_labels(&self) -> &HashMap<String, String> {
        &self.labels
    }
    
    fn update(&mut self, value: f64) {
        self.observe(value);
    }
}
```

## 🔍 分布式追踪

### 追踪上下文

```rust
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    pub fn new(trace_id: String, span_id: String) -> Self {
        Self {
            trace_id,
            span_id,
            parent_span_id: None,
            sampled: true,
            baggage: HashMap::new(),
        }
    }
    
    pub fn with_parent(mut self, parent_span_id: String) -> Self {
        self.parent_span_id = Some(parent_span_id);
        self
    }
    
    pub fn with_baggage(mut self, key: String, value: String) -> Self {
        self.baggage.insert(key, value);
        self
    }
}
```

### 跨度 (Span) 实现

```rust
pub struct Span {
    context: TraceContext,
    name: String,
    start_time: Instant,
    end_time: Option<Instant>,
    attributes: HashMap<String, String>,
    events: Vec<SpanEvent>,
    status: SpanStatus,
}

#[derive(Debug, Clone)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: Instant,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Ok,
    Error(String),
    Unset,
}

impl Span {
    pub fn new(context: TraceContext, name: String) -> Self {
        Self {
            context,
            name,
            start_time: Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Unset,
        }
    }
    
    pub fn set_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }
    
    pub fn add_event(&mut self, name: String, attributes: HashMap<String, String>) {
        self.events.push(SpanEvent {
            name,
            timestamp: Instant::now(),
            attributes,
        });
    }
    
    pub fn set_status(&mut self, status: SpanStatus) {
        self.status = status;
    }
    
    pub fn finish(&mut self) {
        self.end_time = Some(Instant::now());
    }
    
    pub fn get_duration(&self) -> Option<Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }
    
    pub fn create_child(&self, name: String) -> Span {
        let child_context = TraceContext::new(
            self.context.trace_id.clone(),
            uuid::Uuid::new_v4().to_string(),
        ).with_parent(self.context.span_id.clone());
        
        Span::new(child_context, name)
    }
}
```

### 追踪器实现

```rust
pub struct Tracer {
    service_name: String,
    spans: HashMap<String, Span>,
    exporter: Box<dyn SpanExporter>,
}

pub trait SpanExporter {
    async fn export(&self, spans: Vec<Span>) -> Result<(), Box<dyn std::error::Error>>;
}

impl Tracer {
    pub fn new(service_name: String, exporter: Box<dyn SpanExporter>) -> Self {
        Self {
            service_name,
            spans: HashMap::new(),
            exporter,
        }
    }
    
    pub fn start_span(&mut self, name: String) -> String {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let span_id = uuid::Uuid::new_v4().to_string();
        
        let context = TraceContext::new(trace_id, span_id);
        let span = Span::new(context, name);
        
        let span_key = span.context.span_id.clone();
        self.spans.insert(span_key.clone(), span);
        
        span_key
    }
    
    pub fn start_child_span(&mut self, parent_span_id: &str, name: String) -> String {
        if let Some(parent_span) = self.spans.get(parent_span_id) {
            let child_span = parent_span.create_child(name);
            let span_key = child_span.context.span_id.clone();
            self.spans.insert(span_key.clone(), child_span);
            span_key
        } else {
            self.start_span(name)
        }
    }
    
    pub fn finish_span(&mut self, span_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(span) = self.spans.get_mut(span_id) {
            span.finish();
        }
        
        Ok(())
    }
    
    pub async fn export_spans(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let spans: Vec<Span> = self.spans.values().cloned().collect();
        self.exporter.export(spans).await?;
        
        // 清理已导出的跨度
        self.spans.clear();
        
        Ok(())
    }
}
```

## 📝 日志管理

### 结构化日志

```rust
pub struct StructuredLogger {
    level: LogLevel,
    formatter: Box<dyn LogFormatter>,
    appenders: Vec<Box<dyn LogAppender>>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub trait LogFormatter {
    fn format(&self, record: &LogRecord) -> String;
}

pub trait LogAppender {
    async fn append(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: SystemTime,
    pub target: String,
    pub fields: HashMap<String, String>,
    pub span_context: Option<TraceContext>,
}

impl StructuredLogger {
    pub fn new(level: LogLevel, formatter: Box<dyn LogFormatter>) -> Self {
        Self {
            level,
            formatter,
            appenders: Vec::new(),
        }
    }
    
    pub fn add_appender(&mut self, appender: Box<dyn LogAppender>) {
        self.appenders.push(appender);
    }
    
    pub async fn log(&self, level: LogLevel, message: String, fields: HashMap<String, String>) {
        if level < self.level {
            return;
        }
        
        let record = LogRecord {
            level,
            message,
            timestamp: SystemTime::now(),
            target: "distributed".to_string(),
            fields,
            span_context: None,
        };
        
        let formatted = self.formatter.format(&record);
        
        for appender in &self.appenders {
            if let Err(e) = appender.append(&record).await {
                eprintln!("Failed to append log: {}", e);
            }
        }
    }
    
    pub async fn info(&self, message: String, fields: HashMap<String, String>) {
        self.log(LogLevel::Info, message, fields).await;
    }
    
    pub async fn error(&self, message: String, fields: HashMap<String, String>) {
        self.log(LogLevel::Error, message, fields).await;
    }
    
    pub async fn debug(&self, message: String, fields: HashMap<String, String>) {
        self.log(LogLevel::Debug, message, fields).await;
    }
}
```

### JSON 格式化器

```rust
pub struct JsonFormatter;

impl LogFormatter for JsonFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let mut json = serde_json::Map::new();
        
        json.insert("timestamp".to_string(), serde_json::Value::String(
            record.timestamp.duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()
        ));
        json.insert("level".to_string(), serde_json::Value::String(
            format!("{:?}", record.level)
        ));
        json.insert("message".to_string(), serde_json::Value::String(
            record.message.clone()
        ));
        json.insert("target".to_string(), serde_json::Value::String(
            record.target.clone()
        ));
        
        // 添加字段
        for (key, value) in &record.fields {
            json.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        
        // 添加追踪上下文
        if let Some(context) = &record.span_context {
            json.insert("trace_id".to_string(), serde_json::Value::String(
                context.trace_id.clone()
            ));
            json.insert("span_id".to_string(), serde_json::Value::String(
                context.span_id.clone()
            ));
        }
        
        serde_json::to_string(&json).unwrap_or_default()
    }
}
```

## 🔍 健康检查

### 健康检查器

```rust
pub struct HealthChecker {
    checks: HashMap<String, Box<dyn HealthCheck>>,
    check_interval: Duration,
    timeout: Duration,
}

pub trait HealthCheck {
    async fn check(&self) -> HealthStatus;
    fn get_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: HealthState,
    pub message: String,
    pub details: HashMap<String, String>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthState {
    Healthy,
    Unhealthy,
    Degraded,
}

impl HealthChecker {
    pub fn new(check_interval: Duration, timeout: Duration) -> Self {
        Self {
            checks: HashMap::new(),
            check_interval,
            timeout,
        }
    }
    
    pub fn add_check(&mut self, name: String, check: Box<dyn HealthCheck>) {
        self.checks.insert(name, check);
    }
    
    pub async fn check_health(&self) -> HealthStatus {
        let mut overall_status = HealthState::Healthy;
        let mut details = HashMap::new();
        let mut messages = Vec::new();
        
        for (name, check) in &self.checks {
            match tokio::time::timeout(self.timeout, check.check()).await {
                Ok(status) => {
                    details.insert(name.clone(), format!("{:?}", status.status));
                    
                    match status.status {
                        HealthState::Unhealthy => {
                            overall_status = HealthState::Unhealthy;
                            messages.push(format!("{}: {}", name, status.message));
                        }
                        HealthState::Degraded => {
                            if overall_status == HealthState::Healthy {
                                overall_status = HealthState::Degraded;
                            }
                            messages.push(format!("{}: {}", name, status.message));
                        }
                        HealthState::Healthy => {
                            // 健康状态不需要特殊处理
                        }
                    }
                }
                Err(_) => {
                    overall_status = HealthState::Unhealthy;
                    details.insert(name.clone(), "timeout".to_string());
                    messages.push(format!("{}: check timeout", name));
                }
            }
        }
        
        HealthStatus {
            status: overall_status,
            message: messages.join("; "),
            details,
            timestamp: SystemTime::now(),
        }
    }
    
    pub async fn start_health_checking(&self) -> Result<(), Box<dyn std::error::Error>> {
        let checks = self.checks.clone();
        let check_interval = self.check_interval;
        let timeout = self.timeout;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            
            loop {
                interval.tick().await;
                
                for (name, check) in &checks {
                    match tokio::time::timeout(timeout, check.check()).await {
                        Ok(status) => {
                            println!("Health check {}: {:?}", name, status.status);
                        }
                        Err(_) => {
                            println!("Health check {}: timeout", name);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
}
```

### 具体健康检查实现

```rust
pub struct DatabaseHealthCheck {
    connection_string: String,
}

impl DatabaseHealthCheck {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> HealthStatus {
        // 实现数据库健康检查逻辑
        match self.ping_database().await {
            Ok(_) => HealthStatus {
                status: HealthState::Healthy,
                message: "Database connection OK".to_string(),
                details: HashMap::new(),
                timestamp: SystemTime::now(),
            },
            Err(e) => HealthStatus {
                status: HealthState::Unhealthy,
                message: format!("Database connection failed: {}", e),
                details: HashMap::new(),
                timestamp: SystemTime::now(),
            },
        }
    }
    
    fn get_name(&self) -> &str {
        "database"
    }
}

impl DatabaseHealthCheck {
    async fn ping_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 实现数据库 ping 逻辑
        Ok(())
    }
}
```

## 🧪 测试策略

### 可观测性测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_counter_metric() {
        let mut counter = Counter::new("test_counter".to_string(), HashMap::new());
        
        counter.inc();
        counter.inc_by(5.0);
        
        assert_eq!(counter.get_value(), 6.0);
    }
    
    #[test]
    fn test_histogram_metric() {
        let mut histogram = Histogram::new(
            "test_histogram".to_string(),
            vec![0.1, 0.5, 1.0, 5.0],
            HashMap::new(),
        );
        
        histogram.observe(0.3);
        histogram.observe(0.7);
        histogram.observe(2.0);
        
        assert_eq!(histogram.get_percentile(50.0), 0.5);
        assert_eq!(histogram.get_percentile(90.0), 1.0);
    }
    
    #[tokio::test]
    async fn test_tracer() {
        struct MockExporter;
        
        impl SpanExporter for MockExporter {
            async fn export(&self, spans: Vec<Span>) -> Result<(), Box<dyn std::error::Error>> {
                println!("Exported {} spans", spans.len());
                Ok(())
            }
        }
        
        let mut tracer = Tracer::new("test_service".to_string(), Box::new(MockExporter));
        
        let span_id = tracer.start_span("test_operation".to_string());
        tracer.finish_span(&span_id).unwrap();
        
        tracer.export_spans().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_health_checker() {
        struct MockHealthCheck {
            name: String,
            should_fail: bool,
        }
        
        impl HealthCheck for MockHealthCheck {
            async fn check(&self) -> HealthStatus {
                if self.should_fail {
                    HealthStatus {
                        status: HealthState::Unhealthy,
                        message: "Mock check failed".to_string(),
                        details: HashMap::new(),
                        timestamp: SystemTime::now(),
                    }
                } else {
                    HealthStatus {
                        status: HealthState::Healthy,
                        message: "Mock check passed".to_string(),
                        details: HashMap::new(),
                        timestamp: SystemTime::now(),
                    }
                }
            }
            
            fn get_name(&self) -> &str {
                &self.name
            }
        }
        
        let health_checker = HealthChecker::new(Duration::from_secs(1), Duration::from_millis(100));
        
        let status = health_checker.check_health().await;
        assert_eq!(status.status, HealthState::Healthy);
    }
}
```

## 📚 进一步阅读

- [性能优化](../performance/OPTIMIZATION.md) - 性能监控和优化
- [测试策略](../testing/README.md) - 可观测性测试
- [故障处理](../failure/README.md) - 故障检测和监控
- [共识机制](../consensus/README.md) - 共识算法的可观测性

## 🔗 相关文档

- [性能优化](../performance/OPTIMIZATION.md)
- [测试策略](../testing/README.md)
- [故障处理](../failure/README.md)
- [共识机制](../consensus/README.md)
- [实验指南](../EXPERIMENT_GUIDE.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

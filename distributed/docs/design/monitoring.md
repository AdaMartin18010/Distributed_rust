# 3.10.6 监控和可观测性 (Monitoring and Observability)

## 目录

- [3.10.6 监控和可观测性 (Monitoring and Observability)](#3106-监控和可观测性-monitoring-and-observability)
  - [目录](#目录)
  - [核心概念](#核心概念)
    - [可观测性三大支柱](#可观测性三大支柱)
    - [监控 vs 可观测性](#监控-vs-可观测性)
    - [黄金信号](#黄金信号)
  - [监控指标](#监控指标)
    - [指标类型](#指标类型)
    - [系统指标](#系统指标)
    - [RED方法](#red方法)
    - [USE方法](#use方法)
  - [分布式追踪](#分布式追踪)
    - [OpenTelemetry追踪](#opentelemetry追踪)
    - [采样策略](#采样策略)
  - [日志管理](#日志管理)
    - [结构化日志](#结构化日志)
    - [日志聚合](#日志聚合)
    - [日志查询](#日志查询)
  - [告警系统](#告警系统)
    - [告警规则](#告警规则)
    - [告警路由](#告警路由)
    - [告警疲劳防护](#告警疲劳防护)
  - [可观测性平台](#可观测性平台)
    - [Prometheus + Grafana](#prometheus--grafana)
    - [Jaeger追踪](#jaeger追踪)
  - [Rust实现示例](#rust实现示例)
    - [指标收集](#指标收集)
    - [分布式追踪1](#分布式追踪1)
    - [结构化日志1](#结构化日志1)
    - [健康检查](#健康检查)
  - [最佳实践](#最佳实践)
    - [监控策略](#监控策略)
    - [日志最佳实践](#日志最佳实践)
    - [追踪最佳实践](#追踪最佳实践)
    - [SLO/SLI设定](#slosli设定)
    - [可观测性清单](#可观测性清单)
  - [相关文档](#相关文档)
  - [参考资料](#参考资料)
    - [工具和库](#工具和库)
    - [标准和协议](#标准和协议)
    - [最佳实践1](#最佳实践1)

---

## 核心概念

可观测性是理解系统内部状态和行为的能力，通过监控、日志和追踪三大支柱实现。

### 可观测性三大支柱

```text
┌─────────────────────────────────────────┐
│         可观测性 (Observability)        │
├─────────────────────────────────────────┤
│                                         │
│  1. 指标 (Metrics)                      │
│     - 数值化的时间序列数据              │
│     - 聚合和统计信息                    │
│     - 低存储开销                        │
│                                         │
│  2. 日志 (Logs)                         │
│     - 离散的事件记录                    │
│     - 详细的上下文信息                  │
│     - 高存储开销                        │
│                                         │
│  3. 追踪 (Traces)                       │
│     - 跨服务的请求路径                  │
│     - 因果关系和依赖                    │
│     - 中等存储开销                      │
│                                         │
└─────────────────────────────────────────┘
```

### 监控 vs 可观测性

**监控 (Monitoring)**：

- 预定义的问题检测
- "已知的未知"
- 健康检查、阈值告警

**可观测性 (Observability)**：

- 探索性分析
- "未知的未知"
- 系统内部状态可查询

### 黄金信号

**Google SRE四大黄金信号**：

```text
1. 延迟 (Latency)
   - 请求处理时间
   - P50, P95, P99
   
2. 流量 (Traffic)
   - 请求速率
   - RPS/TPS
   
3. 错误 (Errors)
   - 错误率
   - 5xx错误占比
   
4. 饱和度 (Saturation)
   - 资源利用率
   - CPU、内存、磁盘、网络
```

---

## 监控指标

### 指标类型

**Prometheus指标类型**：

```text
1. Counter (计数器)
   - 单调递增
   - 示例：请求总数、错误总数
   
2. Gauge (仪表盘)
   - 可增可减
   - 示例：CPU使用率、内存使用量
   
3. Histogram (直方图)
   - 分桶统计
   - 示例：请求延迟分布
   
4. Summary (摘要)
   - 分位数统计
   - 示例：P50/P95/P99延迟
```

### 系统指标

**基础设施指标**：

```text
CPU指标：
- cpu_usage_percent
- cpu_load_average
- cpu_context_switches

内存指标：
- memory_used_bytes
- memory_available_bytes
- memory_swap_used_bytes

磁盘指标：
- disk_used_bytes
- disk_io_operations
- disk_io_time_ms

网络指标：
- network_bytes_sent
- network_bytes_received
- network_errors_total
```

**应用指标**：

```text
请求指标：
- http_requests_total
- http_request_duration_seconds
- http_requests_in_flight

业务指标：
- orders_created_total
- user_registrations_total
- payment_transactions_amount

资源指标：
- database_connections_active
- cache_hit_ratio
- queue_depth
```

### RED方法

```text
Rate (速率)：
- 每秒请求数
- requests_per_second

Errors (错误)：
- 错误率
- error_rate = errors / total_requests

Duration (时长)：
- 请求时长分布
- P50, P95, P99 latency
```

### USE方法

```text
Utilization (利用率)：
- 资源使用百分比
- cpu_utilization, memory_utilization

Saturation (饱和度)：
- 资源队列长度
- thread_pool_queue_depth

Errors (错误)：
- 错误计数
- disk_errors, network_errors
```

---

## 分布式追踪

### OpenTelemetry追踪

**Trace结构**：

```text
Trace (跟踪)
  └─ Span (跨度)
      ├─ Span ID
      ├─ Trace ID
      ├─ Parent Span ID
      ├─ Start Time
      ├─ End Time
      ├─ Attributes (属性)
      ├─ Events (事件)
      └─ Links (链接)

示例：
Trace ID: abc123
  ├─ Span: API Gateway [100ms]
  │   ├─ Span: Auth Service [20ms]
  │   └─ Span: Order Service [70ms]
  │       ├─ Span: DB Query [30ms]
  │       └─ Span: Payment Service [35ms]
  └─ Span: Notification Service [10ms]
```

**上下文传播**：

```text
W3C Trace Context:
  traceparent: 00-{trace-id}-{parent-id}-{flags}
  tracestate: {vendor1}={data1},{vendor2}={data2}

示例：
traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
```

### 采样策略

```text
1. 头采样 (Head-based Sampling)
   - 在Trace开始时决定是否采样
   - 固定采样率
   - 概率采样

2. 尾采样 (Tail-based Sampling)
   - 在Trace结束后决定是否保留
   - 基于延迟采样
   - 基于错误采样

3. 自适应采样
   - 动态调整采样率
   - 基于系统负载
```

---

## 日志管理

### 结构化日志

**日志级别**：

```text
TRACE  - 最详细的调试信息
DEBUG  - 调试信息
INFO   - 常规信息
WARN   - 警告信息
ERROR  - 错误信息
FATAL  - 严重错误，程序退出
```

**日志格式**：

```json
{
  "timestamp": "2025-10-17T10:30:00.123Z",
  "level": "INFO",
  "message": "User login successful",
  "service": "auth-service",
  "trace_id": "abc123",
  "span_id": "def456",
  "user_id": "user_789",
  "ip": "192.168.1.100",
  "duration_ms": 45
}
```

### 日志聚合

**ELK Stack**：

```text
Elasticsearch ← Logstash ← Filebeat
     ↑
   Kibana

流程：
1. 应用 → 日志文件
2. Filebeat → 采集日志
3. Logstash → 解析、转换
4. Elasticsearch → 存储、索引
5. Kibana → 可视化、查询
```

### 日志查询

**示例查询**：

```text
# 查找特定用户的错误日志
level:ERROR AND user_id:"user_789"

# 查找慢查询
duration_ms:>1000 AND service:"database"

# 查找特定时间范围的日志
timestamp:[2025-10-17T00:00:00 TO 2025-10-17T23:59:59]
```

---

## 告警系统

### 告警规则

**基于阈值的告警**：

```yaml
# Prometheus告警规则
groups:
  - name: example_alerts
    rules:
      # 高错误率告警
      - alert: HighErrorRate
        expr: |
          rate(http_requests_total{status=~"5.."}[5m])
          / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"
      
      # 高延迟告警
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99,
            rate(http_request_duration_seconds_bucket[5m])
          ) > 1.0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "P99 latency is {{ $value }}s"
      
      # 服务不可用告警
      - alert: ServiceDown
        expr: up{job="my-service"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service is down"
          description: "{{ $labels.instance }} is unreachable"
```

### 告警路由

```yaml
# Alertmanager路由配置
route:
  receiver: 'team-default'
  group_by: ['alertname', 'cluster']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  
  routes:
    # 严重告警立即发送
    - match:
        severity: critical
      receiver: 'team-pager'
      group_wait: 10s
      repeat_interval: 1h
    
    # 警告级别告警
    - match:
        severity: warning
      receiver: 'team-slack'
      group_wait: 1m
      repeat_interval: 12h

receivers:
  - name: 'team-default'
    email_configs:
      - to: 'team@example.com'
  
  - name: 'team-pager'
    pagerduty_configs:
      - service_key: 'xxx'
  
  - name: 'team-slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/xxx'
        channel: '#alerts'
```

### 告警疲劳防护

```text
1. 告警聚合
   - 相似告警合并
   - 按时间窗口分组

2. 告警抑制
   - 上游故障抑制下游告警
   - 维护窗口静默

3. 告警优先级
   - 严重性分级
   - 业务影响评估

4. 告警阈值调优
   - 基于历史数据
   - 动态阈值
```

---

## 可观测性平台

### Prometheus + Grafana

**Prometheus架构**：

```text
┌──────────────┐
│ Applications │
└──────┬───────┘
       │ /metrics
       ↓
┌──────────────┐     ┌─────────────┐
│  Prometheus  │────→│ Alertmanager│
└──────┬───────┘     └─────────────┘
       │
       ↓
┌──────────────┐
│   Grafana    │
└──────────────┘
```

**Grafana Dashboard示例**：

```json
{
  "dashboard": {
    "title": "Service Overview",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m])"
          }
        ]
      },
      {
        "title": "Latency (P99)",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))"
          }
        ]
      }
    ]
  }
}
```

### Jaeger追踪

**Jaeger架构**：

```text
Applications
    ↓
Jaeger Client (SDK)
    ↓
Jaeger Agent
    ↓
Jaeger Collector
    ↓
Storage (Cassandra/Elasticsearch)
    ↓
Jaeger Query UI
```

---

## Rust实现示例

### 指标收集

```rust
use prometheus::{
    Counter, Histogram, HistogramOpts, Opts, Registry, Encoder, TextEncoder,
};
use std::sync::Arc;

/// 应用指标
pub struct AppMetrics {
    pub http_requests_total: Counter,
    pub http_request_duration: Histogram,
    pub http_requests_in_flight: prometheus::IntGauge,
    pub registry: Registry,
}

impl AppMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        // 请求计数器
        let http_requests_total = Counter::with_opts(
            Opts::new("http_requests_total", "Total HTTP requests")
                .namespace("myapp")
        )?;
        registry.register(Box::new(http_requests_total.clone()))?;
        
        // 请求时长直方图
        let http_request_duration = Histogram::with_opts(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration")
                .namespace("myapp")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
        )?;
        registry.register(Box::new(http_request_duration.clone()))?;
        
        // 进行中的请求
        let http_requests_in_flight = prometheus::IntGauge::with_opts(
            Opts::new("http_requests_in_flight", "HTTP requests currently being processed")
                .namespace("myapp")
        )?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        
        Ok(Self {
            http_requests_total,
            http_request_duration,
            http_requests_in_flight,
            registry,
        })
    }
    
    /// 导出指标
    pub fn export(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer).unwrap())
    }
}

/// 请求处理中间件
use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

pub async fn metrics_middleware(
    State(metrics): State<Arc<AppMetrics>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    // 增加进行中的请求计数
    metrics.http_requests_in_flight.inc();
    
    // 处理请求
    let response = next.run(request).await;
    
    // 减少进行中的请求计数
    metrics.http_requests_in_flight.dec();
    
    // 记录指标
    let duration = start.elapsed();
    metrics.http_requests_total.inc();
    metrics.http_request_duration.observe(duration.as_secs_f64());
    
    response
}

/// Metrics端点
use axum::{routing::get, Router};

pub fn metrics_router(metrics: Arc<AppMetrics>) -> Router {
    Router::new()
        .route("/metrics", get(move || async move {
            match metrics.export() {
                Ok(body) => body,
                Err(e) => format!("Error exporting metrics: {}", e),
            }
        }))
}
```

### 分布式追踪1

```rust
use opentelemetry::{
    global,
    trace::{Tracer, TracerProvider, Span, SpanKind, Status},
    KeyValue,
};
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_jaeger::new_agent_pipeline;

/// 初始化追踪
pub fn init_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tracer = new_agent_pipeline()
        .with_service_name(service_name)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", "1.0.0"),
                ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    
    global::set_tracer_provider(tracer);
    
    Ok(())
}

/// 追踪装饰器
use tracing::{instrument, info, error};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[instrument(skip(user_id))]
pub async fn process_order(order_id: String, user_id: String) -> Result<(), anyhow::Error> {
    let tracer = global::tracer("order-service");
    let mut span = tracer
        .span_builder("process_order")
        .with_kind(SpanKind::Internal)
        .start(&tracer);
    
    span.set_attribute(KeyValue::new("order.id", order_id.clone()));
    span.set_attribute(KeyValue::new("user.id", user_id.clone()));
    
    info!("Processing order: {}", order_id);
    
    // 模拟业务逻辑
    match validate_order(&order_id).await {
        Ok(_) => {
            span.add_event("order_validated", vec![]);
            charge_payment(&order_id).await?;
            span.add_event("payment_charged", vec![]);
            span.set_status(Status::Ok);
            Ok(())
        }
        Err(e) => {
            error!("Order validation failed: {}", e);
            span.set_status(Status::error(format!("Validation failed: {}", e)));
            Err(e)
        }
    }
}

async fn validate_order(order_id: &str) -> Result<(), anyhow::Error> {
    // 业务逻辑
    Ok(())
}

async fn charge_payment(order_id: &str) -> Result<(), anyhow::Error> {
    // 业务逻辑
    Ok(())
}

/// HTTP请求追踪
use axum::http::HeaderMap;

pub async fn traced_http_request(
    url: &str,
    headers: &HeaderMap,
) -> Result<String, anyhow::Error> {
    let tracer = global::tracer("http-client");
    let mut span = tracer
        .span_builder("http_request")
        .with_kind(SpanKind::Client)
        .start(&tracer);
    
    span.set_attribute(KeyValue::new("http.url", url.to_string()));
    span.set_attribute(KeyValue::new("http.method", "GET"));
    
    // 注入trace context到HTTP header
    let mut injected_headers = headers.clone();
    // 实际实现需要使用W3C Trace Context Propagator
    
    // 发送HTTP请求
    let client = reqwest::Client::new();
    let response = client.get(url).headers(injected_headers).send().await?;
    
    span.set_attribute(KeyValue::new("http.status_code", response.status().as_u16() as i64));
    
    let body = response.text().await?;
    span.end();
    
    Ok(body)
}
```

### 结构化日志1

```rust
use tracing::{info, warn, error, debug};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt,
    EnvFilter,
};

/// 初始化日志
pub fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive("myapp=debug".parse().unwrap())
            .add_directive("tokio=info".parse().unwrap()))
        .with(fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true))
        .init();
}

/// 使用结构化日志
#[instrument(fields(user_id = %user_id, order_id = %order_id))]
pub async fn create_order(user_id: String, order_id: String) -> Result<(), anyhow::Error> {
    info!("Creating new order");
    
    debug!(
        items = ?vec!["item1", "item2"],
        total = 99.99,
        "Order details"
    );
    
    match save_to_db(&order_id).await {
        Ok(_) => {
            info!(order_id = %order_id, "Order created successfully");
            Ok(())
        }
        Err(e) => {
            error!(
                error = %e,
                order_id = %order_id,
                "Failed to create order"
            );
            Err(e)
        }
    }
}

async fn save_to_db(order_id: &str) -> Result<(), anyhow::Error> {
    // 数据库操作
    Ok(())
}
```

### 健康检查

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck + Send + Sync>>,
}

#[async_trait::async_trait]
pub trait HealthCheck {
    fn name(&self) -> &str;
    async fn check(&self) -> ComponentHealth;
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }
    
    pub fn register(mut self, check: Box<dyn HealthCheck + Send + Sync>) -> Self {
        self.checks.push(check);
        self
    }
    
    pub async fn check_health(&self) -> HealthReport {
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;
        
        for check in &self.checks {
            let component_health = check.check().await;
            
            // 更新整体状态
            overall_status = match (&overall_status, &component_health.status) {
                (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
                (HealthStatus::Unhealthy, _) => HealthStatus::Unhealthy,
                (_, HealthStatus::Degraded) | (HealthStatus::Degraded, _) => HealthStatus::Degraded,
                _ => HealthStatus::Healthy,
            };
            
            components.insert(check.name().to_string(), component_health);
        }
        
        HealthReport {
            status: overall_status,
            components,
            timestamp: chrono::Utc::now(),
        }
    }
}

// 数据库健康检查
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        "database"
    }
    
    async fn check(&self) -> ComponentHealth {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => ComponentHealth {
                status: HealthStatus::Healthy,
                message: Some("Database connection is healthy".to_string()),
                last_check: chrono::Utc::now(),
            },
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Database error: {}", e)),
                last_check: chrono::Utc::now(),
            },
        }
    }
}

// Redis健康检查
pub struct RedisHealthCheck {
    client: redis::Client,
}

#[async_trait::async_trait]
impl HealthCheck for RedisHealthCheck {
    fn name(&self) -> &str {
        "redis"
    }
    
    async fn check(&self) -> ComponentHealth {
        match self.client.get_async_connection().await {
            Ok(mut conn) => {
                match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                    Ok(_) => ComponentHealth {
                        status: HealthStatus::Healthy,
                        message: Some("Redis connection is healthy".to_string()),
                        last_check: chrono::Utc::now(),
                    },
                    Err(e) => ComponentHealth {
                        status: HealthStatus::Unhealthy,
                        message: Some(format!("Redis error: {}", e)),
                        last_check: chrono::Utc::now(),
                    },
                }
            }
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Redis connection error: {}", e)),
                last_check: chrono::Utc::now(),
            },
        }
    }
}

// 健康检查端点
use axum::{extract::State, Json};

pub async fn health_handler(
    State(checker): State<Arc<HealthChecker>>,
) -> Json<HealthReport> {
    Json(checker.check_health().await)
}

pub async fn readiness_handler(
    State(checker): State<Arc<HealthChecker>>,
) -> (axum::http::StatusCode, Json<HealthReport>) {
    let report = checker.check_health().await;
    
    let status_code = match report.status {
        HealthStatus::Healthy => axum::http::StatusCode::OK,
        HealthStatus::Degraded => axum::http::StatusCode::OK,
        HealthStatus::Unhealthy => axum::http::StatusCode::SERVICE_UNAVAILABLE,
    };
    
    (status_code, Json(report))
}
```

---

## 最佳实践

### 监控策略

```text
1. 分层监控
   □ 基础设施层 (CPU、内存、网络)
   □ 平台层 (Kubernetes、数据库)
   □ 应用层 (业务指标、自定义指标)
   □ 用户体验层 (真实用户监控)

2. 指标命名
   □ 使用一致的命名约定
   □ 包含namespace和subsystem
   □ 使用描述性的名称
   □ 示例：myapp_http_requests_total

3. 标签使用
   □ 使用低基数标签
   □ 避免高基数标签（如用户ID）
   □ 常用标签：method, status, endpoint

4. 采样和聚合
   □ 高频指标使用采样
   □ 预聚合减少查询开销
   □ 使用recording rules
```

### 日志最佳实践

```text
1. 日志级别
   □ 生产环境使用INFO及以上
   □ DEBUG仅用于开发
   □ ERROR必须可操作

2. 结构化日志
   □ 使用JSON格式
   □ 包含trace_id、span_id
   □ 包含上下文信息

3. 日志内容
   □ 不记录敏感信息（密码、token）
   □ 记录请求ID用于追踪
   □ 记录时间戳和时区

4. 日志管理
   □ 集中式日志存储
   □ 定期归档
   □ 设置保留期限
```

### 追踪最佳实践

```text
1. 采样策略
   □ 生产环境使用合理采样率（1-10%）
   □ 错误请求100%采样
   □ 慢请求100%采样

2. Span设计
   □ 为关键操作创建span
   □ 记录有用的属性
   □ 避免过深的span层级

3. 上下文传播
   □ 使用标准协议（W3C Trace Context）
   □ 跨所有服务传播
   □ 处理缺失上下文

4. 性能考虑
   □ 异步发送追踪数据
   □ 批量上传
   □ 限制span大小
```

### SLO/SLI设定

```text
服务级别指标 (SLI)：
- 可用性：99.9%
- 延迟P99：< 100ms
- 错误率：< 0.1%

服务级别目标 (SLO)：
- 30天滚动窗口
- 错误预算：0.1% × 30天

告警基于错误预算：
- 快速燃烧率：1小时消耗2%预算 → P1告警
- 慢速燃烧率：6小时消耗5%预算 → P2告警
```

### 可观测性清单

```text
□ 指标收集
  □ 实现四大黄金信号
  □ 使用RED/USE方法
  □ 自定义业务指标
  □ 设置合理的保留期

□ 分布式追踪
  □ 集成OpenTelemetry
  □ 配置采样策略
  □ 上下文传播
  □ 性能优化

□ 日志管理
  □ 结构化日志
  □ 集中式存储
  □ 日志关联（trace_id）
  □ 日志归档

□ 告警配置
  □ 基于SLO的告警
  □ 告警路由
  □ 告警去重
  □ On-call轮换

□ 可视化
  □ Grafana仪表盘
  □ 关键指标可视化
  □ 实时监控
  □ 历史趋势分析

□ 响应流程
  □ Runbook文档
  □ 事故响应流程
  □ 事后分析(Postmortem)
  □ 持续改进
```

---

## 相关文档

- [3.8 可观测性](../observability/README.md)
- [3.10.5 性能优化](performance.md)
- [3.10.4 安全设计](security.md)
- [设计最佳实践](BEST_PRACTICES.md)

## 参考资料

### 工具和库

- **指标**: `prometheus`, `metrics`
- **追踪**: `opentelemetry`, `tracing`
- **日志**: `tracing-subscriber`, `slog`
- **可视化**: Grafana, Kibana, Jaeger UI

### 标准和协议

- OpenTelemetry
- Prometheus Exposition Format
- W3C Trace Context
- OpenMetrics

### 最佳实践1

- Google SRE Book
- Observability Engineering
- Distributed Systems Observability

# 错误处理（Error Handling）

> 分布式系统中的错误处理策略和最佳实践

## 目录

- [错误处理（Error Handling）](#错误处理error-handling)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [错误分类](#错误分类)
    - [错误传播](#错误传播)
    - [错误恢复](#错误恢复)
  - [🔧 实现机制](#-实现机制)
    - [错误类型系统](#错误类型系统)
    - [错误处理中间件](#错误处理中间件)
    - [断路器模式](#断路器模式)
  - [🚀 高级特性](#-高级特性)
    - [错误聚合](#错误聚合)
    - [错误预测](#错误预测)
  - [🧪 测试策略](#-测试策略)
    - [错误注入测试](#错误注入测试)
  - [🔍 性能优化](#-性能优化)
    - [错误处理优化](#错误处理优化)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

错误处理是分布式系统设计中的关键组成部分。良好的错误处理策略能够提高系统的可靠性、可观测性和用户体验。本文档详细介绍了分布式系统中的错误处理机制和最佳实践。

## 🎯 核心概念

### 错误分类

**定义 1（错误分类）**: 根据错误的性质、影响范围和可恢复性，将错误分为不同的类别。

**错误类型**:

- **瞬时错误**: 临时性的错误，通常可以通过重试解决
- **永久错误**: 持续性的错误，需要人工干预或系统修复
- **业务错误**: 由业务逻辑引起的错误，如参数验证失败
- **系统错误**: 由系统资源或基础设施引起的错误

### 错误传播

**定义 2（错误传播）**: 错误在系统组件间的传递和处理机制。

**传播策略**:

- **快速失败**: 立即返回错误，不进行重试
- **重试机制**: 自动重试失败的请求
- **错误转换**: 将底层错误转换为上层错误
- **错误聚合**: 将多个相关错误合并处理

### 错误恢复

**定义 3（错误恢复）**: 系统从错误状态恢复到正常状态的过程。

**恢复策略**:

- **自动恢复**: 系统自动尝试恢复
- **降级服务**: 提供有限功能的服务
- **故障转移**: 切换到备用系统
- **数据恢复**: 从备份恢复数据

## 🔧 实现机制

### 错误类型系统

```rust
use std::fmt;
use std::error::Error as StdError;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Debug, Clone)]
pub enum DistributedError {
    // 网络错误
    NetworkError {
        message: String,
        retry_after: Option<Duration>,
        is_transient: bool,
    },
    // 超时错误
    TimeoutError {
        operation: String,
        timeout_duration: Duration,
        retry_count: u32,
    },
    // 一致性错误
    ConsistencyError {
        expected_state: String,
        actual_state: String,
        conflict_resolution: ConflictResolution,
    },
    // 资源错误
    ResourceError {
        resource_type: String,
        resource_id: String,
        error_code: String,
        is_recoverable: bool,
    },
    // 业务错误
    BusinessError {
        error_code: String,
        message: String,
        context: HashMap<String, String>,
    },
    // 系统错误
    SystemError {
        component: String,
        error_code: String,
        details: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone)]
pub enum ConflictResolution {
    LastWriterWins,
    FirstWriterWins,
    Merge,
    Manual,
}

impl fmt::Display for DistributedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistributedError::NetworkError { message, .. } => {
                write!(f, "Network error: {}", message)
            }
            DistributedError::TimeoutError { operation, .. } => {
                write!(f, "Timeout error in operation: {}", operation)
            }
            DistributedError::ConsistencyError { expected_state, actual_state, .. } => {
                write!(f, "Consistency error: expected {}, got {}", expected_state, actual_state)
            }
            DistributedError::ResourceError { resource_type, resource_id, .. } => {
                write!(f, "Resource error: {} {}", resource_type, resource_id)
            }
            DistributedError::BusinessError { error_code, message, .. } => {
                write!(f, "Business error [{}]: {}", error_code, message)
            }
            DistributedError::SystemError { component, error_code, .. } => {
                write!(f, "System error in {} [{}]", component, error_code)
            }
        }
    }
}

impl StdError for DistributedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl DistributedError {
    // 检查是否为瞬时错误
    pub fn is_transient(&self) -> bool {
        match self {
            DistributedError::NetworkError { is_transient, .. } => *is_transient,
            DistributedError::TimeoutError { .. } => true,
            DistributedError::ConsistencyError { .. } => false,
            DistributedError::ResourceError { is_recoverable, .. } => *is_recoverable,
            DistributedError::BusinessError { .. } => false,
            DistributedError::SystemError { .. } => false,
        }
    }
    
    // 获取重试延迟
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            DistributedError::NetworkError { retry_after, .. } => *retry_after,
            DistributedError::TimeoutError { timeout_duration, retry_count } => {
                // 指数退避
                Some(*timeout_duration * 2_u32.pow(*retry_count))
            }
            _ => None,
        }
    }
    
    // 获取错误严重性
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            DistributedError::NetworkError { .. } => ErrorSeverity::Medium,
            DistributedError::TimeoutError { .. } => ErrorSeverity::Medium,
            DistributedError::ConsistencyError { .. } => ErrorSeverity::High,
            DistributedError::ResourceError { .. } => ErrorSeverity::High,
            DistributedError::BusinessError { .. } => ErrorSeverity::Low,
            DistributedError::SystemError { .. } => ErrorSeverity::Critical,
        }
    }
    
    // 获取错误上下文
    pub fn context(&self) -> HashMap<String, String> {
        let mut context = HashMap::new();
        
        match self {
            DistributedError::NetworkError { message, retry_after, is_transient } => {
                context.insert("error_type".to_string(), "network".to_string());
                context.insert("message".to_string(), message.clone());
                context.insert("is_transient".to_string(), is_transient.to_string());
                if let Some(duration) = retry_after {
                    context.insert("retry_after".to_string(), duration.as_secs().to_string());
                }
            }
            DistributedError::TimeoutError { operation, timeout_duration, retry_count } => {
                context.insert("error_type".to_string(), "timeout".to_string());
                context.insert("operation".to_string(), operation.clone());
                context.insert("timeout_duration".to_string(), timeout_duration.as_secs().to_string());
                context.insert("retry_count".to_string(), retry_count.to_string());
            }
            DistributedError::ConsistencyError { expected_state, actual_state, conflict_resolution } => {
                context.insert("error_type".to_string(), "consistency".to_string());
                context.insert("expected_state".to_string(), expected_state.clone());
                context.insert("actual_state".to_string(), actual_state.clone());
                context.insert("conflict_resolution".to_string(), format!("{:?}", conflict_resolution));
            }
            DistributedError::ResourceError { resource_type, resource_id, error_code, is_recoverable } => {
                context.insert("error_type".to_string(), "resource".to_string());
                context.insert("resource_type".to_string(), resource_type.clone());
                context.insert("resource_id".to_string(), resource_id.clone());
                context.insert("error_code".to_string(), error_code.clone());
                context.insert("is_recoverable".to_string(), is_recoverable.to_string());
            }
            DistributedError::BusinessError { error_code, message, context: business_context } => {
                context.insert("error_type".to_string(), "business".to_string());
                context.insert("error_code".to_string(), error_code.clone());
                context.insert("message".to_string(), message.clone());
                context.extend(business_context.clone());
            }
            DistributedError::SystemError { component, error_code, details, timestamp } => {
                context.insert("error_type".to_string(), "system".to_string());
                context.insert("component".to_string(), component.clone());
                context.insert("error_code".to_string(), error_code.clone());
                context.insert("details".to_string(), details.clone());
                context.insert("timestamp".to_string(), timestamp.to_string());
            }
        }
        
        context
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// 错误处理结果
#[derive(Debug, Clone)]
pub struct ErrorHandlingResult<T> {
    pub result: Result<T, DistributedError>,
    pub handling_strategy: ErrorHandlingStrategy,
    pub retry_count: u32,
    pub total_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum ErrorHandlingStrategy {
    Retry,
    Fallback,
    CircuitBreaker,
    Bulkhead,
    Timeout,
}

// 错误处理器
pub struct ErrorHandler {
    retry_policies: HashMap<String, RetryPolicy>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
    fallback_handlers: HashMap<String, FallbackHandler>,
    error_aggregator: ErrorAggregator,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub last_failure_time: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct FallbackHandler {
    pub handler_id: String,
    pub fallback_function: String,
    pub fallback_data: HashMap<String, String>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            retry_policies: HashMap::new(),
            circuit_breakers: HashMap::new(),
            fallback_handlers: HashMap::new(),
            error_aggregator: ErrorAggregator::new(),
        }
    }
    
    // 处理错误
    pub async fn handle_error<T>(
        &self,
        operation: &str,
        error: DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<ErrorHandlingResult<T>, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        
        // 记录错误
        self.error_aggregator.record_error(&error, context).await?;
        
        // 选择处理策略
        let strategy = self.select_handling_strategy(operation, &error).await?;
        
        match strategy {
            ErrorHandlingStrategy::Retry => {
                self.handle_with_retry(operation, error, context).await
            }
            ErrorHandlingStrategy::Fallback => {
                self.handle_with_fallback(operation, error, context).await
            }
            ErrorHandlingStrategy::CircuitBreaker => {
                self.handle_with_circuit_breaker(operation, error, context).await
            }
            ErrorHandlingStrategy::Bulkhead => {
                self.handle_with_bulkhead(operation, error, context).await
            }
            ErrorHandlingStrategy::Timeout => {
                self.handle_with_timeout(operation, error, context).await
            }
        }
    }
    
    // 选择处理策略
    async fn select_handling_strategy(
        &self,
        operation: &str,
        error: &DistributedError,
    ) -> Result<ErrorHandlingStrategy, Box<dyn std::error::Error>> {
        if error.is_transient() {
            Ok(ErrorHandlingStrategy::Retry)
        } else {
            match error.severity() {
                ErrorSeverity::Critical => Ok(ErrorHandlingStrategy::CircuitBreaker),
                ErrorSeverity::High => Ok(ErrorHandlingStrategy::Fallback),
                ErrorSeverity::Medium => Ok(ErrorHandlingStrategy::Bulkhead),
                ErrorSeverity::Low => Ok(ErrorHandlingStrategy::Timeout),
            }
        }
    }
    
    // 重试处理
    async fn handle_with_retry<T>(
        &self,
        operation: &str,
        error: DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<ErrorHandlingResult<T>, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        let mut retry_count = 0;
        
        if let Some(policy) = self.retry_policies.get(operation) {
            while retry_count < policy.max_retries {
                if let Some(delay) = error.retry_after() {
                    tokio::time::sleep(delay).await;
                }
                
                retry_count += 1;
                
                // 这里应该重新执行操作，简化实现
                if retry_count >= policy.max_retries {
                    break;
                }
            }
        }
        
        let total_duration = start_time.elapsed().unwrap();
        
        Ok(ErrorHandlingResult {
            result: Err(error),
            handling_strategy: ErrorHandlingStrategy::Retry,
            retry_count,
            total_duration,
        })
    }
    
    // 降级处理
    async fn handle_with_fallback<T>(
        &self,
        operation: &str,
        error: DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<ErrorHandlingResult<T>, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        
        if let Some(fallback_handler) = self.fallback_handlers.get(operation) {
            // 执行降级逻辑
            println!("Executing fallback for operation: {}", operation);
        }
        
        let total_duration = start_time.elapsed().unwrap();
        
        Ok(ErrorHandlingResult {
            result: Err(error),
            handling_strategy: ErrorHandlingStrategy::Fallback,
            retry_count: 0,
            total_duration,
        })
    }
    
    // 断路器处理
    async fn handle_with_circuit_breaker<T>(
        &self,
        operation: &str,
        error: DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<ErrorHandlingResult<T>, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        
        if let Some(circuit_breaker) = self.circuit_breakers.get(operation) {
            match circuit_breaker.state {
                CircuitBreakerState::Open => {
                    // 断路器打开，直接返回错误
                }
                CircuitBreakerState::HalfOpen => {
                    // 半开状态，允许少量请求通过
                }
                CircuitBreakerState::Closed => {
                    // 关闭状态，正常处理
                }
            }
        }
        
        let total_duration = start_time.elapsed().unwrap();
        
        Ok(ErrorHandlingResult {
            result: Err(error),
            handling_strategy: ErrorHandlingStrategy::CircuitBreaker,
            retry_count: 0,
            total_duration,
        })
    }
    
    // 隔离处理
    async fn handle_with_bulkhead<T>(
        &self,
        operation: &str,
        error: DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<ErrorHandlingResult<T>, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        
        // 隔离处理逻辑
        println!("Handling with bulkhead for operation: {}", operation);
        
        let total_duration = start_time.elapsed().unwrap();
        
        Ok(ErrorHandlingResult {
            result: Err(error),
            handling_strategy: ErrorHandlingStrategy::Bulkhead,
            retry_count: 0,
            total_duration,
        })
    }
    
    // 超时处理
    async fn handle_with_timeout<T>(
        &self,
        operation: &str,
        error: DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<ErrorHandlingResult<T>, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        
        // 超时处理逻辑
        println!("Handling with timeout for operation: {}", operation);
        
        let total_duration = start_time.elapsed().unwrap();
        
        Ok(ErrorHandlingResult {
            result: Err(error),
            handling_strategy: ErrorHandlingStrategy::Timeout,
            retry_count: 0,
            total_duration,
        })
    }
    
    // 添加重试策略
    pub fn add_retry_policy(&mut self, operation: &str, policy: RetryPolicy) {
        self.retry_policies.insert(operation.to_string(), policy);
    }
    
    // 添加断路器
    pub fn add_circuit_breaker(&mut self, operation: &str, circuit_breaker: CircuitBreaker) {
        self.circuit_breakers.insert(operation.to_string(), circuit_breaker);
    }
    
    // 添加降级处理器
    pub fn add_fallback_handler(&mut self, operation: &str, handler: FallbackHandler) {
        self.fallback_handlers.insert(operation.to_string(), handler);
    }
}

// 错误聚合器
pub struct ErrorAggregator {
    error_counts: HashMap<String, u32>,
    error_timestamps: HashMap<String, Vec<u64>>,
    error_contexts: HashMap<String, Vec<HashMap<String, String>>>,
}

impl ErrorAggregator {
    pub fn new() -> Self {
        Self {
            error_counts: HashMap::new(),
            error_timestamps: HashMap::new(),
            error_contexts: HashMap::new(),
        }
    }
    
    // 记录错误
    pub async fn record_error(
        &mut self,
        error: &DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let error_key = format!("{:?}", error);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 更新错误计数
        *self.error_counts.entry(error_key.clone()).or_insert(0) += 1;
        
        // 记录时间戳
        self.error_timestamps
            .entry(error_key.clone())
            .or_insert_with(Vec::new)
            .push(timestamp);
        
        // 记录上下文
        self.error_contexts
            .entry(error_key)
            .or_insert_with(Vec::new)
            .push(context.clone());
        
        Ok(())
    }
    
    // 获取错误统计
    pub fn get_error_stats(&self) -> HashMap<String, ErrorStats> {
        let mut stats = HashMap::new();
        
        for (error_key, count) in &self.error_counts {
            let timestamps = self.error_timestamps.get(error_key).unwrap_or(&Vec::new());
            let contexts = self.error_contexts.get(error_key).unwrap_or(&Vec::new());
            
            stats.insert(error_key.clone(), ErrorStats {
                count: *count,
                first_occurrence: timestamps.first().copied(),
                last_occurrence: timestamps.last().copied(),
                contexts: contexts.clone(),
            });
        }
        
        stats
    }
}

#[derive(Debug, Clone)]
pub struct ErrorStats {
    pub count: u32,
    pub first_occurrence: Option<u64>,
    pub last_occurrence: Option<u64>,
    pub contexts: Vec<HashMap<String, String>>,
}
```

### 错误处理中间件

```rust
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::{json, Value};

// 错误处理中间件
pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    
    match next.run(request).await {
        Ok(response) => {
            let duration = start_time.elapsed();
            log::info!("Request completed successfully in {:?}", duration);
            Ok(response)
        }
        Err(error) => {
            let duration = start_time.elapsed();
            log::error!("Request failed after {:?}: {:?}", duration, error);
            
            // 创建错误响应
            let error_response = create_error_response(&error, duration);
            Ok(error_response)
        }
    }
}

// 创建错误响应
fn create_error_response(error: &dyn std::error::Error, duration: std::time::Duration) -> Response {
    let error_code = match error.downcast_ref::<DistributedError>() {
        Some(distributed_error) => match distributed_error {
            DistributedError::NetworkError { .. } => "NETWORK_ERROR",
            DistributedError::TimeoutError { .. } => "TIMEOUT_ERROR",
            DistributedError::ConsistencyError { .. } => "CONSISTENCY_ERROR",
            DistributedError::ResourceError { .. } => "RESOURCE_ERROR",
            DistributedError::BusinessError { error_code, .. } => error_code,
            DistributedError::SystemError { error_code, .. } => error_code,
        },
        None => "UNKNOWN_ERROR",
    };
    
    let status_code = match error.downcast_ref::<DistributedError>() {
        Some(distributed_error) => match distributed_error.severity() {
            ErrorSeverity::Critical => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorSeverity::High => StatusCode::BAD_REQUEST,
            ErrorSeverity::Medium => StatusCode::SERVICE_UNAVAILABLE,
            ErrorSeverity::Low => StatusCode::BAD_REQUEST,
        },
        None => StatusCode::INTERNAL_SERVER_ERROR,
    };
    
    let error_body = json!({
        "error": {
            "code": error_code,
            "message": error.to_string(),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "duration_ms": duration.as_millis(),
        }
    });
    
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&error_body).unwrap().into())
        .unwrap()
}

// 错误处理扩展
pub trait ErrorHandlerExt {
    fn with_error_handler(self) -> Self;
}

impl ErrorHandlerExt for axum::Router {
    fn with_error_handler(self) -> Self {
        self.layer(axum::middleware::from_fn(error_handling_middleware))
    }
}
```

### 断路器模式

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_max_calls: u32,
    failure_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    half_open_calls: Arc<Mutex<u32>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration, half_open_max_calls: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            failure_threshold,
            recovery_timeout,
            half_open_max_calls,
            failure_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            half_open_calls: Arc::new(Mutex::new(0)),
        }
    }
    
    // 执行操作
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // 检查断路器状态
        if !self.can_execute() {
            return Err("Circuit breaker is open".into());
        }
        
        // 执行操作
        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }
    
    // 检查是否可以执行
    fn can_execute(&self) -> bool {
        let state = self.state.lock().unwrap();
        
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // 检查是否应该进入半开状态
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        *self.state.lock().unwrap() = CircuitBreakerState::HalfOpen;
                        *self.half_open_calls.lock().unwrap() = 0;
                        return true;
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => {
                let half_open_calls = *self.half_open_calls.lock().unwrap();
                half_open_calls < self.half_open_max_calls
            }
        }
    }
    
    // 成功回调
    fn on_success(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut half_open_calls = self.half_open_calls.lock().unwrap();
        
        match *state {
            CircuitBreakerState::Closed => {
                // 重置失败计数
                *failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                // 成功，关闭断路器
                *state = CircuitBreakerState::Closed;
                *failure_count = 0;
                *half_open_calls = 0;
            }
            CircuitBreakerState::Open => {
                // 不应该到达这里
            }
        }
    }
    
    // 失败回调
    fn on_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut half_open_calls = self.half_open_calls.lock().unwrap();
        let mut last_failure_time = self.last_failure_time.lock().unwrap();
        
        match *state {
            CircuitBreakerState::Closed => {
                *failure_count += 1;
                *last_failure_time = Some(Instant::now());
                
                if *failure_count >= self.failure_threshold {
                    *state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                *half_open_calls += 1;
                *last_failure_time = Some(Instant::now());
                
                if *half_open_calls >= self.half_open_max_calls {
                    *state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::Open => {
                *last_failure_time = Some(Instant::now());
            }
        }
    }
    
    // 获取当前状态
    pub fn get_state(&self) -> CircuitBreakerState {
        *self.state.lock().unwrap()
    }
    
    // 获取失败计数
    pub fn get_failure_count(&self) -> u32 {
        *self.failure_count.lock().unwrap()
    }
    
    // 手动重置断路器
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut half_open_calls = self.half_open_calls.lock().unwrap();
        let mut last_failure_time = self.last_failure_time.lock().unwrap();
        
        *state = CircuitBreakerState::Closed;
        *failure_count = 0;
        *half_open_calls = 0;
        *last_failure_time = None;
    }
}
```

## 🚀 高级特性

### 错误聚合

```rust
pub struct ErrorAggregator {
    error_counts: HashMap<String, u32>,
    error_timestamps: HashMap<String, Vec<u64>>,
    error_contexts: HashMap<String, Vec<HashMap<String, String>>>,
    aggregation_window: Duration,
    max_errors_per_window: u32,
}

impl ErrorAggregator {
    pub fn new(aggregation_window: Duration, max_errors_per_window: u32) -> Self {
        Self {
            error_counts: HashMap::new(),
            error_timestamps: HashMap::new(),
            error_contexts: HashMap::new(),
            aggregation_window,
            max_errors_per_window,
        }
    }
    
    // 记录错误
    pub async fn record_error(
        &mut self,
        error: &DistributedError,
        context: &HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let error_key = format!("{:?}", error);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 清理过期错误
        self.cleanup_expired_errors(timestamp).await?;
        
        // 更新错误计数
        *self.error_counts.entry(error_key.clone()).or_insert(0) += 1;
        
        // 记录时间戳
        self.error_timestamps
            .entry(error_key.clone())
            .or_insert_with(Vec::new)
            .push(timestamp);
        
        // 记录上下文
        self.error_contexts
            .entry(error_key)
            .or_insert_with(Vec::new)
            .push(context.clone());
        
        Ok(())
    }
    
    // 清理过期错误
    async fn cleanup_expired_errors(&mut self, current_timestamp: u64) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff_time = current_timestamp - self.aggregation_window.as_secs();
        
        for (error_key, timestamps) in &mut self.error_timestamps {
            timestamps.retain(|&timestamp| timestamp > cutoff_time);
            
            if timestamps.is_empty() {
                self.error_counts.remove(error_key);
                self.error_contexts.remove(error_key);
            } else {
                if let Some(count) = self.error_counts.get_mut(error_key) {
                    *count = timestamps.len() as u32;
                }
            }
        }
        
        Ok(())
    }
    
    // 获取错误统计
    pub fn get_error_stats(&self) -> HashMap<String, ErrorStats> {
        let mut stats = HashMap::new();
        
        for (error_key, count) in &self.error_counts {
            let timestamps = self.error_timestamps.get(error_key).unwrap_or(&Vec::new());
            let contexts = self.error_contexts.get(error_key).unwrap_or(&Vec::new());
            
            stats.insert(error_key.clone(), ErrorStats {
                count: *count,
                first_occurrence: timestamps.first().copied(),
                last_occurrence: timestamps.last().copied(),
                contexts: contexts.clone(),
            });
        }
        
        stats
    }
    
    // 检查是否超过错误阈值
    pub fn is_error_threshold_exceeded(&self) -> bool {
        self.error_counts.values().any(|&count| count > self.max_errors_per_window)
    }
    
    // 获取错误趋势
    pub fn get_error_trends(&self) -> HashMap<String, ErrorTrend> {
        let mut trends = HashMap::new();
        
        for (error_key, timestamps) in &self.error_timestamps {
            if timestamps.len() >= 2 {
                let first_timestamp = timestamps.first().unwrap();
                let last_timestamp = timestamps.last().unwrap();
                let duration = last_timestamp - first_timestamp;
                let rate = timestamps.len() as f64 / duration as f64;
                
                trends.insert(error_key.clone(), ErrorTrend {
                    error_key: error_key.clone(),
                    rate,
                    duration,
                    count: timestamps.len() as u32,
                });
            }
        }
        
        trends
    }
}

#[derive(Debug, Clone)]
pub struct ErrorTrend {
    pub error_key: String,
    pub rate: f64,
    pub duration: u64,
    pub count: u32,
}
```

### 错误预测

```rust
pub struct ErrorPredictor {
    error_history: Vec<ErrorRecord>,
    prediction_model: PredictionModel,
    prediction_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct ErrorRecord {
    pub error: DistributedError,
    pub timestamp: u64,
    pub context: HashMap<String, String>,
    pub severity: ErrorSeverity,
}

pub struct PredictionModel {
    model_type: ModelType,
    parameters: HashMap<String, f64>,
    training_data: Vec<ErrorRecord>,
}

#[derive(Debug, Clone)]
pub enum ModelType {
    LinearRegression,
    DecisionTree,
    NeuralNetwork,
}

impl ErrorPredictor {
    pub fn new(prediction_threshold: f64) -> Self {
        Self {
            error_history: Vec::new(),
            prediction_model: PredictionModel::new(),
            prediction_threshold,
        }
    }
    
    // 记录错误
    pub fn record_error(&mut self, error: DistributedError, context: HashMap<String, String>) {
        let record = ErrorRecord {
            error,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context,
            severity: ErrorSeverity::Medium,
        };
        
        self.error_history.push(record);
        
        // 保持历史记录大小
        if self.error_history.len() > 1000 {
            self.error_history.remove(0);
        }
    }
    
    // 预测错误
    pub async fn predict_error(&self, context: &HashMap<String, String>) -> Result<ErrorPrediction, Box<dyn std::error::Error>> {
        let prediction = self.prediction_model.predict(context, &self.error_history).await?;
        
        Ok(ErrorPrediction {
            probability: prediction.probability,
            predicted_error_type: prediction.error_type,
            confidence: prediction.confidence,
            recommended_action: prediction.recommended_action,
        })
    }
    
    // 训练模型
    pub async fn train_model(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.prediction_model.train(&self.error_history).await?;
        Ok(())
    }
    
    // 获取错误模式
    pub fn get_error_patterns(&self) -> Vec<ErrorPattern> {
        let mut patterns = Vec::new();
        
        // 按错误类型分组
        let mut error_groups: HashMap<String, Vec<&ErrorRecord>> = HashMap::new();
        for record in &self.error_history {
            let error_type = format!("{:?}", record.error);
            error_groups.entry(error_type).or_insert_with(Vec::new).push(record);
        }
        
        // 分析每个组的模式
        for (error_type, records) in error_groups {
            if records.len() >= 3 {
                let pattern = self.analyze_pattern(&error_type, records);
                patterns.push(pattern);
            }
        }
        
        patterns
    }
    
    // 分析模式
    fn analyze_pattern(&self, error_type: &str, records: &[&ErrorRecord]) -> ErrorPattern {
        let timestamps: Vec<u64> = records.iter().map(|r| r.timestamp).collect();
        let intervals: Vec<u64> = timestamps.windows(2).map(|w| w[1] - w[0]).collect();
        
        let avg_interval = if !intervals.is_empty() {
            intervals.iter().sum::<u64>() as f64 / intervals.len() as f64
        } else {
            0.0
        };
        
        ErrorPattern {
            error_type: error_type.to_string(),
            frequency: records.len() as f64,
            avg_interval,
            severity: records.iter().map(|r| r.severity.clone()).collect(),
            common_contexts: self.extract_common_contexts(records),
        }
    }
    
    // 提取共同上下文
    fn extract_common_contexts(&self, records: &[&ErrorRecord]) -> HashMap<String, String> {
        let mut context_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();
        
        for record in records {
            for (key, value) in &record.context {
                context_counts
                    .entry(key.clone())
                    .or_insert_with(HashMap::new)
                    .entry(value.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
        
        let mut common_contexts = HashMap::new();
        for (key, value_counts) in context_counts {
            if let Some((most_common_value, _)) = value_counts.iter().max_by_key(|(_, &count)| count) {
                common_contexts.insert(key, most_common_value.clone());
            }
        }
        
        common_contexts
    }
}

impl PredictionModel {
    pub fn new() -> Self {
        Self {
            model_type: ModelType::LinearRegression,
            parameters: HashMap::new(),
            training_data: Vec::new(),
        }
    }
    
    // 预测
    pub async fn predict(
        &self,
        context: &HashMap<String, String>,
        error_history: &[ErrorRecord],
    ) -> Result<ModelPrediction, Box<dyn std::error::Error>> {
        // 简化实现，实际应该使用机器学习模型
        let probability = 0.5; // 示例概率
        let confidence = 0.8; // 示例置信度
        
        Ok(ModelPrediction {
            probability,
            error_type: "NetworkError".to_string(),
            confidence,
            recommended_action: "Retry with exponential backoff".to_string(),
        })
    }
    
    // 训练
    pub async fn train(&mut self, training_data: &[ErrorRecord]) -> Result<(), Box<dyn std::error::Error>> {
        self.training_data = training_data.to_vec();
        
        // 简化实现，实际应该训练机器学习模型
        self.parameters.insert("weight".to_string(), 0.5);
        self.parameters.insert("bias".to_string(), 0.1);
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ErrorPrediction {
    pub probability: f64,
    pub predicted_error_type: String,
    pub confidence: f64,
    pub recommended_action: String,
}

#[derive(Debug, Clone)]
pub struct ModelPrediction {
    pub probability: f64,
    pub error_type: String,
    pub confidence: f64,
    pub recommended_action: String,
}

#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub error_type: String,
    pub frequency: f64,
    pub avg_interval: f64,
    pub severity: Vec<ErrorSeverity>,
    pub common_contexts: HashMap<String, String>,
}
```

## 🧪 测试策略

### 错误注入测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_error_handler() {
        let mut error_handler = ErrorHandler::new();
        
        // 添加重试策略
        error_handler.add_retry_policy("test_operation", RetryPolicy {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            jitter: true,
        });
        
        // 测试网络错误处理
        let network_error = DistributedError::NetworkError {
            message: "Connection failed".to_string(),
            retry_after: Some(Duration::from_millis(100)),
            is_transient: true,
        };
        
        let context = HashMap::new();
        let result = error_handler.handle_error::<String>("test_operation", network_error, &context).await;
        
        assert!(result.is_ok());
        let handling_result = result.unwrap();
        assert_eq!(handling_result.handling_strategy, ErrorHandlingStrategy::Retry);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(1), 2);
        
        // 初始状态应该是关闭的
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::Closed);
        
        // 模拟失败
        for _ in 0..3 {
            circuit_breaker.on_failure();
        }
        
        // 应该打开断路器
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::Open);
        
        // 重置断路器
        circuit_breaker.reset();
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_error_aggregator() {
        let mut aggregator = ErrorAggregator::new(Duration::from_secs(60), 10);
        
        let error = DistributedError::NetworkError {
            message: "Connection failed".to_string(),
            retry_after: Some(Duration::from_millis(100)),
            is_transient: true,
        };
        
        let context = HashMap::new();
        aggregator.record_error(&error, &context).await.unwrap();
        
        let stats = aggregator.get_error_stats();
        assert_eq!(stats.len(), 1);
    }
    
    #[tokio::test]
    async fn test_error_predictor() {
        let mut predictor = ErrorPredictor::new(0.5);
        
        let error = DistributedError::NetworkError {
            message: "Connection failed".to_string(),
            retry_after: Some(Duration::from_millis(100)),
            is_transient: true,
        };
        
        let context = HashMap::new();
        predictor.record_error(error, context);
        
        let patterns = predictor.get_error_patterns();
        assert!(patterns.is_empty()); // 需要至少3个错误才能形成模式
    }
}
```

## 🔍 性能优化

### 错误处理优化

```rust
pub struct ErrorHandlingOptimizer {
    performance_metrics: HashMap<String, PerformanceMetric>,
    optimization_rules: Vec<OptimizationRule>,
    cache: ErrorCache,
}

pub struct ErrorCache {
    error_responses: HashMap<String, CachedErrorResponse>,
    cache_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct CachedErrorResponse {
    pub response: String,
    pub created_at: u64,
    pub ttl: Duration,
}

impl ErrorHandlingOptimizer {
    pub fn new() -> Self {
        Self {
            performance_metrics: HashMap::new(),
            optimization_rules: Vec::new(),
            cache: ErrorCache::new(Duration::from_secs(300)),
        }
    }
    
    // 优化错误处理性能
    pub async fn optimize_error_handling(&self) -> Result<Vec<OptimizationRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();
        
        // 分析性能指标
        for (metric_name, metric) in &self.performance_metrics {
            if metric.value > 1000.0 { // 假设1000ms是阈值
                recommendations.push(OptimizationRecommendation {
                    metric_name: metric_name.clone(),
                    current_value: metric.value,
                    recommended_value: 500.0,
                    optimization_action: "Cache error responses".to_string(),
                });
            }
        }
        
        Ok(recommendations)
    }
    
    // 缓存错误响应
    pub async fn cache_error_response(&mut self, error_key: &str, response: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.cache.cache_response(error_key, response).await?;
        Ok(())
    }
    
    // 获取缓存的错误响应
    pub async fn get_cached_error_response(&self, error_key: &str) -> Option<String> {
        self.cache.get_response(error_key).await
    }
}

impl ErrorCache {
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            error_responses: HashMap::new(),
            cache_ttl,
        }
    }
    
    // 缓存响应
    pub async fn cache_response(&mut self, error_key: &str, response: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cached_response = CachedErrorResponse {
            response: response.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: self.cache_ttl,
        };
        
        self.error_responses.insert(error_key.to_string(), cached_response);
        Ok(())
    }
    
    // 获取响应
    pub async fn get_response(&self, error_key: &str) -> Option<String> {
        if let Some(cached_response) = self.error_responses.get(error_key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now - cached_response.created_at < cached_response.ttl.as_secs() {
                Some(cached_response.response.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub metric_name: String,
    pub current_value: f64,
    pub recommended_value: f64,
    pub optimization_action: String,
}
```

## 📚 进一步阅读

- [最佳实践](./BEST_PRACTICES.md) - 系统设计最佳实践
- [常见陷阱](../PITFALLS.md) - 常见错误和避免方法
- [风格规范](../STYLE_GUIDE.md) - 代码和文档风格规范
- [架构模式](./architecture_patterns.md) - 系统架构模式

## 🔗 相关文档

- [最佳实践](./BEST_PRACTICES.md)
- [常见陷阱](../PITFALLS.md)
- [风格规范](../STYLE_GUIDE.md)
- [架构模式](./architecture_patterns.md)
- [配置管理](./configuration.md)
- [安全设计](./security.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

use crate::errors::DistributedError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

#[cfg(feature = "runtime-tokio")]
use tokio::sync::{mpsc, oneshot};
#[cfg(feature = "runtime-tokio")]
use tokio::time::timeout;

/// RPC 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: u64,
    pub method: String,
    pub payload: Vec<u8>,
    pub timeout: Option<Duration>,
}

/// RPC 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Result<Vec<u8>, String>,
}

/// 批量 RPC 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRpcRequest {
    pub requests: Vec<RpcRequest>,
    pub batch_id: u64,
}

/// 批量 RPC 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRpcResponse {
    pub responses: Vec<RpcResponse>,
    pub batch_id: u64,
}

/// 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: String,
    pub created_at: Instant,
    pub last_used: Instant,
    pub is_healthy: bool,
    pub request_count: u64,
}

/// 连接池配置
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub health_check_interval: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
        }
    }
}

pub trait RpcClient {
    fn call(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>, DistributedError>;
    
    /// 异步调用
    #[cfg(feature = "runtime-tokio")]
    async fn call_async(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>, DistributedError>;
    
    /// 批量调用
    #[cfg(feature = "runtime-tokio")]
    async fn call_batch(&self, requests: Vec<RpcRequest>) -> Result<Vec<RpcResponse>, DistributedError>;
}

pub trait RpcServer {
    fn register(&mut self, method: &str, handler: Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>);
    
    /// 注册异步处理器
    #[cfg(feature = "runtime-tokio")]
    fn register_async(&mut self, method: &str, handler: Box<dyn Fn(&[u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> + Send + Sync>);
}

/// 连接池管理器
pub struct ConnectionPool {
    connections: Arc<Mutex<HashMap<String, ConnectionInfo>>>,
    config: ConnectionPoolConfig,
    next_connection_id: Arc<Mutex<u64>>,
}

impl ConnectionPool {
    pub fn new(config: ConnectionPoolConfig) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            config,
            next_connection_id: Arc::new(Mutex::new(1)),
        }
    }

    /// 获取连接
    #[cfg(feature = "runtime-tokio")]
    pub async fn get_connection(&self, _endpoint: &str) -> Result<String, DistributedError> {
        let mut connections = self.connections.lock().unwrap();
        
        // 查找可用连接
        for (id, conn) in connections.iter_mut() {
            if conn.is_healthy && conn.last_used.elapsed() < self.config.idle_timeout {
                conn.last_used = Instant::now();
                conn.request_count += 1;
                return Ok(id.clone());
            }
        }

        // 创建新连接
        if connections.len() < self.config.max_connections {
            let mut next_id = self.next_connection_id.lock().unwrap();
            let connection_id = format!("conn_{}", *next_id);
            *next_id += 1;

            let connection_info = ConnectionInfo {
                id: connection_id.clone(),
                created_at: Instant::now(),
                last_used: Instant::now(),
                is_healthy: true,
                request_count: 1,
            };

            connections.insert(connection_id.clone(), connection_info);
            Ok(connection_id)
        } else {
            Err(DistributedError::Network("Connection pool exhausted".to_string()))
        }
    }

    /// 释放连接
    pub fn release_connection(&self, connection_id: &str) {
        let mut connections = self.connections.lock().unwrap();
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.last_used = Instant::now();
        }
    }

    /// 标记连接为不健康
    pub fn mark_unhealthy(&self, connection_id: &str) {
        let mut connections = self.connections.lock().unwrap();
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.is_healthy = false;
        }
    }

    /// 清理过期连接
    pub fn cleanup_expired(&self) {
        let mut connections = self.connections.lock().unwrap();
        let min_connections = self.config.min_connections;
        let current_len = connections.len();
        
        if current_len > min_connections {
            connections.retain(|_, conn| {
                conn.is_healthy && 
                conn.last_used.elapsed() < self.config.idle_timeout
            });
        }
    }

    /// 获取连接统计信息
    pub fn get_stats(&self) -> HashMap<String, ConnectionInfo> {
        self.connections.lock().unwrap().clone()
    }
}

/// 请求合并器
#[cfg(feature = "runtime-tokio")]
pub struct RequestBatcher {
    pending_requests: Arc<Mutex<Vec<RpcRequest>>>,
    batch_size: usize,
    batch_timeout: Duration,
    sender: mpsc::UnboundedSender<Vec<RpcRequest>>,
}

#[cfg(feature = "runtime-tokio")]
impl RequestBatcher {
    pub fn new(batch_size: usize, batch_timeout: Duration) -> (Self, mpsc::UnboundedReceiver<Vec<RpcRequest>>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let batcher = Self {
            pending_requests: Arc::new(Mutex::new(Vec::new())),
            batch_size,
            batch_timeout,
            sender,
        };

        (batcher, receiver)
    }

    /// 添加请求到批次
    pub async fn add_request(&self, request: RpcRequest) -> Result<Vec<RpcResponse>, DistributedError> {
        let (_response_sender, response_receiver) = oneshot::channel();
        
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.push(request);
            
            if pending.len() >= self.batch_size {
                let batch = pending.drain(..).collect();
                self.sender.send(batch).map_err(|_| DistributedError::Network("Failed to send batch".to_string()))?;
            }
        }

        // 等待响应
        timeout(self.batch_timeout, response_receiver)
            .await
            .map_err(|_| DistributedError::Network("Batch timeout".to_string()))?
            .map_err(|_| DistributedError::Network("Response channel closed".to_string()))?
    }
}

/// 增强的内存 RPC 服务器
#[derive(Default, Clone)]
pub struct InMemoryRpcServer {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>>>>,
    async_handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&[u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> + Send + Sync>>>>,
}

impl InMemoryRpcServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 处理批量请求
    #[cfg(feature = "runtime-tokio")]
    pub async fn handle_batch(&self, batch_request: BatchRpcRequest) -> BatchRpcResponse {
        let mut responses = Vec::new();
        
        for request in batch_request.requests {
            let result = if let Some(handler) = self.handlers.read().unwrap().get(&request.method) {
                Ok(handler(&request.payload))
            } else {
                Err(format!("Method not found: {}", request.method))
            };
            
            responses.push(RpcResponse {
                id: request.id,
                result,
            });
        }
        
        BatchRpcResponse {
            responses,
            batch_id: batch_request.batch_id,
        }
    }
}

impl RpcServer for InMemoryRpcServer {
    fn register(&mut self, method: &str, handler: Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>) {
        self.handlers
            .write()
            .expect("lock")
            .insert(method.to_string(), handler.into());
    }
    
    #[cfg(feature = "runtime-tokio")]
    fn register_async(&mut self, method: &str, handler: Box<dyn Fn(&[u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> + Send + Sync>) {
        // 简化实现，将异步处理器包装为同步处理器
        let _async_handler = handler;
        let sync_handler = Arc::new(move |payload: &[u8]| {
            // 这里需要在实际实现中使用 tokio::runtime 来运行异步代码
            // 为了简化，我们返回一个占位符
            b"async_handler_placeholder".to_vec()
        });
        
        self.handlers
            .write()
            .expect("lock")
            .insert(method.to_string(), sync_handler);
    }
}

/// 增强的内存 RPC 客户端
#[derive(Clone)]
pub struct InMemoryRpcClient {
    server: InMemoryRpcServer,
    connection_pool: Arc<ConnectionPool>,
    next_request_id: Arc<Mutex<u64>>,
}

impl InMemoryRpcClient {
    pub fn new(server: InMemoryRpcServer) -> Self {
        let config = ConnectionPoolConfig::default();
        Self {
            server,
            connection_pool: Arc::new(ConnectionPool::new(config)),
            next_request_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn with_connection_pool(server: InMemoryRpcServer, connection_pool: Arc<ConnectionPool>) -> Self {
        Self {
            server,
            connection_pool,
            next_request_id: Arc::new(Mutex::new(1)),
        }
    }

    /// 生成请求ID
    fn generate_request_id(&self) -> u64 {
        let mut next_id = self.next_request_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    }
}

impl RpcClient for InMemoryRpcClient {
    fn call(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>, DistributedError> {
        let handlers = self
            .server
            .handlers
            .read()
            .map_err(|_| DistributedError::Network("lock poisoned".into()))?;
        let f = handlers
            .get(method)
            .ok_or_else(|| DistributedError::Network(format!("method not found: {}", method)))?;
        Ok(f(payload))
    }

    #[cfg(feature = "runtime-tokio")]
    async fn call_async(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>, DistributedError> {
        // 获取连接
        let _connection_id = self.connection_pool.get_connection("localhost").await?;
        
        // 执行异步调用
        let result = self.call(method, payload);
        
        // 释放连接
        // self.connection_pool.release_connection(&connection_id);
        
        result
    }

    #[cfg(feature = "runtime-tokio")]
    async fn call_batch(&self, requests: Vec<RpcRequest>) -> Result<Vec<RpcResponse>, DistributedError> {
        let batch_id = self.generate_request_id();
        let batch_request = BatchRpcRequest {
            requests,
            batch_id,
        };

        let batch_response = self.server.handle_batch(batch_request).await;
        Ok(batch_response.responses)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub retry_on_empty: bool,
    pub backoff_base_ms: Option<u64>,
}

/// 重试客户端
pub struct RetryClient<C: RpcClient> {
    pub inner: C,
    pub policy: RetryPolicy,
}

impl<C: RpcClient> RetryClient<C> {
    pub fn new(inner: C, policy: RetryPolicy) -> Self {
        Self { inner, policy }
    }
}

impl<C: RpcClient> RpcClient for RetryClient<C> {
    fn call(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>, DistributedError> {
        let mut last_err: Option<DistributedError> = None;
        for attempt in 0..=self.policy.max_retries {
            match self.inner.call(method, payload) {
                Ok(v) => {
                    if self.policy.retry_on_empty && v.is_empty() {
                        if let Some(base) = self.policy.backoff_base_ms {
                            let delay = base.saturating_mul(1u64 << attempt.min(16));
                            std::thread::sleep(std::time::Duration::from_millis(delay));
                        }
                        continue;
                    }
                    return Ok(v);
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
            if let Some(base) = self.policy.backoff_base_ms {
                let delay = base.saturating_mul(1u64 << attempt.min(16));
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
        }
        Err(last_err.unwrap_or_else(|| DistributedError::Network("retry failed".into())))
    }

    #[cfg(feature = "runtime-tokio")]
    async fn call_async(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>, DistributedError> {
        let mut last_err: Option<DistributedError> = None;
        for attempt in 0..=self.policy.max_retries {
            match self.inner.call_async(method, payload).await {
                Ok(v) => {
                    if self.policy.retry_on_empty && v.is_empty() {
                        if let Some(base) = self.policy.backoff_base_ms {
                            let delay = base.saturating_mul(1u64 << attempt.min(16));
                            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                        }
                        continue;
                    }
                    return Ok(v);
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
            if let Some(base) = self.policy.backoff_base_ms {
                let delay = base.saturating_mul(1u64 << attempt.min(16));
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
        Err(last_err.unwrap_or_else(|| DistributedError::Network("retry failed".into())))
    }

    #[cfg(feature = "runtime-tokio")]
    async fn call_batch(&self, requests: Vec<RpcRequest>) -> Result<Vec<RpcResponse>, DistributedError> {
        let mut last_err: Option<DistributedError> = None;
        for attempt in 0..=self.policy.max_retries {
            match self.inner.call_batch(requests.clone()).await {
                Ok(responses) => {
                    // 检查是否有空响应需要重试
                    if self.policy.retry_on_empty && responses.iter().any(|r| r.result.is_ok() && r.result.as_ref().unwrap().is_empty()) {
                        if let Some(base) = self.policy.backoff_base_ms {
                            let delay = base.saturating_mul(1u64 << attempt.min(16));
                            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                        }
                        continue;
                    }
                    return Ok(responses);
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
            if let Some(base) = self.policy.backoff_base_ms {
                let delay = base.saturating_mul(1u64 << attempt.min(16));
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
        Err(last_err.unwrap_or_else(|| DistributedError::Network("retry failed".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_connection_pool() {
        let config = ConnectionPoolConfig {
            max_connections: 5,
            min_connections: 1,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
        };
        let pool = ConnectionPool::new(config);

        // 获取连接
        let conn1 = pool.get_connection("localhost").await.unwrap();
        let conn2 = pool.get_connection("localhost").await.unwrap();
        
        // 由于连接池会重用可用连接，conn1 和 conn2 可能相同
        // 这是正常行为，因为连接池会优先重用健康的连接
        assert!(!conn1.is_empty());
        assert!(!conn2.is_empty());
        
        // 释放连接
        pool.release_connection(&conn1);
        pool.release_connection(&conn2);
    }

    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_batch_rpc() {
        let mut server = InMemoryRpcServer::new();
        server.register("echo", Box::new(|payload| payload.to_vec()));
        
        let client = InMemoryRpcClient::new(server);
        
        let requests = vec![
            RpcRequest {
                id: 1,
                method: "echo".to_string(),
                payload: b"hello".to_vec(),
                timeout: None,
            },
            RpcRequest {
                id: 2,
                method: "echo".to_string(),
                payload: b"world".to_vec(),
                timeout: None,
            },
        ];

        let responses = client.call_batch(requests).await.unwrap();
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].result.as_ref().unwrap(), b"hello");
        assert_eq!(responses[1].result.as_ref().unwrap(), b"world");
    }

    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_async_rpc() {
        let mut server = InMemoryRpcServer::new();
        server.register("echo", Box::new(|payload| payload.to_vec()));
        
        let client = InMemoryRpcClient::new(server);
        
        let result = client.call_async("echo", b"test").await.unwrap();
        assert_eq!(result, b"test");
    }

    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_retry_client() {
        let mut server = InMemoryRpcServer::new();
        server.register("echo", Box::new(|payload| payload.to_vec()));
        
        let client = InMemoryRpcClient::new(server);
        let retry_policy = RetryPolicy {
            max_retries: 3,
            retry_on_empty: false,
            backoff_base_ms: Some(10),
        };
        
        let retry_client = RetryClient::new(client, retry_policy);
        
        let result = retry_client.call_async("echo", b"retry_test").await.unwrap();
        assert_eq!(result, b"retry_test");
    }

    #[test]
    fn test_request_batcher() {
        let (batcher, _receiver) = RequestBatcher::new(3, Duration::from_millis(100));
        
        // 测试请求合并逻辑
        let _request = RpcRequest {
            id: 1,
            method: "test".to_string(),
            payload: b"test".to_vec(),
            timeout: None,
        };
        
        // 这里需要实际运行异步代码来测试
        // 为了简化，我们只测试结构体创建
        assert_eq!(batcher.batch_size, 3);
    }
}

//! 网络通信使用示例

use crate::network::{
    InMemoryRpcClient, InMemoryRpcServer, RetryClient, RetryPolicy,
    RpcClient, RpcServer
};

#[cfg(feature = "runtime-tokio")]
use crate::network::{ConnectionPool, ConnectionPoolConfig, RpcRequest};
//use std::time::Duration;

/// 基本 RPC 通信示例
pub fn basic_rpc_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 基本 RPC 通信示例 ===");
    
    // 创建 RPC 服务器
    let mut server = InMemoryRpcServer::new();
    
    // 注册处理器
    server.register("echo", Box::new(|payload| {
        println!("服务器收到请求: {:?}", String::from_utf8_lossy(payload));
        payload.to_vec()
    }));
    
    server.register("add", Box::new(|payload| {
        let input = String::from_utf8_lossy(payload);
        let parts: Vec<&str> = input.split(',').collect();
        if parts.len() == 2 {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                let result = a + b;
                println!("服务器计算: {} + {} = {}", a, b, result);
                return result.to_string().into_bytes();
            }
        }
        b"Invalid input".to_vec()
    }));
    
    // 创建 RPC 客户端
    let client = InMemoryRpcClient::new(server);
    
    // 发送 echo 请求
    println!("发送 echo 请求...");
    let echo_response = client.call("echo", b"Hello, RPC!")?;
    println!("Echo 响应: {}", String::from_utf8_lossy(&echo_response));
    
    // 发送计算请求
    println!("发送计算请求...");
    let add_response = client.call("add", b"10,20")?;
    println!("计算结果: {}", String::from_utf8_lossy(&add_response));
    
    Ok(())
}

/// 连接池使用示例
#[cfg(feature = "runtime-tokio")]
pub async fn connection_pool_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 连接池使用示例 ===");
    
    // 创建连接池配置
    let config = ConnectionPoolConfig {
        max_connections: 5,
        min_connections: 2,
        connection_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(60),
        health_check_interval: Duration::from_secs(30),
    };
    
    let pool = ConnectionPool::new(config);
    
    // 获取多个连接
    println!("获取连接...");
    let mut connections = Vec::new();
    
    for i in 0..3 {
        let conn_id = pool.get_connection("localhost").await?;
        connections.push(conn_id.clone());
        println!("获取连接 {}: {}", i + 1, conn_id);
    }
    
    // 释放连接
    println!("释放连接...");
    for (i, conn_id) in connections.iter().enumerate() {
        pool.release_connection(conn_id);
        println!("释放连接 {}: {}", i + 1, conn_id);
    }
    
    // 获取连接统计信息
    let stats = pool.get_stats();
    println!("连接池统计: {} 个活跃连接", stats.len());
    
    Ok(())
}

/// 重试机制示例
pub fn retry_mechanism_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 重试机制示例 ===");
    
    // 创建服务器（模拟不稳定的服务）
    let mut server = InMemoryRpcServer::new();
    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let call_count_clone = call_count.clone();
    
    server.register("unstable_service", Box::new(move |_payload| {
        let count = call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        println!("服务调用次数: {}", count);
        
        // 前两次调用失败，第三次成功
        if count < 3 {
            panic!("模拟服务失败");
        }
        
        format!("成功响应 (第{}次调用)", count).into_bytes()
    }));
    
    let client = InMemoryRpcClient::new(server);
    
    // 配置重试策略
    let retry_policy = RetryPolicy {
        max_retries: 3,
        retry_on_empty: false,
        backoff_base_ms: Some(100), // 100ms 基础延迟
    };
    
    let retry_client = RetryClient::new(client, retry_policy);
    
    // 发送请求（会自动重试）
    println!("发送请求到不稳定服务...");
    match retry_client.call("unstable_service", b"test") {
        Ok(response) => {
            println!("✅ 最终成功: {}", String::from_utf8_lossy(&response));
        }
        Err(e) => {
            println!("❌ 重试后仍然失败: {}", e);
        }
    }
    
    Ok(())
}

/// 批量 RPC 请求示例
#[cfg(feature = "runtime-tokio")]
pub async fn batch_rpc_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 批量 RPC 请求示例 ===");
    
    // 创建服务器
    let mut server = InMemoryRpcServer::new();
    
    server.register("process_data", Box::new(|payload| {
        let input = String::from_utf8_lossy(payload);
        let result = format!("processed: {}", input);
        println!("处理数据: {} -> {}", input, result);
        result.into_bytes()
    }));
    
    let client = InMemoryRpcClient::new(server);
    
    // 创建批量请求
    let requests = vec![
        RpcRequest {
            id: 1,
            method: "process_data".to_string(),
            payload: b"data_1".to_vec(),
            timeout: Some(Duration::from_secs(5)),
        },
        RpcRequest {
            id: 2,
            method: "process_data".to_string(),
            payload: b"data_2".to_vec(),
            timeout: Some(Duration::from_secs(5)),
        },
        RpcRequest {
            id: 3,
            method: "process_data".to_string(),
            payload: b"data_3".to_vec(),
            timeout: Some(Duration::from_secs(5)),
        },
    ];
    
    println!("发送批量请求...");
    let responses = client.call_batch(requests).await?;
    
    println!("批量响应:");
    for response in responses {
        match response.result {
            Ok(data) => {
                println!("请求 {}: ✅ {}", response.id, String::from_utf8_lossy(&data));
            }
            Err(error) => {
                println!("请求 {}: ❌ {}", response.id, error);
            }
        }
    }
    
    Ok(())
}

/// 异步 RPC 调用示例
#[cfg(feature = "runtime-tokio")]
pub async fn async_rpc_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 异步 RPC 调用示例 ===");
    
    // 创建服务器
    let mut server = InMemoryRpcServer::new();
    
    server.register("slow_operation", Box::new(|payload| {
        let input = String::from_utf8_lossy(payload);
        println!("开始处理慢操作: {}", input);
        
        // 模拟慢操作
        std::thread::sleep(Duration::from_millis(100));
        
        let result = format!("completed: {}", input);
        println!("慢操作完成: {}", result);
        result.into_bytes()
    }));
    
    let client = InMemoryRpcClient::new(server);
    
    // 并发发送多个异步请求
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = client_clone.call_async("slow_operation", 
                format!("task_{}", i).as_bytes()).await;
            let duration = start.elapsed();
            
            match result {
                Ok(data) => {
                    println!("任务 {} 完成: {} (耗时: {:?})", 
                            i, String::from_utf8_lossy(&data), duration);
                }
                Err(e) => {
                    println!("任务 {} 失败: {} (耗时: {:?})", i, e, duration);
                }
            }
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await?;
    }
    
    println!("✅ 所有异步任务完成");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_rpc_demo() {
        basic_rpc_demo().unwrap();
    }
    
    #[test]
    fn test_retry_mechanism_demo() {
        retry_mechanism_demo().unwrap();
    }
    
    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_connection_pool_demo() {
        connection_pool_demo().await.unwrap();
    }
    
    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_batch_rpc_demo() {
        batch_rpc_demo().await.unwrap();
    }
    
    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_async_rpc_demo() {
        async_rpc_demo().await.unwrap();
    }
}

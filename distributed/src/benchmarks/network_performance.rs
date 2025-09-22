//! 网络通信性能基准测试

use crate::network::{
    InMemoryRpcClient, InMemoryRpcServer,
    //RpcRequest,
    //RpcResponse,
    //ConnectionPool, 
    //ConnectionPoolConfig,
    RpcServer, RpcClient,
    RetryClient, RetryPolicy
};
use std::time::{
    //Duration,
    Instant,
};
use std::sync::atomic::{AtomicI32, Ordering};

/// RPC 调用性能测试
pub fn benchmark_rpc_calls() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RPC 调用性能测试 ===");
    
    // 创建服务器
    let mut server = InMemoryRpcServer::new();
    server.register("echo", Box::new(|payload| payload.to_vec()));
    server.register("add", Box::new(|payload| {
        let input = String::from_utf8_lossy(payload);
        let parts: Vec<&str> = input.split(',').collect();
        if parts.len() == 2 {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                return (a + b).to_string().into_bytes();
            }
        }
        b"error".to_vec()
    }));
    
    let client = InMemoryRpcClient::new(server);
    
    // 测试不同大小的负载
    let payload_sizes = vec![64, 256, 1024, 4096, 16384];
    
    for size in &payload_sizes {
        println!("\n负载大小: {} 字节", size);
        
        let payload = vec![0u8; *size];
        let iterations = 1000;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = client.call("echo", &payload);
        }
        
        let duration = start.elapsed();
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();
        let throughput_mbps = (*size as f64 * iterations as f64) / (duration.as_secs_f64() * 1024.0 * 1024.0);
        
        println!("  总耗时: {:?}", duration);
        println!("  每秒操作数: {:.2}", ops_per_sec);
        println!("  吞吐量: {:.2} MB/s", throughput_mbps);
        println!("  平均延迟: {:.2}μs", duration.as_micros() as f64 / iterations as f64);
    }
    
    Ok(())
}

/// 批量 RPC 性能测试
#[cfg(feature = "runtime-tokio")]
pub async fn benchmark_batch_rpc() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 批量 RPC 性能测试 ===");
    
    let mut server = InMemoryRpcServer::new();
    server.register("process", Box::new(|payload| {
        format!("processed_{}", String::from_utf8_lossy(payload)).into_bytes()
    }));
    
    let client = InMemoryRpcClient::new(server);
    
    let batch_sizes = vec![1, 5, 10, 20, 50, 100];
    
    for batch_size in &batch_sizes {
        println!("\n批量大小: {}", batch_size);
        
        let iterations = 100;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let requests: Vec<RpcRequest> = (0..*batch_size)
                .map(|i| RpcRequest {
                    id: i as u64,
                    method: "process".to_string(),
                    payload: format!("data_{}", i).into_bytes(),
                    timeout: Some(Duration::from_secs(5)),
                })
                .collect();
            
            let _ = client.call_batch(requests).await;
        }
        
        let duration = start.elapsed();
        let total_requests = *batch_size * iterations;
        let ops_per_sec = total_requests as f64 / duration.as_secs_f64();
        
        println!("  总耗时: {:?}", duration);
        println!("  总请求数: {}", total_requests);
        println!("  每秒请求数: {:.2}", ops_per_sec);
        println!("  平均延迟: {:.2}μs", duration.as_micros() as f64 / total_requests as f64);
    }
    
    Ok(())
}

/// 连接池性能测试
#[cfg(feature = "runtime-tokio")]
pub async fn benchmark_connection_pool() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 连接池性能测试 ===");
    
    let config = ConnectionPoolConfig {
        max_connections: 10,
        min_connections: 2,
        connection_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(60),
        health_check_interval: Duration::from_secs(30),
    };
    
    let pool = ConnectionPool::new(config);
    
    // 测试连接获取性能
    println!("连接获取性能测试:");
    let iterations = 1000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _conn = pool.get_connection("localhost").await?;
    }
    
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    
    println!("  总耗时: {:?}", duration);
    println!("  每秒连接获取数: {:.2}", ops_per_sec);
    println!("  平均获取时间: {:.2}μs", duration.as_micros() as f64 / iterations as f64);
    
    // 测试并发连接获取
    println!("\n并发连接获取测试:");
    let num_threads = 10;
    let requests_per_thread = 100;
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for _ in 0..num_threads {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..requests_per_thread {
                let _conn = pool_clone.get_connection("localhost").await.unwrap();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await?;
    }
    
    let duration = start.elapsed();
    let total_requests = num_threads * requests_per_thread;
    let ops_per_sec = total_requests as f64 / duration.as_secs_f64();
    
    println!("  并发线程数: {}", num_threads);
    println!("  每线程请求数: {}", requests_per_thread);
    println!("  总耗时: {:?}", duration);
    println!("  每秒连接获取数: {:.2}", ops_per_sec);
    
    Ok(())
}

/// 重试机制性能测试
pub fn benchmark_retry_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 重试机制性能测试 ===");
    
    let mut server = InMemoryRpcServer::new();
    let call_count = std::sync::Arc::new(AtomicI32::new(0));
    let call_count_clone = call_count.clone();
    
    // 模拟不稳定的服务（50% 失败率）
    server.register("unstable", Box::new(move |_payload| {
        let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
        if count % 2 == 0 {
            panic!("模拟失败");
        }
        b"success".to_vec()
    }));
    
    let client = InMemoryRpcClient::new(server);
    
    let retry_policies = vec![
        RetryPolicy {
            max_retries: 1,
            retry_on_empty: false,
            backoff_base_ms: Some(10),
        },
        RetryPolicy {
            max_retries: 3,
            retry_on_empty: false,
            backoff_base_ms: Some(10),
        },
        RetryPolicy {
            max_retries: 5,
            retry_on_empty: false,
            backoff_base_ms: Some(10),
        },
    ];
    
    for (i, policy) in retry_policies.iter().enumerate() {
        println!("\n重试策略 {}: 最大重试次数 {}", i + 1, policy.max_retries);
        
        let retry_client = RetryClient::new(client.clone(), *policy);
        let iterations = 100;
        let start = Instant::now();
        
        let mut success_count = 0;
        for _ in 0..iterations {
            match retry_client.call("unstable", b"test") {
                Ok(_) => success_count += 1,
                Err(_) => {}
            }
        }
        
        let duration = start.elapsed();
        let success_rate = (success_count as f64 / iterations as f64) * 100.0;
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();
        
        println!("  总耗时: {:?}", duration);
        println!("  成功率: {:.2}%", success_rate);
        println!("  每秒操作数: {:.2}", ops_per_sec);
    }
    
    Ok(())
}

/// 内存使用量测试
pub fn benchmark_memory_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 网络模块内存使用量测试 ===");
    
    let num_servers = vec![10, 100, 1000];
    
    for count in &num_servers {
        println!("\n创建 {} 个服务器:", count);
        
        let start = Instant::now();
        let mut servers = Vec::new();
        
        for i in 0..*count {
            let mut server = InMemoryRpcServer::new();
            server.register("test", Box::new(move |payload| {
                format!("response_{}_{}", i, String::from_utf8_lossy(payload)).into_bytes()
            }));
            servers.push(server);
        }
        
        let creation_time = start.elapsed();
        println!("  创建耗时: {:?}", creation_time);
        println!("  平均创建时间: {:.2}μs", 
                creation_time.as_micros() as f64 / *count as f64);
        
        // 测试一些操作
        for (i, server) in servers.iter().enumerate() {
            if i % 100 == 0 {
                let client = InMemoryRpcClient::new(server.clone());
                let _ = client.call("test", b"memory_test");
            }
        }
        
        println!("  内存中服务器数量: {}", count);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_rpc_calls() {
        benchmark_rpc_calls().unwrap();
    }
    
    #[test]
    fn test_benchmark_retry_mechanism() {
        benchmark_retry_mechanism().unwrap();
    }
    
    #[test]
    fn test_benchmark_memory_usage() {
        benchmark_memory_usage().unwrap();
    }
    
    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_benchmark_batch_rpc() {
        benchmark_batch_rpc().await.unwrap();
    }
    
    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_benchmark_connection_pool() {
        benchmark_connection_pool().await.unwrap();
    }
}

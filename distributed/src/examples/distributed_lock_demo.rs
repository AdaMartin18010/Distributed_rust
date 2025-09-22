//! 分布式锁使用示例

//use crate::network::distributed_lock::{DistributedLockManager, DistributedMutex};
//use std::sync::Arc;
//use std::time::Duration;

/// 分布式锁基本使用示例
#[cfg(feature = "runtime-tokio")]
pub async fn basic_distributed_lock_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 分布式锁基本使用示例 ===");
    
    // 创建分布式锁管理器
    let manager = Arc::new(DistributedLockManager::new());
    
    // 创建互斥锁
    let mutex = DistributedMutex::new(
        manager.clone(),
        "demo_resource".to_string(),
        "client_1".to_string(),
    );
    
    // 尝试获取锁
    println!("尝试获取分布式锁...");
    match mutex.try_lock(Duration::from_secs(5), Duration::from_secs(30)) {
        Ok(true) => {
            println!("✅ 成功获取锁");
            
            // 模拟业务操作
            println!("执行临界区操作...");
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // 释放锁
            match mutex.unlock() {
                Ok(true) => println!("✅ 成功释放锁"),
                Ok(false) => println!("⚠️ 锁已经被释放"),
                Err(e) => println!("❌ 释放锁失败: {}", e),
            }
        }
        Ok(false) => {
            println!("⚠️ 获取锁失败，资源被其他客户端占用");
        }
        Err(e) => {
            println!("❌ 获取锁出错: {}", e);
        }
    }
    
    Ok(())
}

/// 并发锁竞争示例
#[cfg(feature = "runtime-tokio")]
pub async fn concurrent_lock_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 并发锁竞争示例 ===");
    
    let manager = Arc::new(DistributedLockManager::new());
    let mut handles = vec![];
    
    // 创建多个并发任务
    for i in 0..5 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let mutex = DistributedMutex::new(
                manager_clone,
                "shared_resource".to_string(),
                format!("client_{}", i),
            );
            
            match mutex.try_lock(Duration::from_secs(1), Duration::from_secs(10)) {
                Ok(true) => {
                    println!("客户端 {} 获取到锁", i);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    match mutex.unlock() {
                        Ok(_) => println!("客户端 {} 释放锁", i),
                        Err(e) => println!("客户端 {} 释放锁失败: {}", i, e),
                    }
                    true
                }
                Ok(false) => {
                    println!("客户端 {} 获取锁失败", i);
                    false
                }
                Err(e) => {
                    println!("客户端 {} 出错: {}", i, e);
                    false
                }
            }
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut success_count = 0;
    for handle in handles {
        if let Ok(success) = handle.await {
            if success {
                success_count += 1;
            }
        }
    }
    
    println!("✅ 成功获取锁的客户端数量: {}", success_count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    //use super::*;
    
    #[cfg(feature = "runtime-tokio")]
    #[tokio::test]
    async fn test_distributed_lock_demo() {
        basic_distributed_lock_demo().await.unwrap();
        concurrent_lock_demo().await.unwrap();
    }
}

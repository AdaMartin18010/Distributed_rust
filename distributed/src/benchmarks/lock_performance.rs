//! 分布式锁性能基准测试

use crate::network::distributed_lock::{DistributedLockManager, DistributedMutex};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;

/// 分布式锁获取性能测试
pub fn benchmark_lock_acquisition() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 分布式锁获取性能测试 ===");
    
    let manager = Arc::new(DistributedLockManager::new());
    let iterations = 1000;
    
    // 单线程性能测试
    println!("单线程性能测试 ({} 次迭代):", iterations);
    let start = Instant::now();
    
    for i in 0..iterations {
        let mutex = DistributedMutex::new(
            manager.clone(),
            format!("resource_{}", i),
            "single_thread".to_string(),
        );
        
        let _ = mutex.try_lock(Duration::from_millis(1), Duration::from_secs(1));
        let _ = mutex.unlock();
    }
    
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    
    println!("总耗时: {:?}", duration);
    println!("每秒操作数: {:.2}", ops_per_sec);
    println!("平均延迟: {:.2}μs", duration.as_micros() as f64 / iterations as f64);
    
    Ok(())
}

/// 并发锁竞争性能测试
pub fn benchmark_concurrent_lock_contention() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 并发锁竞争性能测试 ===");
    
    let manager = Arc::new(DistributedLockManager::new());
    let num_threads = 10;
    let iterations_per_thread = 100;
    
    println!("并发线程数: {}", num_threads);
    println!("每线程迭代数: {}", iterations_per_thread);
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for thread_id in 0..num_threads {
        let manager_clone = manager.clone();
        let handle = thread::spawn(move || {
            let mut success_count = 0;
            let mut total_attempts = 0;
            
            for _i in 0..iterations_per_thread {
                let mutex = DistributedMutex::new(
                    manager_clone.clone(),
                    "shared_resource".to_string(),
                    format!("thread_{}", thread_id),
                );
                
                total_attempts += 1;
                if mutex.try_lock(Duration::from_millis(1), Duration::from_secs(1)).unwrap_or(false) {
                    success_count += 1;
                    let _ = mutex.unlock();
                }
            }
            
            (success_count, total_attempts)
        });
        handles.push(handle);
    }
    
    let mut total_success = 0;
    let mut total_attempts = 0;
    
    for handle in handles {
        let (success, attempts) = handle.join().unwrap();
        total_success += success;
        total_attempts += attempts;
    }
    
    let duration = start.elapsed();
    let total_ops = total_attempts;
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    let success_rate = (total_success as f64 / total_attempts as f64) * 100.0;
    
    println!("总耗时: {:?}", duration);
    println!("总操作数: {}", total_ops);
    println!("成功操作数: {}", total_success);
    println!("成功率: {:.2}%", success_rate);
    println!("每秒操作数: {:.2}", ops_per_sec);
    println!("平均延迟: {:.2}μs", duration.as_micros() as f64 / total_ops as f64);
    
    Ok(())
}

/// 锁持有时间对性能的影响测试
pub fn benchmark_lock_hold_time_impact() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 锁持有时间对性能的影响测试 ===");
    
    let manager = Arc::new(DistributedLockManager::new());
    let hold_times = vec![
        Duration::from_micros(1),
        Duration::from_micros(10),
        Duration::from_micros(100),
        Duration::from_millis(1),
        Duration::from_millis(10),
    ];
    
    for hold_time in &hold_times {
        println!("\n锁持有时间: {:?}", hold_time);
        
        let start = Instant::now();
        let iterations = 100;
        
        for i in 0..iterations {
            let mutex = DistributedMutex::new(
                manager.clone(),
                format!("resource_{}", i),
                "hold_time_test".to_string(),
            );
            
            if mutex.try_lock(Duration::from_millis(1), Duration::from_secs(1)).unwrap_or(false) {
                thread::sleep(*hold_time);
                let _ = mutex.unlock();
            }
        }
        
        let duration = start.elapsed();
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();
        
        println!("  总耗时: {:?}", duration);
        println!("  每秒操作数: {:.2}", ops_per_sec);
    }
    
    Ok(())
}

/// 内存使用量测试
pub fn benchmark_memory_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 内存使用量测试 ===");
    
    let manager = Arc::new(DistributedLockManager::new());
    let num_locks = vec![100, 1000, 10000];
    
    for count in &num_locks {
        println!("\n创建 {} 个锁:", count);
        
        let start = Instant::now();
        let mut locks = Vec::new();
        
        for i in 0..*count {
            let mutex = DistributedMutex::new(
                manager.clone(),
                format!("memory_test_resource_{}", i),
                format!("client_{}", i),
            );
            locks.push(mutex);
        }
        
        let creation_time = start.elapsed();
        println!("  创建耗时: {:?}", creation_time);
        println!("  平均创建时间: {:.2}μs", 
                creation_time.as_micros() as f64 / *count as f64);
        
        // 模拟一些操作
        for (i, mutex) in locks.iter().enumerate() {
            if i % 100 == 0 {
                let _ = mutex.try_lock(Duration::from_millis(1), Duration::from_secs(1));
                let _ = mutex.unlock();
            }
        }
        
        println!("  内存中锁数量: {}", count);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_lock_acquisition() {
        benchmark_lock_acquisition().unwrap();
    }
    
    #[test]
    fn test_benchmark_concurrent_lock_contention() {
        benchmark_concurrent_lock_contention().unwrap();
    }
    
    #[test]
    fn test_benchmark_lock_hold_time_impact() {
        benchmark_lock_hold_time_impact().unwrap();
    }
    
    #[test]
    fn test_benchmark_memory_usage() {
        benchmark_memory_usage().unwrap();
    }
}

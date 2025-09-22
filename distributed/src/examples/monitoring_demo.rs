//! 监控系统使用示例

use crate::monitoring::{
    MetricCollector, PerformanceMonitor, 
    SystemHealthChecker, HealthCheck, HealthStatus
};
use std::collections::HashMap;
use std::time::Duration;

/// 基本监控指标使用示例
pub fn basic_monitoring_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 基本监控指标使用示例 ===");
    
    // 创建指标收集器
    let mut collector = MetricCollector::new();
    
    // 创建计数器
    let request_counter = collector.counter(
        "http_requests_total",
        HashMap::from([
            ("method".to_string(), "GET".to_string()),
            ("endpoint".to_string(), "/api/users".to_string()),
        ]),
    );
    
    // 创建计量器
    let memory_gauge = collector.gauge(
        "memory_usage_bytes",
        HashMap::from([("type".to_string(), "heap".to_string())]),
    );
    
    // 创建直方图
    let response_time_histogram = collector.histogram(
        "http_request_duration_seconds",
        HashMap::from([("method".to_string(), "GET".to_string())]),
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
    );
    
    // 模拟一些操作
    println!("模拟HTTP请求...");
    for i in 0..10 {
        request_counter.inc();
        
        // 模拟响应时间
        let response_time = 0.1 + (i as f64 * 0.05);
        response_time_histogram.observe(response_time);
        
        println!("请求 {}: 响应时间 {:.3}s", i + 1, response_time);
    }
    
    // 更新内存使用量
    memory_gauge.set((1024 * 1024 * 128) as u64); // 128MB
    
    // 获取所有指标
    let metrics = collector.get_all_metrics();
    println!("\n收集到的指标数量: {}", metrics.len());
    
    // 显示指标信息
    for metric in &metrics {
        println!("指标: {} = {:?}", metric.name, metric.value);
    }
    
    Ok(())
}

/// 性能监控器使用示例
pub fn performance_monitoring_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 性能监控器使用示例 ===");
    
    // 创建性能监控器
    let mut collector = MetricCollector::new();
    let monitor = PerformanceMonitor::new(&mut collector);
    
    // 模拟一些请求
    println!("模拟服务器请求处理...");
    for i in 0..5 {
        let start = std::time::Instant::now();
        
        // 模拟请求处理
        std::thread::sleep(Duration::from_millis(50 + i * 10));
        
        let duration = start.elapsed();
        let duration_seconds = duration.as_secs_f64();
        
        // 记录请求
        monitor.record_request(duration);
        
        // 模拟一些错误
        if i == 2 {
            monitor.record_error();
            println!("请求 {}: 处理失败", i + 1);
        } else {
            println!("请求 {}: 处理成功，耗时 {:.3}s", i + 1, duration_seconds);
        }
    }
    
    // 更新活跃连接数
    monitor.set_active_connections(42);
    
    println!("✅ 性能监控数据已记录");
    
    Ok(())
}

/// 健康检查系统使用示例
pub fn health_check_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 健康检查系统使用示例 ===");
    
    let mut health_checker = SystemHealthChecker::new();
    
    // 注册数据库健康检查
    health_checker.register_check("database".to_string(), || {
        let start = std::time::Instant::now();
        
        // 模拟数据库连接检查
        std::thread::sleep(Duration::from_millis(10));
        
        let duration = start.elapsed();
        
        HealthCheck {
            name: "database".to_string(),
            status: HealthStatus::Healthy,
            message: "数据库连接正常".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            duration_ms: duration.as_millis() as u64,
        }
    });
    
    // 注册缓存健康检查
    health_checker.register_check("cache".to_string(), || {
        let start = std::time::Instant::now();
        
        // 模拟缓存检查
        std::thread::sleep(Duration::from_millis(5));
        
        let duration = start.elapsed();
        
        HealthCheck {
            name: "cache".to_string(),
            status: HealthStatus::Healthy,
            message: "缓存服务正常".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            duration_ms: duration.as_millis() as u64,
        }
    });
    
    // 注册外部服务健康检查（模拟失败）
    health_checker.register_check("external_api".to_string(), || {
        let start = std::time::Instant::now();
        
        // 模拟外部API检查失败
        std::thread::sleep(Duration::from_millis(100));
        
        let duration = start.elapsed();
        
        HealthCheck {
            name: "external_api".to_string(),
            status: HealthStatus::Unhealthy,
            message: "外部API服务不可用".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            duration_ms: duration.as_millis() as u64,
        }
    });
    
    // 运行所有健康检查
    let results = health_checker.run_all_checks();
    
    println!("健康检查结果:");
    for check in &results {
        let status_icon = match check.status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Degraded => "⚠️",
            HealthStatus::Unhealthy => "❌",
        };
        println!("{} {}: {} ({}ms)", 
                status_icon, check.name, check.message, check.duration_ms);
    }
    
    // 获取整体健康状态
    let overall_status = health_checker.get_overall_status();
    let overall_icon = match overall_status {
        HealthStatus::Healthy => "✅",
        HealthStatus::Degraded => "⚠️",
        HealthStatus::Unhealthy => "❌",
    };
    println!("\n整体健康状态: {} {:?}", overall_icon, overall_status);
    
    Ok(())
}

/// Prometheus 格式导出示例
pub fn prometheus_export_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Prometheus 格式导出示例 ===");
    
    let mut collector = MetricCollector::new();
    
    // 创建一些指标
    let counter = collector.counter(
        "requests_total",
        HashMap::from([("service".to_string(), "api".to_string())]),
    );
    
    let gauge = collector.gauge(
        "active_connections",
        HashMap::from([("type".to_string(), "tcp".to_string())]),
    );
    
    let histogram = collector.histogram(
        "request_duration_seconds",
        HashMap::from([("method".to_string(), "POST".to_string())]),
        vec![0.1, 0.5, 1.0, 2.5, 5.0],
    );
    
    // 更新指标
    counter.add(100);
    gauge.set(25);
    histogram.observe(0.3);
    histogram.observe(0.8);
    histogram.observe(1.2);
    
    // 导出为 Prometheus 格式
    let prometheus_output = collector.get_all_metrics();
    
    println!("Prometheus 格式指标:");
    for metric in &prometheus_output {
        println!("{}", serde_json::to_string_pretty(metric)?);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_monitoring_demo() {
        basic_monitoring_demo().unwrap();
    }
    
    #[test]
    fn test_performance_monitoring_demo() {
        performance_monitoring_demo().unwrap();
    }
    
    #[test]
    fn test_health_check_demo() {
        health_check_demo().unwrap();
    }
    
    #[test]
    fn test_prometheus_export_demo() {
        prometheus_export_demo().unwrap();
    }
}

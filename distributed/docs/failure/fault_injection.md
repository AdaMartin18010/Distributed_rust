# 故障注入（Fault Injection）

> 分布式系统中的故障注入测试和混沌工程实践

## 目录

- [故障注入（Fault Injection）](#故障注入fault-injection)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [故障类型](#故障类型)
    - [注入策略](#注入策略)
    - [混沌工程](#混沌工程)
  - [🔧 实现机制](#-实现机制)
    - [故障注入器](#故障注入器)
    - [混沌控制器](#混沌控制器)
    - [故障监控](#故障监控)
  - [🚀 高级特性](#-高级特性)
    - [智能故障注入](#智能故障注入)
    - [自适应混沌](#自适应混沌)
  - [🧪 测试策略](#-测试策略)
    - [故障注入测试](#故障注入测试)
  - [🔍 性能优化](#-性能优化)
    - [注入优化](#注入优化)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

故障注入是分布式系统测试和验证的重要技术，通过主动引入故障来验证系统的容错能力和恢复机制。混沌工程是故障注入的扩展，通过在生产环境中进行受控的故障实验来提升系统的可靠性。

## 🎯 核心概念

### 故障类型

**定义 1（故障类型）**: 分布式系统中的故障类型包括：

- **节点故障**: 节点崩溃、重启、资源耗尽
- **网络故障**: 网络分区、延迟、丢包、乱序
- **存储故障**: 磁盘故障、数据损坏、IO错误
- **时钟故障**: 时钟漂移、时钟回拨、时钟停止
- **软件故障**: 内存泄漏、死锁、竞态条件

### 注入策略

**定义 2（注入策略）**: 故障注入的策略包括：

- **随机注入**: 随机时间和位置的故障注入
- **定时注入**: 在特定时间点注入故障
- **条件注入**: 满足特定条件时注入故障
- **级联注入**: 一个故障触发多个相关故障

### 混沌工程

**定义 3（混沌工程）**: 混沌工程是在生产环境中进行受控实验的学科，通过注入故障来验证系统的弹性和可靠性。

## 🔧 实现机制

### 故障注入器

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaultType {
    NodeCrash,
    NetworkPartition,
    NetworkDelay(Duration),
    PacketLoss(f64),
    ClockDrift(i64),
    MemoryLeak(u64),
    CpuSpike(Duration),
    DiskFull,
    DataCorruption,
}

#[derive(Debug, Clone)]
pub struct FaultInjection {
    pub fault_id: String,
    pub fault_type: FaultType,
    pub target_node: String,
    pub injection_time: u64,
    pub duration: Option<Duration>,
    pub probability: f64,
    pub conditions: Vec<FaultCondition>,
}

#[derive(Debug, Clone)]
pub struct FaultCondition {
    pub condition_type: String,
    pub condition_value: String,
    pub operator: String,
}

pub struct FaultInjector {
    active_faults: Arc<RwLock<HashMap<String, FaultInjection>>>,
    fault_history: Arc<RwLock<Vec<FaultRecord>>>,
    injection_scheduler: Arc<RwLock<FaultScheduler>>,
    monitoring: Arc<RwLock<FaultMonitoring>>,
}

#[derive(Debug, Clone)]
pub struct FaultRecord {
    pub fault_id: String,
    pub fault_type: FaultType,
    pub target_node: String,
    pub injection_time: u64,
    pub recovery_time: Option<u64>,
    pub duration: Option<u64>,
    pub success: bool,
    pub impact: FaultImpact,
}

#[derive(Debug, Clone)]
pub struct FaultImpact {
    pub affected_nodes: Vec<String>,
    pub affected_operations: Vec<String>,
    pub performance_degradation: f64,
    pub data_loss: bool,
}

impl FaultInjector {
    pub fn new() -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
            fault_history: Arc::new(RwLock::new(Vec::new())),
            injection_scheduler: Arc::new(RwLock::new(FaultScheduler::new())),
            monitoring: Arc::new(RwLock::new(FaultMonitoring::new())),
        }
    }
    
    // 注入故障
    pub async fn inject_fault(&self, fault: FaultInjection) -> Result<String, Box<dyn std::error::Error>> {
        let fault_id = fault.fault_id.clone();
        
        // 检查注入条件
        if !self.check_injection_conditions(&fault).await? {
            return Err("Injection conditions not met".into());
        }
        
        // 执行故障注入
        match &fault.fault_type {
            FaultType::NodeCrash => {
                self.inject_node_crash(&fault).await?;
            }
            FaultType::NetworkPartition => {
                self.inject_network_partition(&fault).await?;
            }
            FaultType::NetworkDelay(delay) => {
                self.inject_network_delay(&fault, *delay).await?;
            }
            FaultType::PacketLoss(rate) => {
                self.inject_packet_loss(&fault, *rate).await?;
            }
            FaultType::ClockDrift(offset) => {
                self.inject_clock_drift(&fault, *offset).await?;
            }
            FaultType::MemoryLeak(size) => {
                self.inject_memory_leak(&fault, *size).await?;
            }
            FaultType::CpuSpike(duration) => {
                self.inject_cpu_spike(&fault, *duration).await?;
            }
            FaultType::DiskFull => {
                self.inject_disk_full(&fault).await?;
            }
            FaultType::DataCorruption => {
                self.inject_data_corruption(&fault).await?;
            }
        }
        
        // 记录活跃故障
        let mut active_faults = self.active_faults.write().unwrap();
        active_faults.insert(fault_id.clone(), fault);
        
        // 记录故障历史
        let mut fault_history = self.fault_history.write().unwrap();
        let fault_record = FaultRecord {
            fault_id: fault_id.clone(),
            fault_type: fault.fault_type.clone(),
            target_node: fault.target_node.clone(),
            injection_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            recovery_time: None,
            duration: None,
            success: true,
            impact: FaultImpact {
                affected_nodes: vec![fault.target_node.clone()],
                affected_operations: Vec::new(),
                performance_degradation: 0.0,
                data_loss: false,
            },
        };
        fault_history.push(fault_record);
        
        Ok(fault_id)
    }
    
    // 检查注入条件
    async fn check_injection_conditions(&self, fault: &FaultInjection) -> Result<bool, Box<dyn std::error::Error>> {
        for condition in &fault.conditions {
            if !self.evaluate_condition(condition).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    // 评估条件
    async fn evaluate_condition(&self, condition: &FaultCondition) -> Result<bool, Box<dyn std::error::Error>> {
        // 简化实现，实际应该根据条件类型进行评估
        match condition.condition_type.as_str() {
            "node_load" => {
                // 检查节点负载
                Ok(true)
            }
            "network_latency" => {
                // 检查网络延迟
                Ok(true)
            }
            "memory_usage" => {
                // 检查内存使用率
                Ok(true)
            }
            _ => Ok(true),
        }
    }
    
    // 注入节点崩溃
    async fn inject_node_crash(&self, fault: &FaultInjection) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting node crash on: {}", fault.target_node);
        
        // 模拟节点崩溃
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    // 注入网络分区
    async fn inject_network_partition(&self, fault: &FaultInjection) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting network partition on: {}", fault.target_node);
        
        // 模拟网络分区
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    // 注入网络延迟
    async fn inject_network_delay(&self, fault: &FaultInjection, delay: Duration) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting network delay on: {} with delay: {:?}", fault.target_node, delay);
        
        // 模拟网络延迟
        tokio::time::sleep(delay).await;
        
        Ok(())
    }
    
    // 注入丢包
    async fn inject_packet_loss(&self, fault: &FaultInjection, loss_rate: f64) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting packet loss on: {} with rate: {}", fault.target_node, loss_rate);
        
        // 模拟丢包
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < loss_rate {
            return Err("Packet lost".into());
        }
        
        Ok(())
    }
    
    // 注入时钟漂移
    async fn inject_clock_drift(&self, fault: &FaultInjection, offset: i64) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting clock drift on: {} with offset: {}", fault.target_node, offset);
        
        // 模拟时钟漂移
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    // 注入内存泄漏
    async fn inject_memory_leak(&self, fault: &FaultInjection, size: u64) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting memory leak on: {} with size: {}", fault.target_node, size);
        
        // 模拟内存泄漏
        let _leaked_memory = vec![0u8; size as usize];
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    // 注入CPU峰值
    async fn inject_cpu_spike(&self, fault: &FaultInjection, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting CPU spike on: {} for duration: {:?}", fault.target_node, duration);
        
        // 模拟CPU峰值
        let start_time = SystemTime::now();
        while start_time.elapsed().unwrap() < duration {
            // 消耗CPU资源
            let _ = (0..1000).map(|i| i * i).collect::<Vec<_>>();
        }
        
        Ok(())
    }
    
    // 注入磁盘满
    async fn inject_disk_full(&self, fault: &FaultInjection) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting disk full on: {}", fault.target_node);
        
        // 模拟磁盘满
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    // 注入数据损坏
    async fn inject_data_corruption(&self, fault: &FaultInjection) -> Result<(), Box<dyn std::error::Error>> {
        println!("Injecting data corruption on: {}", fault.target_node);
        
        // 模拟数据损坏
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    // 恢复故障
    pub async fn recover_fault(&self, fault_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut active_faults = self.active_faults.write().unwrap();
        let mut fault_history = self.fault_history.write().unwrap();
        
        if let Some(fault) = active_faults.remove(&fault_id) {
            println!("Recovering fault: {}", fault_id);
            
            // 更新故障历史
            if let Some(record) = fault_history.iter_mut().find(|r| r.fault_id == fault_id) {
                record.recovery_time = Some(SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64);
                record.duration = record.recovery_time.map(|rt| rt - record.injection_time);
            }
        }
        
        Ok(())
    }
    
    // 获取故障统计
    pub fn get_fault_statistics(&self) -> FaultStatistics {
        let fault_history = self.fault_history.read().unwrap();
        let active_faults = self.active_faults.read().unwrap();
        
        let total_faults = fault_history.len();
        let active_faults_count = active_faults.len();
        let successful_faults = fault_history.iter().filter(|r| r.success).count();
        let failed_faults = total_faults - successful_faults;
        
        FaultStatistics {
            total_faults,
            active_faults: active_faults_count,
            successful_faults,
            failed_faults,
            average_recovery_time: self.calculate_average_recovery_time(&fault_history),
        }
    }
    
    // 计算平均恢复时间
    fn calculate_average_recovery_time(&self, fault_history: &[FaultRecord]) -> Option<u64> {
        let recovery_times: Vec<u64> = fault_history.iter()
            .filter_map(|record| record.duration)
            .collect();
        
        if recovery_times.is_empty() {
            None
        } else {
            Some(recovery_times.iter().sum::<u64>() / recovery_times.len() as u64)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FaultStatistics {
    pub total_faults: usize,
    pub active_faults: usize,
    pub successful_faults: usize,
    pub failed_faults: usize,
    pub average_recovery_time: Option<u64>,
}
```

### 混沌控制器

```rust
pub struct ChaosController {
    fault_injector: Arc<FaultInjector>,
    chaos_experiments: Arc<RwLock<Vec<ChaosExperiment>>>,
    experiment_scheduler: Arc<RwLock<ExperimentScheduler>>,
    safety_checks: Arc<RwLock<SafetyChecks>>,
}

#[derive(Debug, Clone)]
pub struct ChaosExperiment {
    pub experiment_id: String,
    pub name: String,
    pub description: String,
    pub fault_injections: Vec<FaultInjection>,
    pub duration: Duration,
    pub success_criteria: Vec<SuccessCriterion>,
    pub safety_conditions: Vec<SafetyCondition>,
}

#[derive(Debug, Clone)]
pub struct SuccessCriterion {
    pub criterion_type: String,
    pub target_value: f64,
    pub operator: String,
}

#[derive(Debug, Clone)]
pub struct SafetyCondition {
    pub condition_type: String,
    pub threshold: f64,
    pub action: SafetyAction,
}

#[derive(Debug, Clone)]
pub enum SafetyAction {
    Abort,
    ReduceIntensity,
    Pause,
    Alert,
}

impl ChaosController {
    pub fn new(fault_injector: Arc<FaultInjector>) -> Self {
        Self {
            fault_injector,
            chaos_experiments: Arc::new(RwLock::new(Vec::new())),
            experiment_scheduler: Arc::new(RwLock::new(ExperimentScheduler::new())),
            safety_checks: Arc::new(RwLock::new(SafetyChecks::new())),
        }
    }
    
    // 创建混沌实验
    pub fn create_experiment(&self, experiment: ChaosExperiment) -> Result<(), Box<dyn std::error::Error>> {
        let mut experiments = self.chaos_experiments.write().unwrap();
        experiments.push(experiment);
        Ok(())
    }
    
    // 执行混沌实验
    pub async fn run_experiment(&self, experiment_id: String) -> Result<ExperimentResult, Box<dyn std::error::Error>> {
        let experiments = self.chaos_experiments.read().unwrap();
        let experiment = experiments.iter()
            .find(|e| e.experiment_id == experiment_id)
            .ok_or("Experiment not found")?;
        
        println!("Starting chaos experiment: {}", experiment.name);
        
        // 执行安全检查
        if !self.run_safety_checks(experiment).await? {
            return Err("Safety checks failed".into());
        }
        
        let start_time = SystemTime::now();
        let mut injected_faults = Vec::new();
        
        // 注入故障
        for fault in &experiment.fault_injections {
            let fault_id = self.fault_injector.inject_fault(fault.clone()).await?;
            injected_faults.push(fault_id);
        }
        
        // 等待实验持续时间
        tokio::time::sleep(experiment.duration).await;
        
        // 恢复故障
        for fault_id in &injected_faults {
            self.fault_injector.recover_fault(fault_id.clone()).await?;
        }
        
        let end_time = SystemTime::now();
        let duration = end_time.duration_since(start_time).unwrap();
        
        // 评估成功标准
        let success = self.evaluate_success_criteria(experiment).await?;
        
        let result = ExperimentResult {
            experiment_id: experiment_id.clone(),
            success,
            duration: duration.as_millis() as u64,
            injected_faults,
            metrics: self.collect_experiment_metrics().await?,
        };
        
        println!("Chaos experiment completed: {}", experiment.name);
        Ok(result)
    }
    
    // 执行安全检查
    async fn run_safety_checks(&self, experiment: &ChaosExperiment) -> Result<bool, Box<dyn std::error::Error>> {
        for condition in &experiment.safety_conditions {
            if !self.check_safety_condition(condition).await? {
                match &condition.action {
                    SafetyAction::Abort => {
                        return Err("Safety condition violated, aborting experiment".into());
                    }
                    SafetyAction::ReduceIntensity => {
                        println!("Reducing experiment intensity due to safety condition");
                    }
                    SafetyAction::Pause => {
                        println!("Pausing experiment due to safety condition");
                    }
                    SafetyAction::Alert => {
                        println!("Alert: Safety condition violated");
                    }
                }
            }
        }
        Ok(true)
    }
    
    // 检查安全条件
    async fn check_safety_condition(&self, condition: &SafetyCondition) -> Result<bool, Box<dyn std::error::Error>> {
        match condition.condition_type.as_str() {
            "system_load" => {
                // 检查系统负载
                let current_load = self.get_system_load().await?;
                Ok(current_load < condition.threshold)
            }
            "error_rate" => {
                // 检查错误率
                let current_error_rate = self.get_error_rate().await?;
                Ok(current_error_rate < condition.threshold)
            }
            "response_time" => {
                // 检查响应时间
                let current_response_time = self.get_response_time().await?;
                Ok(current_response_time < condition.threshold)
            }
            _ => Ok(true),
        }
    }
    
    // 评估成功标准
    async fn evaluate_success_criteria(&self, experiment: &ChaosExperiment) -> Result<bool, Box<dyn std::error::Error>> {
        for criterion in &experiment.success_criteria {
            if !self.evaluate_criterion(criterion).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    // 评估标准
    async fn evaluate_criterion(&self, criterion: &SuccessCriterion) -> Result<bool, Box<dyn std::error::Error>> {
        match criterion.criterion_type.as_str() {
            "availability" => {
                let availability = self.get_availability().await?;
                Ok(availability >= criterion.target_value)
            }
            "performance" => {
                let performance = self.get_performance().await?;
                Ok(performance >= criterion.target_value)
            }
            "consistency" => {
                let consistency = self.get_consistency().await?;
                Ok(consistency >= criterion.target_value)
            }
            _ => Ok(true),
        }
    }
    
    // 收集实验指标
    async fn collect_experiment_metrics(&self) -> Result<ExperimentMetrics, Box<dyn std::error::Error>> {
        Ok(ExperimentMetrics {
            availability: self.get_availability().await?,
            performance: self.get_performance().await?,
            consistency: self.get_consistency().await?,
            error_rate: self.get_error_rate().await?,
            response_time: self.get_response_time().await?,
        })
    }
    
    // 获取系统负载
    async fn get_system_load(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际应该获取真实的系统负载
        Ok(0.5)
    }
    
    // 获取错误率
    async fn get_error_rate(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际应该获取真实的错误率
        Ok(0.01)
    }
    
    // 获取响应时间
    async fn get_response_time(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际应该获取真实的响应时间
        Ok(100.0)
    }
    
    // 获取可用性
    async fn get_availability(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际应该获取真实的可用性
        Ok(0.99)
    }
    
    // 获取性能
    async fn get_performance(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际应该获取真实的性能指标
        Ok(0.95)
    }
    
    // 获取一致性
    async fn get_consistency(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际应该获取真实的一致性指标
        Ok(0.98)
    }
}

#[derive(Debug, Clone)]
pub struct ExperimentResult {
    pub experiment_id: String,
    pub success: bool,
    pub duration: u64,
    pub injected_faults: Vec<String>,
    pub metrics: ExperimentMetrics,
}

#[derive(Debug, Clone)]
pub struct ExperimentMetrics {
    pub availability: f64,
    pub performance: f64,
    pub consistency: f64,
    pub error_rate: f64,
    pub response_time: f64,
}
```

### 故障监控

```rust
pub struct FaultMonitoring {
    metrics_collector: Arc<RwLock<MetricsCollector>>,
    alert_manager: Arc<RwLock<AlertManager>>,
    dashboard: Arc<RwLock<Dashboard>>,
}

pub struct MetricsCollector {
    metrics: HashMap<String, Vec<MetricPoint>>,
    collection_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

pub struct AlertManager {
    alert_rules: Vec<AlertRule>,
    active_alerts: HashMap<String, Alert>,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub rule_id: String,
    pub metric_name: String,
    pub condition: String,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub notification_channels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub alert_id: String,
    pub rule_id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub resolved: bool,
}

impl FaultMonitoring {
    pub fn new() -> Self {
        Self {
            metrics_collector: Arc::new(RwLock::new(MetricsCollector::new())),
            alert_manager: Arc::new(RwLock::new(AlertManager::new())),
            dashboard: Arc::new(RwLock::new(Dashboard::new())),
        }
    }
    
    // 收集指标
    pub async fn collect_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut collector = self.metrics_collector.write().unwrap();
        
        // 收集系统指标
        let system_metrics = self.collect_system_metrics().await?;
        collector.add_metrics("system", system_metrics);
        
        // 收集应用指标
        let app_metrics = self.collect_application_metrics().await?;
        collector.add_metrics("application", app_metrics);
        
        // 收集网络指标
        let network_metrics = self.collect_network_metrics().await?;
        collector.add_metrics("network", network_metrics);
        
        Ok(())
    }
    
    // 收集系统指标
    async fn collect_system_metrics(&self) -> Result<Vec<MetricPoint>, Box<dyn std::error::Error>> {
        let mut metrics = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        // CPU使用率
        metrics.push(MetricPoint {
            timestamp,
            value: 0.75, // 简化实现
            labels: HashMap::new(),
        });
        
        // 内存使用率
        metrics.push(MetricPoint {
            timestamp,
            value: 0.60, // 简化实现
            labels: HashMap::new(),
        });
        
        // 磁盘使用率
        metrics.push(MetricPoint {
            timestamp,
            value: 0.40, // 简化实现
            labels: HashMap::new(),
        });
        
        Ok(metrics)
    }
    
    // 收集应用指标
    async fn collect_application_metrics(&self) -> Result<Vec<MetricPoint>, Box<dyn std::error::Error>> {
        let mut metrics = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        // 请求延迟
        metrics.push(MetricPoint {
            timestamp,
            value: 150.0, // 简化实现
            labels: HashMap::new(),
        });
        
        // 错误率
        metrics.push(MetricPoint {
            timestamp,
            value: 0.02, // 简化实现
            labels: HashMap::new(),
        });
        
        // 吞吐量
        metrics.push(MetricPoint {
            timestamp,
            value: 1000.0, // 简化实现
            labels: HashMap::new(),
        });
        
        Ok(metrics)
    }
    
    // 收集网络指标
    async fn collect_network_metrics(&self) -> Result<Vec<MetricPoint>, Box<dyn std::error::Error>> {
        let mut metrics = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        // 网络延迟
        metrics.push(MetricPoint {
            timestamp,
            value: 50.0, // 简化实现
            labels: HashMap::new(),
        });
        
        // 网络丢包率
        metrics.push(MetricPoint {
            timestamp,
            value: 0.001, // 简化实现
            labels: HashMap::new(),
        });
        
        // 网络带宽使用率
        metrics.push(MetricPoint {
            timestamp,
            value: 0.30, // 简化实现
            labels: HashMap::new(),
        });
        
        Ok(metrics)
    }
    
    // 检查告警
    pub async fn check_alerts(&self) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alert_manager = self.alert_manager.write().unwrap();
        let collector = self.metrics_collector.read().unwrap();
        
        let mut new_alerts = Vec::new();
        
        for rule in &alert_manager.alert_rules {
            if let Some(metric_points) = collector.metrics.get(&rule.metric_name) {
                if let Some(latest_point) = metric_points.last() {
                    if self.evaluate_alert_condition(latest_point.value, &rule.condition, rule.threshold) {
                        let alert = Alert {
                            alert_id: format!("alert_{}", SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis()),
                            rule_id: rule.rule_id.clone(),
                            severity: rule.severity.clone(),
                            message: format!("Metric {} violated threshold: {} {}", 
                                           rule.metric_name, rule.condition, rule.threshold),
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                            resolved: false,
                        };
                        
                        alert_manager.active_alerts.insert(alert.alert_id.clone(), alert.clone());
                        new_alerts.push(alert);
                    }
                }
            }
        }
        
        Ok(new_alerts)
    }
    
    // 评估告警条件
    fn evaluate_alert_condition(&self, value: f64, condition: &str, threshold: f64) -> bool {
        match condition {
            ">" => value > threshold,
            "<" => value < threshold,
            ">=" => value >= threshold,
            "<=" => value <= threshold,
            "==" => (value - threshold).abs() < f64::EPSILON,
            "!=" => (value - threshold).abs() >= f64::EPSILON,
            _ => false,
        }
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            collection_interval: Duration::from_secs(10),
        }
    }
    
    pub fn add_metrics(&mut self, category: &str, metrics: Vec<MetricPoint>) {
        self.metrics.entry(category.to_string()).or_insert_with(Vec::new).extend(metrics);
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alert_rules: Vec::new(),
            active_alerts: HashMap::new(),
        }
    }
    
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        self.alert_rules.push(rule);
    }
}

pub struct FaultScheduler {
    scheduled_faults: Vec<ScheduledFault>,
}

pub struct ScheduledFault {
    pub fault_id: String,
    pub fault_injection: FaultInjection,
    pub schedule_time: u64,
    pub repeat_interval: Option<Duration>,
}

impl FaultScheduler {
    pub fn new() -> Self {
        Self {
            scheduled_faults: Vec::new(),
        }
    }
}

pub struct SafetyChecks {
    safety_rules: Vec<SafetyRule>,
}

pub struct SafetyRule {
    pub rule_id: String,
    pub condition: String,
    pub action: SafetyAction,
}

impl SafetyChecks {
    pub fn new() -> Self {
        Self {
            safety_rules: Vec::new(),
        }
    }
}

pub struct Dashboard {
    widgets: Vec<DashboardWidget>,
}

pub struct DashboardWidget {
    pub widget_id: String,
    pub widget_type: String,
    pub data_source: String,
    pub configuration: HashMap<String, String>,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }
}
```

## 🚀 高级特性

### 智能故障注入

```rust
pub struct IntelligentFaultInjection {
    fault_injector: Arc<FaultInjector>,
    machine_learning: Arc<RwLock<MachineLearningModel>>,
    pattern_analyzer: Arc<RwLock<PatternAnalyzer>>,
}

pub struct MachineLearningModel {
    model_data: Vec<u8>,
    training_data: Vec<TrainingSample>,
    prediction_accuracy: f64,
}

#[derive(Debug, Clone)]
pub struct TrainingSample {
    pub system_state: SystemState,
    pub fault_type: FaultType,
    pub impact: FaultImpact,
    pub recovery_time: u64,
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency: f64,
    pub error_rate: f64,
}

impl IntelligentFaultInjection {
    pub fn new(fault_injector: Arc<FaultInjector>) -> Self {
        Self {
            fault_injector,
            machine_learning: Arc::new(RwLock::new(MachineLearningModel::new())),
            pattern_analyzer: Arc::new(RwLock::new(PatternAnalyzer::new())),
        }
    }
    
    // 智能故障注入
    pub async fn intelligent_injection(&self, target_node: String) -> Result<String, Box<dyn std::error::Error>> {
        // 分析系统状态
        let system_state = self.analyze_system_state().await?;
        
        // 预测最佳故障类型
        let predicted_fault = self.predict_optimal_fault(&system_state).await?;
        
        // 创建智能故障注入
        let fault = FaultInjection {
            fault_id: format!("intelligent_{}", SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()),
            fault_type: predicted_fault,
            target_node,
            injection_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            duration: Some(Duration::from_secs(30)),
            probability: 1.0,
            conditions: Vec::new(),
        };
        
        self.fault_injector.inject_fault(fault).await
    }
    
    // 分析系统状态
    async fn analyze_system_state(&self) -> Result<SystemState, Box<dyn std::error::Error>> {
        Ok(SystemState {
            cpu_usage: 0.75,
            memory_usage: 0.60,
            disk_usage: 0.40,
            network_latency: 50.0,
            error_rate: 0.02,
        })
    }
    
    // 预测最佳故障类型
    async fn predict_optimal_fault(&self, system_state: &SystemState) -> Result<FaultType, Box<dyn std::error::Error>> {
        // 简化实现，实际应该使用机器学习模型
        if system_state.cpu_usage > 0.8 {
            Ok(FaultType::CpuSpike(Duration::from_secs(10)))
        } else if system_state.memory_usage > 0.9 {
            Ok(FaultType::MemoryLeak(1024 * 1024)) // 1MB
        } else if system_state.network_latency > 100.0 {
            Ok(FaultType::NetworkDelay(Duration::from_millis(200)))
        } else {
            Ok(FaultType::NodeCrash)
        }
    }
}

impl MachineLearningModel {
    pub fn new() -> Self {
        Self {
            model_data: Vec::new(),
            training_data: Vec::new(),
            prediction_accuracy: 0.0,
        }
    }
}

pub struct PatternAnalyzer {
    patterns: HashMap<String, FaultPattern>,
}

#[derive(Debug, Clone)]
pub struct FaultPattern {
    pub pattern_id: String,
    pub pattern_type: String,
    pub frequency: f64,
    pub impact: f64,
    pub recovery_time: u64,
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }
}
```

### 自适应混沌

```rust
pub struct AdaptiveChaos {
    chaos_controller: Arc<ChaosController>,
    adaptation_engine: Arc<RwLock<AdaptationEngine>>,
    learning_system: Arc<RwLock<LearningSystem>>,
}

pub struct AdaptationEngine {
    adaptation_rules: Vec<AdaptationRule>,
    adaptation_history: Vec<AdaptationRecord>,
}

#[derive(Debug, Clone)]
pub struct AdaptationRule {
    pub rule_id: String,
    pub trigger_condition: String,
    pub adaptation_action: AdaptationAction,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub enum AdaptationAction {
    IncreaseIntensity,
    DecreaseIntensity,
    ChangeFaultType,
    PauseExperiment,
    ResumeExperiment,
}

#[derive(Debug, Clone)]
pub struct AdaptationRecord {
    pub record_id: String,
    pub rule_id: String,
    pub action: AdaptationAction,
    pub timestamp: u64,
    pub success: bool,
    pub impact: f64,
}

impl AdaptiveChaos {
    pub fn new(chaos_controller: Arc<ChaosController>) -> Self {
        Self {
            chaos_controller,
            adaptation_engine: Arc::new(RwLock::new(AdaptationEngine::new())),
            learning_system: Arc::new(RwLock::new(LearningSystem::new())),
        }
    }
    
    // 自适应混沌实验
    pub async fn adaptive_experiment(&self, experiment_id: String) -> Result<ExperimentResult, Box<dyn std::error::Error>> {
        let mut adaptation_engine = self.adaptation_engine.write().unwrap();
        
        // 开始实验
        let mut result = self.chaos_controller.run_experiment(experiment_id.clone()).await?;
        
        // 监控实验进展
        let mut experiment_duration = Duration::from_secs(0);
        let max_duration = Duration::from_secs(300); // 5分钟
        
        while experiment_duration < max_duration {
            // 检查是否需要适应
            if let Some(adaptation) = self.should_adapt(&result).await? {
                // 执行适应
                self.execute_adaptation(adaptation).await?;
                
                // 记录适应历史
                let adaptation_record = AdaptationRecord {
                    record_id: format!("adapt_{}", SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis()),
                    rule_id: adaptation.rule_id,
                    action: adaptation.action,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    success: true,
                    impact: 0.0,
                };
                
                adaptation_engine.adaptation_history.push(adaptation_record);
            }
            
            tokio::time::sleep(Duration::from_secs(10)).await;
            experiment_duration += Duration::from_secs(10);
        }
        
        Ok(result)
    }
    
    // 判断是否需要适应
    async fn should_adapt(&self, result: &ExperimentResult) -> Result<Option<AdaptationRule>, Box<dyn std::error::Error>> {
        let adaptation_engine = self.adaptation_engine.read().unwrap();
        
        for rule in &adaptation_engine.adaptation_rules {
            if self.evaluate_trigger_condition(&rule.trigger_condition, result).await? {
                return Ok(Some(rule.clone()));
            }
        }
        
        Ok(None)
    }
    
    // 评估触发条件
    async fn evaluate_trigger_condition(&self, condition: &str, result: &ExperimentResult) -> Result<bool, Box<dyn std::error::Error>> {
        match condition {
            "low_availability" => Ok(result.metrics.availability < 0.9),
            "high_error_rate" => Ok(result.metrics.error_rate > 0.1),
            "slow_recovery" => Ok(result.duration > 60000), // 1分钟
            _ => Ok(false),
        }
    }
    
    // 执行适应
    async fn execute_adaptation(&self, rule: AdaptationRule) -> Result<(), Box<dyn std::error::Error>> {
        match rule.adaptation_action {
            AdaptationAction::IncreaseIntensity => {
                println!("Increasing experiment intensity");
            }
            AdaptationAction::DecreaseIntensity => {
                println!("Decreasing experiment intensity");
            }
            AdaptationAction::ChangeFaultType => {
                println!("Changing fault type");
            }
            AdaptationAction::PauseExperiment => {
                println!("Pausing experiment");
            }
            AdaptationAction::ResumeExperiment => {
                println!("Resuming experiment");
            }
        }
        
        Ok(())
    }
}

impl AdaptationEngine {
    pub fn new() -> Self {
        Self {
            adaptation_rules: Vec::new(),
            adaptation_history: Vec::new(),
        }
    }
}

pub struct LearningSystem {
    learning_data: Vec<LearningSample>,
    model_accuracy: f64,
}

#[derive(Debug, Clone)]
pub struct LearningSample {
    pub input: SystemState,
    pub output: FaultType,
    pub success: bool,
}

impl LearningSystem {
    pub fn new() -> Self {
        Self {
            learning_data: Vec::new(),
            model_accuracy: 0.0,
        }
    }
}
```

## 🧪 测试策略

### 故障注入测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fault_injection() {
        let fault_injector = FaultInjector::new();
        
        let fault = FaultInjection {
            fault_id: "test_fault".to_string(),
            fault_type: FaultType::NodeCrash,
            target_node: "node1".to_string(),
            injection_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            duration: Some(Duration::from_secs(10)),
            probability: 1.0,
            conditions: Vec::new(),
        };
        
        let fault_id = fault_injector.inject_fault(fault).await.unwrap();
        assert_eq!(fault_id, "test_fault");
        
        // 恢复故障
        fault_injector.recover_fault(fault_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_chaos_experiment() {
        let fault_injector = Arc::new(FaultInjector::new());
        let chaos_controller = ChaosController::new(fault_injector);
        
        let experiment = ChaosExperiment {
            experiment_id: "test_experiment".to_string(),
            name: "Test Experiment".to_string(),
            description: "A test chaos experiment".to_string(),
            fault_injections: vec![
                FaultInjection {
                    fault_id: "fault1".to_string(),
                    fault_type: FaultType::NetworkDelay(Duration::from_millis(100)),
                    target_node: "node1".to_string(),
                    injection_time: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    duration: Some(Duration::from_secs(5)),
                    probability: 1.0,
                    conditions: Vec::new(),
                }
            ],
            duration: Duration::from_secs(10),
            success_criteria: vec![
                SuccessCriterion {
                    criterion_type: "availability".to_string(),
                    target_value: 0.95,
                    operator: ">=".to_string(),
                }
            ],
            safety_conditions: vec![
                SafetyCondition {
                    condition_type: "system_load".to_string(),
                    threshold: 0.9,
                    action: SafetyAction::Abort,
                }
            ],
        };
        
        chaos_controller.create_experiment(experiment).unwrap();
        
        let result = chaos_controller.run_experiment("test_experiment".to_string()).await.unwrap();
        assert!(result.success);
    }
    
    #[tokio::test]
    async fn test_fault_monitoring() {
        let monitoring = FaultMonitoring::new();
        
        // 收集指标
        monitoring.collect_metrics().await.unwrap();
        
        // 检查告警
        let alerts = monitoring.check_alerts().await.unwrap();
        assert!(alerts.is_empty()); // 初始状态应该没有告警
    }
}
```

## 🔍 性能优化

### 注入优化

```rust
pub struct InjectionOptimizer {
    fault_injector: Arc<FaultInjector>,
    optimization_engine: Arc<RwLock<OptimizationEngine>>,
    performance_analyzer: Arc<RwLock<PerformanceAnalyzer>>,
}

pub struct OptimizationEngine {
    optimization_rules: Vec<OptimizationRule>,
    optimization_history: Vec<OptimizationRecord>,
}

#[derive(Debug, Clone)]
pub struct OptimizationRule {
    pub rule_id: String,
    pub optimization_type: String,
    pub target_metric: String,
    pub improvement_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationRecord {
    pub record_id: String,
    pub rule_id: String,
    pub optimization_type: String,
    pub before_metric: f64,
    pub after_metric: f64,
    pub improvement: f64,
    pub timestamp: u64,
}

impl InjectionOptimizer {
    pub fn new(fault_injector: Arc<FaultInjector>) -> Self {
        Self {
            fault_injector,
            optimization_engine: Arc::new(RwLock::new(OptimizationEngine::new())),
            performance_analyzer: Arc::new(RwLock::new(PerformanceAnalyzer::new())),
        }
    }
    
    // 优化故障注入
    pub async fn optimize_injection(&self, fault: &FaultInjection) -> Result<FaultInjection, Box<dyn std::error::Error>> {
        let mut optimized_fault = fault.clone();
        
        // 分析当前性能
        let current_performance = self.analyze_performance().await?;
        
        // 应用优化规则
        let optimization_engine = self.optimization_engine.read().unwrap();
        for rule in &optimization_engine.optimization_rules {
            if self.should_apply_optimization(rule, &current_performance).await? {
                optimized_fault = self.apply_optimization(rule, optimized_fault).await?;
            }
        }
        
        Ok(optimized_fault)
    }
    
    // 分析性能
    async fn analyze_performance(&self) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
        Ok(PerformanceMetrics {
            cpu_usage: 0.75,
            memory_usage: 0.60,
            disk_usage: 0.40,
            network_latency: 50.0,
            error_rate: 0.02,
            throughput: 1000.0,
            response_time: 150.0,
        })
    }
    
    // 判断是否应该应用优化
    async fn should_apply_optimization(&self, rule: &OptimizationRule, metrics: &PerformanceMetrics) -> Result<bool, Box<dyn std::error::Error>> {
        match rule.target_metric.as_str() {
            "cpu_usage" => Ok(metrics.cpu_usage > rule.improvement_threshold),
            "memory_usage" => Ok(metrics.memory_usage > rule.improvement_threshold),
            "network_latency" => Ok(metrics.network_latency > rule.improvement_threshold),
            "error_rate" => Ok(metrics.error_rate > rule.improvement_threshold),
            _ => Ok(false),
        }
    }
    
    // 应用优化
    async fn apply_optimization(&self, rule: &OptimizationRule, mut fault: FaultInjection) -> Result<FaultInjection, Box<dyn std::error::Error>> {
        match rule.optimization_type.as_str() {
            "reduce_intensity" => {
                // 减少故障强度
                fault.probability *= 0.8;
            }
            "adjust_duration" => {
                // 调整故障持续时间
                if let Some(duration) = fault.duration {
                    fault.duration = Some(duration / 2);
                }
            }
            "change_timing" => {
                // 改变注入时机
                fault.injection_time += 5000; // 延迟5秒
            }
            _ => {}
        }
        
        Ok(fault)
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency: f64,
    pub error_rate: f64,
    pub throughput: f64,
    pub response_time: f64,
}

impl OptimizationEngine {
    pub fn new() -> Self {
        Self {
            optimization_rules: Vec::new(),
            optimization_history: Vec::new(),
        }
    }
}

pub struct PerformanceAnalyzer {
    performance_data: Vec<PerformanceSample>,
    analysis_results: HashMap<String, AnalysisResult>,
}

#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp: u64,
    pub metrics: PerformanceMetrics,
    pub fault_type: Option<FaultType>,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub metric_name: String,
    pub trend: String,
    pub correlation: f64,
    pub recommendation: String,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            performance_data: Vec::new(),
            analysis_results: HashMap::new(),
        }
    }
}
```

## 📚 进一步阅读

- [故障模型](./README.md) - 故障模型概述
- [容错机制](./fault_tolerance.md) - 容错策略和恢复
- [测试策略](../testing/README.md) - 测试策略和验证
- [可观测性](../observability/README.md) - 监控和观测

## 🔗 相关文档

- [故障模型](./README.md)
- [容错机制](./fault_tolerance.md)
- [测试策略](../testing/README.md)
- [可观测性](../observability/README.md)
- [一致性模型](../consistency/README.md)
- [共识机制](../consensus/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

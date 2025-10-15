# 告警系统（Alerting System）

> 分布式系统中的告警规则、通知机制和事件处理

## 目录

- [告警系统（Alerting System）](#告警系统alerting-system)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [告警规则](#告警规则)
    - [通知机制](#通知机制)
    - [事件处理](#事件处理)
  - [🔧 实现机制](#-实现机制)
    - [告警管理器](#告警管理器)
    - [通知发送器](#通知发送器)
    - [事件处理器](#事件处理器)
  - [🚀 高级特性](#-高级特性)
    - [智能告警](#智能告警)
    - [告警聚合](#告警聚合)
  - [🧪 测试策略](#-测试策略)
    - [告警系统测试](#告警系统测试)
  - [🔍 性能优化](#-性能优化)
    - [告警优化](#告警优化)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

告警系统是分布式系统可观测性的重要组成部分，负责监控系统状态、检测异常情况并及时通知相关人员。一个有效的告警系统能够帮助运维团队快速发现和解决问题，确保系统的稳定运行。

## 🎯 核心概念

### 告警规则

**定义 1（告警规则）**: 告警规则定义了触发告警的条件和相应的处理动作，包括：

- **指标条件**: 基于系统指标的阈值判断
- **时间窗口**: 告警条件持续的时间范围
- **严重程度**: 告警的严重程度级别
- **处理动作**: 触发告警后执行的动作

### 通知机制

**定义 2（通知机制）**: 通知机制负责将告警信息传递给相关人员，包括：

- **通知渠道**: 邮件、短信、Slack、微信等
- **通知策略**: 升级策略、静默策略、聚合策略
- **通知内容**: 告警详情、上下文信息、处理建议

### 事件处理

**定义 3（事件处理）**: 事件处理是指对告警事件的完整生命周期管理，包括：

- **事件创建**: 告警事件的生成和记录
- **事件处理**: 告警的确认、处理和解决
- **事件关闭**: 告警事件的关闭和归档

## 🔧 实现机制

### 告警管理器

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertStatus {
    Firing,
    Resolved,
    Silenced,
    Acknowledged,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub notification_channels: Vec<String>,
    pub escalation_policy: EscalationPolicy,
    pub silence_duration: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct AlertCondition {
    pub operator: String,
    pub threshold: f64,
    pub time_window: Duration,
    pub evaluation_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct EscalationPolicy {
    pub levels: Vec<EscalationLevel>,
    pub max_escalations: usize,
}

#[derive(Debug, Clone)]
pub struct EscalationLevel {
    pub level: usize,
    pub delay: Duration,
    pub notification_channels: Vec<String>,
    pub recipients: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub alert_id: String,
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub resolved_at: Option<u64>,
    pub acknowledged_at: Option<u64>,
    pub acknowledged_by: Option<String>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub current_value: f64,
    pub threshold: f64,
}

pub struct AlertManager {
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    notification_sender: Arc<NotificationSender>,
    evaluation_engine: Arc<RwLock<EvaluationEngine>>,
}

impl AlertManager {
    pub fn new(notification_sender: Arc<NotificationSender>) -> Self {
        Self {
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            notification_sender,
            evaluation_engine: Arc::new(RwLock::new(EvaluationEngine::new())),
        }
    }
    
    // 添加告警规则
    pub fn add_alert_rule(&self, rule: AlertRule) -> Result<(), Box<dyn std::error::Error>> {
        let mut alert_rules = self.alert_rules.write().unwrap();
        alert_rules.insert(rule.rule_id.clone(), rule);
        Ok(())
    }
    
    // 评估告警规则
    pub async fn evaluate_alerts(&self) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let alert_rules = self.alert_rules.read().unwrap();
        let mut new_alerts = Vec::new();
        
        for (rule_id, rule) in alert_rules.iter() {
            // 获取指标值
            let metric_value = self.get_metric_value(&rule.metric_name).await?;
            
            // 评估告警条件
            if self.evaluate_condition(&rule.condition, metric_value) {
                // 检查是否已存在相同告警
                if !self.has_active_alert(rule_id) {
                    // 创建新告警
                    let alert = self.create_alert(rule, metric_value).await?;
                    new_alerts.push(alert.clone());
                    
                    // 发送通知
                    self.send_notification(&alert).await?;
                }
            } else {
                // 条件不满足，检查是否需要解决告警
                if self.has_active_alert(rule_id) {
                    self.resolve_alert(rule_id).await?;
                }
            }
        }
        
        Ok(new_alerts)
    }
    
    // 获取指标值
    async fn get_metric_value(&self, metric_name: &str) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际应该从指标收集器获取
        match metric_name {
            "cpu_usage" => Ok(0.75),
            "memory_usage" => Ok(0.60),
            "disk_usage" => Ok(0.40),
            "network_latency" => Ok(50.0),
            "error_rate" => Ok(0.02),
            _ => Ok(0.0),
        }
    }
    
    // 评估告警条件
    fn evaluate_condition(&self, condition: &AlertCondition, value: f64) -> bool {
        match condition.operator.as_str() {
            ">" => value > condition.threshold,
            "<" => value < condition.threshold,
            ">=" => value >= condition.threshold,
            "<=" => value <= condition.threshold,
            "==" => (value - condition.threshold).abs() < f64::EPSILON,
            "!=" => (value - condition.threshold).abs() >= f64::EPSILON,
            _ => false,
        }
    }
    
    // 检查是否有活跃告警
    fn has_active_alert(&self, rule_id: &str) -> bool {
        let active_alerts = self.active_alerts.read().unwrap();
        active_alerts.values().any(|alert| alert.rule_id == rule_id && alert.status == AlertStatus::Firing)
    }
    
    // 创建告警
    async fn create_alert(&self, rule: &AlertRule, current_value: f64) -> Result<Alert, Box<dyn std::error::Error>> {
        let alert_id = format!("alert_{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        let alert = Alert {
            alert_id: alert_id.clone(),
            rule_id: rule.rule_id.clone(),
            name: rule.name.clone(),
            description: rule.description.clone(),
            severity: rule.severity.clone(),
            status: AlertStatus::Firing,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            resolved_at: None,
            acknowledged_at: None,
            acknowledged_by: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            current_value,
            threshold: rule.condition.threshold,
        };
        
        // 添加到活跃告警
        let mut active_alerts = self.active_alerts.write().unwrap();
        active_alerts.insert(alert_id.clone(), alert.clone());
        
        // 添加到告警历史
        let mut alert_history = self.alert_history.write().unwrap();
        alert_history.push(alert.clone());
        
        Ok(alert)
    }
    
    // 解决告警
    pub async fn resolve_alert(&self, rule_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut active_alerts = self.active_alerts.write().unwrap();
        let mut alert_history = self.alert_history.write().unwrap();
        
        if let Some(alert) = active_alerts.values_mut().find(|a| a.rule_id == rule_id) {
            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64);
            alert.updated_at = alert.resolved_at.unwrap();
        }
        
        // 更新告警历史
        if let Some(history_alert) = alert_history.iter_mut().find(|a| a.rule_id == rule_id) {
            history_alert.status = AlertStatus::Resolved;
            history_alert.resolved_at = Some(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64);
            history_alert.updated_at = history_alert.resolved_at.unwrap();
        }
        
        Ok(())
    }
    
    // 确认告警
    pub async fn acknowledge_alert(&self, alert_id: &str, acknowledged_by: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut active_alerts = self.active_alerts.write().unwrap();
        
        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Acknowledged;
            alert.acknowledged_at = Some(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64);
            alert.acknowledged_by = Some(acknowledged_by);
            alert.updated_at = alert.acknowledged_at.unwrap();
        }
        
        Ok(())
    }
    
    // 发送通知
    async fn send_notification(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        let alert_rules = self.alert_rules.read().unwrap();
        
        if let Some(rule) = alert_rules.get(&alert.rule_id) {
            for channel in &rule.notification_channels {
                self.notification_sender.send_notification(channel, alert).await?;
            }
        }
        
        Ok(())
    }
    
    // 获取告警统计
    pub fn get_alert_statistics(&self) -> AlertStatistics {
        let active_alerts = self.active_alerts.read().unwrap();
        let alert_history = self.alert_history.read().unwrap();
        
        let total_alerts = alert_history.len();
        let active_alerts_count = active_alerts.len();
        let resolved_alerts = alert_history.iter().filter(|a| a.status == AlertStatus::Resolved).count();
        let acknowledged_alerts = alert_history.iter().filter(|a| a.status == AlertStatus::Acknowledged).count();
        
        AlertStatistics {
            total_alerts,
            active_alerts: active_alerts_count,
            resolved_alerts,
            acknowledged_alerts,
            average_resolution_time: self.calculate_average_resolution_time(&alert_history),
        }
    }
    
    // 计算平均解决时间
    fn calculate_average_resolution_time(&self, alert_history: &[Alert]) -> Option<u64> {
        let resolution_times: Vec<u64> = alert_history.iter()
            .filter_map(|alert| {
                if let (Some(created_at), Some(resolved_at)) = (Some(alert.created_at), alert.resolved_at) {
                    Some(resolved_at - created_at)
                } else {
                    None
                }
            })
            .collect();
        
        if resolution_times.is_empty() {
            None
        } else {
            Some(resolution_times.iter().sum::<u64>() / resolution_times.len() as u64)
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub active_alerts: usize,
    pub resolved_alerts: usize,
    pub acknowledged_alerts: usize,
    pub average_resolution_time: Option<u64>,
}
```

### 通知发送器

```rust
pub struct NotificationSender {
    email_sender: Arc<EmailSender>,
    sms_sender: Arc<SmsSender>,
    slack_sender: Arc<SlackSender>,
    webhook_sender: Arc<WebhookSender>,
}

pub struct EmailSender {
    smtp_server: String,
    smtp_port: u16,
    username: String,
    password: String,
}

pub struct SmsSender {
    api_endpoint: String,
    api_key: String,
}

pub struct SlackSender {
    webhook_url: String,
    channel: String,
}

pub struct WebhookSender {
    webhook_urls: HashMap<String, String>,
}

impl NotificationSender {
    pub fn new() -> Self {
        Self {
            email_sender: Arc::new(EmailSender::new()),
            sms_sender: Arc::new(SmsSender::new()),
            slack_sender: Arc::new(SlackSender::new()),
            webhook_sender: Arc::new(WebhookSender::new()),
        }
    }
    
    // 发送通知
    pub async fn send_notification(&self, channel: &str, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        match channel {
            "email" => {
                self.email_sender.send_email(alert).await?;
            }
            "sms" => {
                self.sms_sender.send_sms(alert).await?;
            }
            "slack" => {
                self.slack_sender.send_slack_message(alert).await?;
            }
            "webhook" => {
                self.webhook_sender.send_webhook(alert).await?;
            }
            _ => {
                return Err("Unsupported notification channel".into());
            }
        }
        
        Ok(())
    }
}

impl EmailSender {
    pub fn new() -> Self {
        Self {
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            username: "".to_string(),
            password: "".to_string(),
        }
    }
    
    pub async fn send_email(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending email alert: {}", alert.name);
        
        // 简化实现，实际应该使用邮件库发送邮件
        let email_content = format!(
            "Alert: {}\nDescription: {}\nSeverity: {:?}\nCurrent Value: {}\nThreshold: {}",
            alert.name, alert.description, alert.severity, alert.current_value, alert.threshold
        );
        
        println!("Email content: {}", email_content);
        Ok(())
    }
}

impl SmsSender {
    pub fn new() -> Self {
        Self {
            api_endpoint: "https://api.sms.com/send".to_string(),
            api_key: "".to_string(),
        }
    }
    
    pub async fn send_sms(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending SMS alert: {}", alert.name);
        
        // 简化实现，实际应该使用短信API发送短信
        let sms_content = format!("Alert: {} - {}", alert.name, alert.description);
        
        println!("SMS content: {}", sms_content);
        Ok(())
    }
}

impl SlackSender {
    pub fn new() -> Self {
        Self {
            webhook_url: "".to_string(),
            channel: "#alerts".to_string(),
        }
    }
    
    pub async fn send_slack_message(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending Slack alert: {}", alert.name);
        
        // 简化实现，实际应该使用Slack API发送消息
        let slack_message = format!(
            "🚨 Alert: {}\nDescription: {}\nSeverity: {:?}\nCurrent Value: {}\nThreshold: {}",
            alert.name, alert.description, alert.severity, alert.current_value, alert.threshold
        );
        
        println!("Slack message: {}", slack_message);
        Ok(())
    }
}

impl WebhookSender {
    pub fn new() -> Self {
        Self {
            webhook_urls: HashMap::new(),
        }
    }
    
    pub async fn send_webhook(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending webhook alert: {}", alert.name);
        
        // 简化实现，实际应该发送HTTP请求到webhook URL
        let webhook_payload = serde_json::json!({
            "alert_id": alert.alert_id,
            "name": alert.name,
            "description": alert.description,
            "severity": alert.severity,
            "current_value": alert.current_value,
            "threshold": alert.threshold,
            "created_at": alert.created_at,
        });
        
        println!("Webhook payload: {}", webhook_payload);
        Ok(())
    }
}
```

### 事件处理器

```rust
pub struct EventProcessor {
    alert_manager: Arc<AlertManager>,
    event_queue: Arc<RwLock<Vec<AlertEvent>>>,
    event_handlers: Arc<RwLock<HashMap<String, EventHandler>>>,
    event_history: Arc<RwLock<Vec<AlertEvent>>>,
}

#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub event_id: String,
    pub alert_id: String,
    pub event_type: EventType,
    pub timestamp: u64,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    AlertFired,
    AlertResolved,
    AlertAcknowledged,
    AlertEscalated,
    AlertSilenced,
}

pub struct EventHandler {
    pub handler_id: String,
    pub event_types: Vec<EventType>,
    pub handler_function: String,
    pub enabled: bool,
}

impl EventProcessor {
    pub fn new(alert_manager: Arc<AlertManager>) -> Self {
        Self {
            alert_manager,
            event_queue: Arc::new(RwLock::new(Vec::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    // 处理事件
    pub async fn process_events(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut event_queue = self.event_queue.write().unwrap();
        let event_handlers = self.event_handlers.read().unwrap();
        
        while let Some(event) = event_queue.pop() {
            // 查找匹配的事件处理器
            for (handler_id, handler) in event_handlers.iter() {
                if handler.enabled && handler.event_types.contains(&event.event_type) {
                    self.execute_handler(handler, &event).await?;
                }
            }
            
            // 添加到事件历史
            let mut event_history = self.event_history.write().unwrap();
            event_history.push(event);
        }
        
        Ok(())
    }
    
    // 执行事件处理器
    async fn execute_handler(&self, handler: &EventHandler, event: &AlertEvent) -> Result<(), Box<dyn std::error::Error>> {
        match handler.handler_function.as_str() {
            "log_event" => {
                self.log_event(event).await?;
            }
            "send_notification" => {
                self.send_event_notification(event).await?;
            }
            "trigger_automation" => {
                self.trigger_automation(event).await?;
            }
            "update_dashboard" => {
                self.update_dashboard(event).await?;
            }
            _ => {
                println!("Unknown handler function: {}", handler.handler_function);
            }
        }
        
        Ok(())
    }
    
    // 记录事件
    async fn log_event(&self, event: &AlertEvent) -> Result<(), Box<dyn std::error::Error>> {
        println!("Event logged: {:?} for alert {}", event.event_type, event.alert_id);
        Ok(())
    }
    
    // 发送事件通知
    async fn send_event_notification(&self, event: &AlertEvent) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending event notification: {:?} for alert {}", event.event_type, event.alert_id);
        Ok(())
    }
    
    // 触发自动化
    async fn trigger_automation(&self, event: &AlertEvent) -> Result<(), Box<dyn std::error::Error>> {
        println!("Triggering automation for event: {:?}", event.event_type);
        Ok(())
    }
    
    // 更新仪表板
    async fn update_dashboard(&self, event: &AlertEvent) -> Result<(), Box<dyn std::error::Error>> {
        println!("Updating dashboard for event: {:?}", event.event_type);
        Ok(())
    }
    
    // 添加事件处理器
    pub fn add_event_handler(&self, handler: EventHandler) -> Result<(), Box<dyn std::error::Error>> {
        let mut event_handlers = self.event_handlers.write().unwrap();
        event_handlers.insert(handler.handler_id.clone(), handler);
        Ok(())
    }
    
    // 创建事件
    pub fn create_event(&self, alert_id: String, event_type: EventType, data: HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
        let event_id = format!("event_{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        let event = AlertEvent {
            event_id: event_id.clone(),
            alert_id,
            event_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            data,
        };
        
        let mut event_queue = self.event_queue.write().unwrap();
        event_queue.push(event);
        
        Ok(event_id)
    }
}

pub struct EvaluationEngine {
    evaluation_rules: Vec<EvaluationRule>,
    evaluation_history: Vec<EvaluationRecord>,
}

#[derive(Debug, Clone)]
pub struct EvaluationRule {
    pub rule_id: String,
    pub metric_name: String,
    pub condition: String,
    pub threshold: f64,
    pub time_window: Duration,
}

#[derive(Debug, Clone)]
pub struct EvaluationRecord {
    pub record_id: String,
    pub rule_id: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub triggered: bool,
    pub timestamp: u64,
}

impl EvaluationEngine {
    pub fn new() -> Self {
        Self {
            evaluation_rules: Vec::new(),
            evaluation_history: Vec::new(),
        }
    }
}
```

## 🚀 高级特性

### 智能告警

```rust
pub struct IntelligentAlerting {
    alert_manager: Arc<AlertManager>,
    machine_learning: Arc<RwLock<MachineLearningModel>>,
    anomaly_detector: Arc<RwLock<AnomalyDetector>>,
    pattern_analyzer: Arc<RwLock<PatternAnalyzer>>,
}

pub struct MachineLearningModel {
    model_data: Vec<u8>,
    training_data: Vec<TrainingSample>,
    prediction_accuracy: f64,
}

#[derive(Debug, Clone)]
pub struct TrainingSample {
    pub metrics: HashMap<String, f64>,
    pub alert_triggered: bool,
    pub alert_severity: AlertSeverity,
    pub resolution_time: u64,
}

pub struct AnomalyDetector {
    baseline_metrics: HashMap<String, Vec<f64>>,
    anomaly_threshold: f64,
    detection_window: Duration,
}

pub struct PatternAnalyzer {
    patterns: HashMap<String, AlertPattern>,
    pattern_history: Vec<PatternRecord>,
}

#[derive(Debug, Clone)]
pub struct AlertPattern {
    pub pattern_id: String,
    pub pattern_type: String,
    pub frequency: f64,
    pub severity: AlertSeverity,
    pub common_causes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PatternRecord {
    pub record_id: String,
    pub pattern_id: String,
    pub timestamp: u64,
    pub metrics: HashMap<String, f64>,
    pub alert_triggered: bool,
}

impl IntelligentAlerting {
    pub fn new(alert_manager: Arc<AlertManager>) -> Self {
        Self {
            alert_manager,
            machine_learning: Arc::new(RwLock::new(MachineLearningModel::new())),
            anomaly_detector: Arc::new(RwLock::new(AnomalyDetector::new())),
            pattern_analyzer: Arc::new(RwLock::new(PatternAnalyzer::new())),
        }
    }
    
    // 智能告警评估
    pub async fn intelligent_evaluation(&self, metrics: HashMap<String, f64>) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut intelligent_alerts = Vec::new();
        
        // 异常检测
        let anomalies = self.detect_anomalies(&metrics).await?;
        for anomaly in anomalies {
            let alert = self.create_anomaly_alert(anomaly).await?;
            intelligent_alerts.push(alert);
        }
        
        // 模式分析
        let patterns = self.analyze_patterns(&metrics).await?;
        for pattern in patterns {
            let alert = self.create_pattern_alert(pattern).await?;
            intelligent_alerts.push(alert);
        }
        
        // 机器学习预测
        let predictions = self.predict_alerts(&metrics).await?;
        for prediction in predictions {
            let alert = self.create_prediction_alert(prediction).await?;
            intelligent_alerts.push(alert);
        }
        
        Ok(intelligent_alerts)
    }
    
    // 检测异常
    async fn detect_anomalies(&self, metrics: &HashMap<String, f64>) -> Result<Vec<Anomaly>, Box<dyn std::error::Error>> {
        let anomaly_detector = self.anomaly_detector.read().unwrap();
        let mut anomalies = Vec::new();
        
        for (metric_name, &value) in metrics {
            if let Some(baseline) = anomaly_detector.baseline_metrics.get(metric_name) {
                let baseline_mean = baseline.iter().sum::<f64>() / baseline.len() as f64;
                let baseline_std = (baseline.iter()
                    .map(|x| (x - baseline_mean).powi(2))
                    .sum::<f64>() / baseline.len() as f64).sqrt();
                
                let z_score = (value - baseline_mean) / baseline_std;
                
                if z_score.abs() > anomaly_detector.anomaly_threshold {
                    anomalies.push(Anomaly {
                        metric_name: metric_name.clone(),
                        value,
                        baseline_mean,
                        z_score,
                        severity: if z_score.abs() > 3.0 { AlertSeverity::High } else { AlertSeverity::Medium },
                    });
                }
            }
        }
        
        Ok(anomalies)
    }
    
    // 分析模式
    async fn analyze_patterns(&self, metrics: &HashMap<String, f64>) -> Result<Vec<AlertPattern>, Box<dyn std::error::Error>> {
        let pattern_analyzer = self.pattern_analyzer.read().unwrap();
        let mut matched_patterns = Vec::new();
        
        for (pattern_id, pattern) in &pattern_analyzer.patterns {
            if self.matches_pattern(metrics, pattern) {
                matched_patterns.push(pattern.clone());
            }
        }
        
        Ok(matched_patterns)
    }
    
    // 匹配模式
    fn matches_pattern(&self, metrics: &HashMap<String, f64>, pattern: &AlertPattern) -> bool {
        // 简化实现，实际应该使用更复杂的模式匹配算法
        match pattern.pattern_type.as_str() {
            "cpu_spike" => {
                metrics.get("cpu_usage").map_or(false, |&usage| usage > 0.9)
            }
            "memory_leak" => {
                metrics.get("memory_usage").map_or(false, |&usage| usage > 0.95)
            }
            "network_congestion" => {
                metrics.get("network_latency").map_or(false, |&latency| latency > 1000.0)
            }
            _ => false,
        }
    }
    
    // 预测告警
    async fn predict_alerts(&self, metrics: &HashMap<String, f64>) -> Result<Vec<AlertPrediction>, Box<dyn std::error::Error>> {
        let machine_learning = self.machine_learning.read().unwrap();
        let mut predictions = Vec::new();
        
        // 简化实现，实际应该使用机器学习模型进行预测
        for (metric_name, &value) in metrics {
            if value > 0.8 {
                predictions.push(AlertPrediction {
                    metric_name: metric_name.clone(),
                    predicted_value: value,
                    confidence: 0.85,
                    time_to_alert: Duration::from_secs(300), // 5分钟
                    severity: AlertSeverity::Medium,
                });
            }
        }
        
        Ok(predictions)
    }
    
    // 创建异常告警
    async fn create_anomaly_alert(&self, anomaly: Anomaly) -> Result<Alert, Box<dyn std::error::Error>> {
        let alert_id = format!("anomaly_{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        Ok(Alert {
            alert_id: alert_id.clone(),
            rule_id: "anomaly_detection".to_string(),
            name: format!("Anomaly detected in {}", anomaly.metric_name),
            description: format!("Anomalous value detected: {} (z-score: {:.2})", anomaly.value, anomaly.z_score),
            severity: anomaly.severity,
            status: AlertStatus::Firing,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            resolved_at: None,
            acknowledged_at: None,
            acknowledged_by: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            current_value: anomaly.value,
            threshold: anomaly.baseline_mean,
        })
    }
    
    // 创建模式告警
    async fn create_pattern_alert(&self, pattern: AlertPattern) -> Result<Alert, Box<dyn std::error::Error>> {
        let alert_id = format!("pattern_{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        Ok(Alert {
            alert_id: alert_id.clone(),
            rule_id: "pattern_detection".to_string(),
            name: format!("Pattern detected: {}", pattern.pattern_type),
            description: format!("Known pattern detected with frequency: {:.2}", pattern.frequency),
            severity: pattern.severity,
            status: AlertStatus::Firing,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            resolved_at: None,
            acknowledged_at: None,
            acknowledged_by: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            current_value: 0.0,
            threshold: 0.0,
        })
    }
    
    // 创建预测告警
    async fn create_prediction_alert(&self, prediction: AlertPrediction) -> Result<Alert, Box<dyn std::error::Error>> {
        let alert_id = format!("prediction_{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        Ok(Alert {
            alert_id: alert_id.clone(),
            rule_id: "prediction".to_string(),
            name: format!("Predicted alert for {}", prediction.metric_name),
            description: format!("Alert predicted with confidence: {:.2}, time to alert: {:?}", 
                               prediction.confidence, prediction.time_to_alert),
            severity: prediction.severity,
            status: AlertStatus::Firing,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            resolved_at: None,
            acknowledged_at: None,
            acknowledged_by: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            current_value: prediction.predicted_value,
            threshold: 0.0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Anomaly {
    pub metric_name: String,
    pub value: f64,
    pub baseline_mean: f64,
    pub z_score: f64,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone)]
pub struct AlertPrediction {
    pub metric_name: String,
    pub predicted_value: f64,
    pub confidence: f64,
    pub time_to_alert: Duration,
    pub severity: AlertSeverity,
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

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            baseline_metrics: HashMap::new(),
            anomaly_threshold: 2.0,
            detection_window: Duration::from_secs(300),
        }
    }
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            pattern_history: Vec::new(),
        }
    }
}
```

### 告警聚合

```rust
pub struct AlertAggregation {
    alert_manager: Arc<AlertManager>,
    aggregation_rules: Arc<RwLock<Vec<AggregationRule>>>,
    aggregated_alerts: Arc<RwLock<HashMap<String, AggregatedAlert>>>,
}

#[derive(Debug, Clone)]
pub struct AggregationRule {
    pub rule_id: String,
    pub name: String,
    pub group_by: Vec<String>,
    pub time_window: Duration,
    pub max_alerts: usize,
    pub aggregation_function: AggregationFunction,
}

#[derive(Debug, Clone)]
pub enum AggregationFunction {
    Count,
    Sum,
    Average,
    Maximum,
    Minimum,
}

#[derive(Debug, Clone)]
pub struct AggregatedAlert {
    pub aggregated_id: String,
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub resolved_at: Option<u64>,
    pub alert_count: usize,
    pub aggregated_value: f64,
    pub labels: HashMap<String, String>,
    pub source_alerts: Vec<String>,
}

impl AlertAggregation {
    pub fn new(alert_manager: Arc<AlertManager>) -> Self {
        Self {
            alert_manager,
            aggregation_rules: Arc::new(RwLock::new(Vec::new())),
            aggregated_alerts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // 添加聚合规则
    pub fn add_aggregation_rule(&self, rule: AggregationRule) -> Result<(), Box<dyn std::error::Error>> {
        let mut aggregation_rules = self.aggregation_rules.write().unwrap();
        aggregation_rules.push(rule);
        Ok(())
    }
    
    // 聚合告警
    pub async fn aggregate_alerts(&self) -> Result<Vec<AggregatedAlert>, Box<dyn std::error::Error>> {
        let active_alerts = self.alert_manager.active_alerts.read().unwrap();
        let aggregation_rules = self.aggregation_rules.read().unwrap();
        let mut new_aggregated_alerts = Vec::new();
        
        for rule in aggregation_rules.iter() {
            // 按规则分组告警
            let grouped_alerts = self.group_alerts_by_rule(&active_alerts, rule)?;
            
            for (group_key, alerts) in grouped_alerts {
                if alerts.len() >= rule.max_alerts {
                    // 创建聚合告警
                    let aggregated_alert = self.create_aggregated_alert(rule, &alerts, group_key).await?;
                    new_aggregated_alerts.push(aggregated_alert.clone());
                    
                    // 存储聚合告警
                    let mut aggregated_alerts = self.aggregated_alerts.write().unwrap();
                    aggregated_alerts.insert(aggregated_alert.aggregated_id.clone(), aggregated_alert);
                }
            }
        }
        
        Ok(new_aggregated_alerts)
    }
    
    // 按规则分组告警
    fn group_alerts_by_rule(&self, active_alerts: &HashMap<String, Alert>, rule: &AggregationRule) 
        -> Result<HashMap<String, Vec<Alert>>, Box<dyn std::error::Error>> {
        let mut grouped_alerts = HashMap::new();
        
        for alert in active_alerts.values() {
            let group_key = self.generate_group_key(alert, &rule.group_by)?;
            grouped_alerts.entry(group_key).or_insert_with(Vec::new).push(alert.clone());
        }
        
        Ok(grouped_alerts)
    }
    
    // 生成分组键
    fn generate_group_key(&self, alert: &Alert, group_by: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        let mut key_parts = Vec::new();
        
        for field in group_by {
            match field.as_str() {
                "rule_id" => key_parts.push(alert.rule_id.clone()),
                "severity" => key_parts.push(format!("{:?}", alert.severity)),
                "labels" => {
                    let labels_str = alert.labels.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(",");
                    key_parts.push(labels_str);
                }
                _ => {
                    return Err("Unknown group by field".into());
                }
            }
        }
        
        Ok(key_parts.join("|"))
    }
    
    // 创建聚合告警
    async fn create_aggregated_alert(&self, rule: &AggregationRule, alerts: &[Alert], group_key: String) 
        -> Result<AggregatedAlert, Box<dyn std::error::Error>> {
        let aggregated_id = format!("aggregated_{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        // 计算聚合值
        let aggregated_value = self.calculate_aggregated_value(alerts, &rule.aggregation_function)?;
        
        // 确定最高严重程度
        let max_severity = alerts.iter()
            .map(|a| &a.severity)
            .max_by_key(|s| match s {
                AlertSeverity::Critical => 4,
                AlertSeverity::High => 3,
                AlertSeverity::Medium => 2,
                AlertSeverity::Low => 1,
            })
            .unwrap_or(&AlertSeverity::Low)
            .clone();
        
        let aggregated_alert = AggregatedAlert {
            aggregated_id: aggregated_id.clone(),
            rule_id: rule.rule_id.clone(),
            name: format!("Aggregated: {}", rule.name),
            description: format!("Aggregated {} alerts", alerts.len()),
            severity: max_severity,
            status: AlertStatus::Firing,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            resolved_at: None,
            alert_count: alerts.len(),
            aggregated_value,
            labels: HashMap::new(),
            source_alerts: alerts.iter().map(|a| a.alert_id.clone()).collect(),
        };
        
        Ok(aggregated_alert)
    }
    
    // 计算聚合值
    fn calculate_aggregated_value(&self, alerts: &[Alert], function: &AggregationFunction) -> Result<f64, Box<dyn std::error::Error>> {
        let values: Vec<f64> = alerts.iter().map(|a| a.current_value).collect();
        
        match function {
            AggregationFunction::Count => Ok(alerts.len() as f64),
            AggregationFunction::Sum => Ok(values.iter().sum()),
            AggregationFunction::Average => Ok(values.iter().sum::<f64>() / values.len() as f64),
            AggregationFunction::Maximum => Ok(values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
            AggregationFunction::Minimum => Ok(values.iter().fold(f64::INFINITY, |a, &b| a.min(b))),
        }
    }
}
```

## 🧪 测试策略

### 告警系统测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_alert_creation() {
        let notification_sender = Arc::new(NotificationSender::new());
        let alert_manager = AlertManager::new(notification_sender);
        
        let rule = AlertRule {
            rule_id: "test_rule".to_string(),
            name: "Test Alert".to_string(),
            description: "A test alert rule".to_string(),
            metric_name: "cpu_usage".to_string(),
            condition: AlertCondition {
                operator: ">".to_string(),
                threshold: 0.8,
                time_window: Duration::from_secs(60),
                evaluation_interval: Duration::from_secs(10),
            },
            severity: AlertSeverity::High,
            notification_channels: vec!["email".to_string()],
            escalation_policy: EscalationPolicy {
                levels: Vec::new(),
                max_escalations: 3,
            },
            silence_duration: None,
        };
        
        alert_manager.add_alert_rule(rule).unwrap();
        
        let alerts = alert_manager.evaluate_alerts().await.unwrap();
        assert!(!alerts.is_empty());
    }
    
    #[tokio::test]
    async fn test_notification_sending() {
        let notification_sender = Arc::new(NotificationSender::new());
        
        let alert = Alert {
            alert_id: "test_alert".to_string(),
            rule_id: "test_rule".to_string(),
            name: "Test Alert".to_string(),
            description: "A test alert".to_string(),
            severity: AlertSeverity::High,
            status: AlertStatus::Firing,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            resolved_at: None,
            acknowledged_at: None,
            acknowledged_by: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            current_value: 0.9,
            threshold: 0.8,
        };
        
        let result = notification_sender.send_notification("email", &alert).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_event_processing() {
        let notification_sender = Arc::new(NotificationSender::new());
        let alert_manager = Arc::new(AlertManager::new(notification_sender));
        let event_processor = EventProcessor::new(alert_manager);
        
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        
        let event_id = event_processor.create_event(
            "test_alert".to_string(),
            EventType::AlertFired,
            data
        ).unwrap();
        
        assert!(!event_id.is_empty());
        
        let result = event_processor.process_events().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_alert_aggregation() {
        let notification_sender = Arc::new(NotificationSender::new());
        let alert_manager = Arc::new(AlertManager::new(notification_sender));
        let alert_aggregation = AlertAggregation::new(alert_manager);
        
        let rule = AggregationRule {
            rule_id: "aggregation_rule".to_string(),
            name: "Aggregation Rule".to_string(),
            group_by: vec!["rule_id".to_string()],
            time_window: Duration::from_secs(300),
            max_alerts: 2,
            aggregation_function: AggregationFunction::Count,
        };
        
        alert_aggregation.add_aggregation_rule(rule).unwrap();
        
        let aggregated_alerts = alert_aggregation.aggregate_alerts().await.unwrap();
        assert!(aggregated_alerts.is_empty()); // 初始状态应该没有聚合告警
    }
}
```

## 🔍 性能优化

### 告警优化

```rust
pub struct AlertOptimizer {
    alert_manager: Arc<AlertManager>,
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

impl AlertOptimizer {
    pub fn new(alert_manager: Arc<AlertManager>) -> Self {
        Self {
            alert_manager,
            optimization_engine: Arc::new(RwLock::new(OptimizationEngine::new())),
            performance_analyzer: Arc::new(RwLock::new(PerformanceAnalyzer::new())),
        }
    }
    
    // 优化告警规则
    pub async fn optimize_alert_rules(&self) -> Result<Vec<AlertRule>, Box<dyn std::error::Error>> {
        let alert_rules = self.alert_manager.alert_rules.read().unwrap();
        let mut optimized_rules = Vec::new();
        
        for (rule_id, rule) in alert_rules.iter() {
            // 分析规则性能
            let performance_metrics = self.analyze_rule_performance(rule).await?;
            
            // 应用优化
            let optimized_rule = self.apply_optimizations(rule, &performance_metrics).await?;
            optimized_rules.push(optimized_rule);
        }
        
        Ok(optimized_rules)
    }
    
    // 分析规则性能
    async fn analyze_rule_performance(&self, rule: &AlertRule) -> Result<RulePerformanceMetrics, Box<dyn std::error::Error>> {
        Ok(RulePerformanceMetrics {
            false_positive_rate: 0.05,
            false_negative_rate: 0.02,
            average_response_time: 100.0,
            alert_frequency: 0.1,
            resolution_time: 300.0,
        })
    }
    
    // 应用优化
    async fn apply_optimizations(&self, rule: &AlertRule, metrics: &RulePerformanceMetrics) -> Result<AlertRule, Box<dyn std::error::Error>> {
        let mut optimized_rule = rule.clone();
        
        // 优化阈值
        if metrics.false_positive_rate > 0.1 {
            optimized_rule.condition.threshold *= 1.1; // 提高阈值
        }
        
        // 优化时间窗口
        if metrics.alert_frequency > 0.2 {
            optimized_rule.condition.time_window *= 2; // 增加时间窗口
        }
        
        // 优化评估间隔
        if metrics.average_response_time > 200.0 {
            optimized_rule.condition.evaluation_interval *= 2; // 增加评估间隔
        }
        
        Ok(optimized_rule)
    }
}

#[derive(Debug, Clone)]
pub struct RulePerformanceMetrics {
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub average_response_time: f64,
    pub alert_frequency: f64,
    pub resolution_time: f64,
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
    pub metrics: RulePerformanceMetrics,
    pub rule_id: String,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub rule_id: String,
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

- [监控指标](./README.md) - 监控指标概述
- [分布式追踪](./tracing.md) - 链路追踪和性能分析
- [日志管理](./logging.md) - 结构化日志和日志聚合
- [健康检查](./health_check.md) - 健康检查和故障诊断

## 🔗 相关文档

- [监控指标](./README.md)
- [分布式追踪](./tracing.md)
- [日志管理](./logging.md)
- [健康检查](./health_check.md)
- [故障处理](../failure/README.md)
- [测试策略](../testing/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

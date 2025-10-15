# 分布式事务

> 分布式系统中的事务处理、SAGA、TCC、2PC 等模式实现

## 目录

- [分布式事务](#分布式事务)
  - [目录](#目录)
  - [📋 概述](#-概述)
  - [🎯 核心概念](#-核心概念)
    - [事务特性 (ACID)](#事务特性-acid)
    - [分布式事务挑战](#分布式事务挑战)
  - [🔧 事务模式](#-事务模式)
    - [SAGA 模式](#saga-模式)
    - [具体 SAGA 实现示例](#具体-saga-实现示例)
    - [TCC 模式 (Try-Confirm-Cancel)](#tcc-模式-try-confirm-cancel)
    - [2PC 模式 (Two-Phase Commit)](#2pc-模式-two-phase-commit)
  - [🔄 幂等性处理](#-幂等性处理)
    - [幂等性保证](#幂等性保证)
  - [🧪 测试策略](#-测试策略)
    - [事务测试](#事务测试)
  - [🔍 性能优化](#-性能优化)
    - [异步事务处理](#异步事务处理)
  - [📚 进一步阅读](#-进一步阅读)
  - [🔗 相关文档](#-相关文档)

## 📋 概述

分布式事务是分布式系统中的核心概念，用于确保跨多个服务或数据源的操作的一致性和原子性。

## 🎯 核心概念

### 事务特性 (ACID)

- **A (Atomicity)**: 原子性 - 事务要么全部成功，要么全部失败
- **C (Consistency)**: 一致性 - 事务执行前后系统状态保持一致
- **I (Isolation)**: 隔离性 - 并发事务之间相互隔离
- **D (Durability)**: 持久性 - 已提交的事务结果永久保存

### 分布式事务挑战

- **网络分区**: 网络故障导致节点间通信中断
- **节点故障**: 参与事务的节点可能发生故障
- **时钟不同步**: 不同节点的时钟可能存在偏差
- **部分失败**: 事务可能在某些节点成功，在另一些节点失败

## 🔧 事务模式

### SAGA 模式

SAGA 模式通过将长事务分解为多个短事务，每个短事务都有对应的补偿操作。

```rust
pub trait SagaStep {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // 执行步骤
    async fn execute(&mut self) -> Result<(), Self::Error>;
    
    // 补偿操作
    async fn compensate(&mut self) -> Result<(), Self::Error>;
    
    // 检查步骤是否已执行
    fn is_executed(&self) -> bool;
    
    // 获取步骤ID
    fn step_id(&self) -> &str;
}

pub struct SagaTransaction {
    steps: Vec<Box<dyn SagaStep>>,
    executed_steps: Vec<String>,
    compensation_required: bool,
}

impl SagaTransaction {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            executed_steps: Vec::new(),
            compensation_required: false,
        }
    }
    
    pub fn add_step(&mut self, step: Box<dyn SagaStep>) {
        self.steps.push(step);
    }
    
    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (i, step) in self.steps.iter_mut().enumerate() {
            match step.execute().await {
                Ok(_) => {
                    self.executed_steps.push(step.step_id().to_string());
                }
                Err(e) => {
                    // 执行失败，需要补偿
                    self.compensation_required = true;
                    return Err(format!("Step {} failed: {}", i, e).into());
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn compensate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 逆序执行补偿操作
        for step in self.steps.iter_mut().rev() {
            if step.is_executed() {
                if let Err(e) = step.compensate().await {
                    return Err(format!("Compensation failed for step {}: {}", step.step_id(), e).into());
                }
            }
        }
        
        Ok(())
    }
}
```

### 具体 SAGA 实现示例

```rust
// 支付步骤
pub struct PaymentStep {
    user_id: String,
    amount: u64,
    executed: bool,
    payment_id: Option<String>,
}

impl PaymentStep {
    pub fn new(user_id: String, amount: u64) -> Self {
        Self {
            user_id,
            amount,
            executed: false,
            payment_id: None,
        }
    }
}

impl SagaStep for PaymentStep {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn execute(&mut self) -> Result<(), Self::Error> {
        if self.executed {
            return Ok(());
        }
        
        // 模拟支付处理
        let payment_id = format!("payment_{}", uuid::Uuid::new_v4());
        
        // 调用支付服务
        let result = self.process_payment(&payment_id).await?;
        
        if result.success {
            self.payment_id = Some(payment_id);
            self.executed = true;
            Ok(())
        } else {
            Err("Payment failed".into())
        }
    }
    
    async fn compensate(&mut self) -> Result<(), Self::Error> {
        if let Some(payment_id) = &self.payment_id {
            // 执行退款
            self.refund_payment(payment_id).await?;
            self.executed = false;
            self.payment_id = None;
        }
        
        Ok(())
    }
    
    fn is_executed(&self) -> bool {
        self.executed
    }
    
    fn step_id(&self) -> &str {
        "payment"
    }
}

impl PaymentStep {
    async fn process_payment(&self, payment_id: &str) -> Result<PaymentResult, Box<dyn std::error::Error>> {
        // 模拟支付处理逻辑
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 模拟支付成功
        Ok(PaymentResult { success: true })
    }
    
    async fn refund_payment(&self, payment_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 模拟退款处理逻辑
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }
}

#[derive(Debug)]
struct PaymentResult {
    success: bool,
}

// 库存步骤
pub struct InventoryStep {
    product_id: String,
    quantity: u32,
    executed: bool,
    reservation_id: Option<String>,
}

impl InventoryStep {
    pub fn new(product_id: String, quantity: u32) -> Self {
        Self {
            product_id,
            quantity,
            executed: false,
            reservation_id: None,
        }
    }
}

impl SagaStep for InventoryStep {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn execute(&mut self) -> Result<(), Self::Error> {
        if self.executed {
            return Ok(());
        }
        
        // 模拟库存预留
        let reservation_id = format!("reservation_{}", uuid::Uuid::new_v4());
        
        let result = self.reserve_inventory(&reservation_id).await?;
        
        if result.success {
            self.reservation_id = Some(reservation_id);
            self.executed = true;
            Ok(())
        } else {
            Err("Inventory reservation failed".into())
        }
    }
    
    async fn compensate(&mut self) -> Result<(), Self::Error> {
        if let Some(reservation_id) = &self.reservation_id {
            // 释放库存预留
            self.release_inventory(reservation_id).await?;
            self.executed = false;
            self.reservation_id = None;
        }
        
        Ok(())
    }
    
    fn is_executed(&self) -> bool {
        self.executed
    }
    
    fn step_id(&self) -> &str {
        "inventory"
    }
}

impl InventoryStep {
    async fn reserve_inventory(&self, reservation_id: &str) -> Result<InventoryResult, Box<dyn std::error::Error>> {
        // 模拟库存预留逻辑
        tokio::time::sleep(Duration::from_millis(80)).await;
        
        // 模拟库存预留成功
        Ok(InventoryResult { success: true })
    }
    
    async fn release_inventory(&self, reservation_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 模拟库存释放逻辑
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok(())
    }
}

#[derive(Debug)]
struct InventoryResult {
    success: bool,
}
```

### TCC 模式 (Try-Confirm-Cancel)

TCC 模式将事务分为三个阶段：尝试(Try)、确认(Confirm)、取消(Cancel)。

```rust
pub trait TCCParticipant {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // 尝试阶段：预留资源
    async fn try(&mut self, context: &TCCContext) -> Result<(), Self::Error>;
    
    // 确认阶段：提交操作
    async fn confirm(&mut self, context: &TCCContext) -> Result<(), Self::Error>;
    
    // 取消阶段：释放资源
    async fn cancel(&mut self, context: &TCCContext) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct TCCContext {
    pub transaction_id: String,
    pub participant_id: String,
    pub timeout: Duration,
    pub created_at: u64,
}

pub struct TCCTransaction {
    participants: Vec<Box<dyn TCCParticipant>>,
    context: TCCContext,
    phase: TCCPhase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TCCPhase {
    Initial,
    Trying,
    Confirming,
    Cancelling,
    Committed,
    Aborted,
}

impl TCCTransaction {
    pub fn new(transaction_id: String) -> Self {
        let context = TCCContext {
            transaction_id: transaction_id.clone(),
            participant_id: "coordinator".to_string(),
            timeout: Duration::from_secs(30),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        };
        
        Self {
            participants: Vec::new(),
            context,
            phase: TCCPhase::Initial,
        }
    }
    
    pub fn add_participant(&mut self, participant: Box<dyn TCCParticipant>) {
        self.participants.push(participant);
    }
    
    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 第一阶段：尝试
        self.phase = TCCPhase::Trying;
        
        let mut try_results = Vec::new();
        for participant in &mut self.participants {
            match participant.try(&self.context).await {
                Ok(_) => try_results.push(true),
                Err(e) => {
                    try_results.push(false);
                    // 尝试失败，执行取消
                    return self.cancel().await;
                }
            }
        }
        
        // 第二阶段：确认
        self.phase = TCCPhase::Confirming;
        
        for (i, participant) in self.participants.iter_mut().enumerate() {
            if try_results[i] {
                if let Err(e) = participant.confirm(&self.context).await {
                    // 确认失败，执行取消
                    return self.cancel().await;
                }
            }
        }
        
        self.phase = TCCPhase::Committed;
        Ok(())
    }
    
    pub async fn cancel(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.phase = TCCPhase::Cancelling;
        
        // 对所有参与者执行取消操作
        for participant in &mut self.participants {
            let _ = participant.cancel(&self.context).await;
        }
        
        self.phase = TCCPhase::Aborted;
        Ok(())
    }
}
```

### 2PC 模式 (Two-Phase Commit)

2PC 模式通过协调者和参与者之间的两阶段协议确保事务的原子性。

```rust
pub trait TwoPhaseCommitParticipant {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // 准备阶段：准备提交
    async fn prepare(&mut self, transaction_id: &str) -> Result<PrepareResult, Self::Error>;
    
    // 提交阶段：提交事务
    async fn commit(&mut self, transaction_id: &str) -> Result<(), Self::Error>;
    
    // 中止阶段：中止事务
    async fn abort(&mut self, transaction_id: &str) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub enum PrepareResult {
    Prepared,
    Aborted,
}

pub struct TwoPhaseCommitCoordinator {
    participants: Vec<Box<dyn TwoPhaseCommitParticipant>>,
    transaction_id: String,
    phase: TwoPhaseCommitPhase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TwoPhaseCommitPhase {
    Initial,
    Preparing,
    Committing,
    Aborting,
    Committed,
    Aborted,
}

impl TwoPhaseCommitCoordinator {
    pub fn new(transaction_id: String) -> Self {
        Self {
            participants: Vec::new(),
            transaction_id,
            phase: TwoPhaseCommitPhase::Initial,
        }
    }
    
    pub fn add_participant(&mut self, participant: Box<dyn TwoPhaseCommitParticipant>) {
        self.participants.push(participant);
    }
    
    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 第一阶段：准备
        self.phase = TwoPhaseCommitPhase::Preparing;
        
        let mut prepare_results = Vec::new();
        for participant in &mut self.participants {
            match participant.prepare(&self.transaction_id).await {
                Ok(result) => prepare_results.push(result),
                Err(_) => {
                    prepare_results.push(PrepareResult::Aborted);
                }
            }
        }
        
        // 检查所有参与者是否都准备就绪
        let all_prepared = prepare_results.iter().all(|result| {
            matches!(result, PrepareResult::Prepared)
        });
        
        if all_prepared {
            // 第二阶段：提交
            self.phase = TwoPhaseCommitPhase::Committing;
            
            for participant in &mut self.participants {
                if let Err(e) = participant.commit(&self.transaction_id).await {
                    // 提交失败，记录错误但继续
                    eprintln!("Commit failed: {}", e);
                }
            }
            
            self.phase = TwoPhaseCommitPhase::Committed;
            Ok(())
        } else {
            // 有参与者准备失败，中止事务
            self.abort().await
        }
    }
    
    pub async fn abort(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.phase = TwoPhaseCommitPhase::Aborting;
        
        for participant in &mut self.participants {
            let _ = participant.abort(&self.transaction_id).await;
        }
        
        self.phase = TwoPhaseCommitPhase::Aborted;
        Ok(())
    }
}
```

## 🔄 幂等性处理

### 幂等性保证

```rust
pub struct IdempotencyManager {
    processed_operations: HashMap<String, OperationResult>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub result: Vec<u8>,
    pub timestamp: u64,
    pub status: OperationStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Success,
    Failure,
    InProgress,
}

impl IdempotencyManager {
    pub fn new(ttl: Duration) -> Self {
        Self {
            processed_operations: HashMap::new(),
            ttl,
        }
    }
    
    pub async fn execute_idempotent<F, Fut>(
        &mut self,
        operation_id: String,
        operation: F,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>>,
    {
        // 检查是否已经处理过
        if let Some(result) = self.processed_operations.get(&operation_id) {
            if result.timestamp + self.ttl.as_millis() as u64 > 
               SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 {
                match result.status {
                    OperationStatus::Success => return Ok(result.result.clone()),
                    OperationStatus::Failure => return Err("Previous operation failed".into()),
                    OperationStatus::InProgress => return Err("Operation in progress".into()),
                }
            }
        }
        
        // 标记为处理中
        self.processed_operations.insert(
            operation_id.clone(),
            OperationResult {
                result: Vec::new(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                status: OperationStatus::InProgress,
            },
        );
        
        // 执行操作
        match operation().await {
            Ok(result) => {
                // 记录成功结果
                self.processed_operations.insert(
                    operation_id,
                    OperationResult {
                        result: result.clone(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                        status: OperationStatus::Success,
                    },
                );
                Ok(result)
            }
            Err(e) => {
                // 记录失败结果
                self.processed_operations.insert(
                    operation_id,
                    OperationResult {
                        result: Vec::new(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                        status: OperationStatus::Failure,
                    },
                );
                Err(e)
            }
        }
    }
    
    pub fn cleanup_expired(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let ttl_ms = self.ttl.as_millis() as u64;
        
        self.processed_operations.retain(|_, result| {
            now - result.timestamp < ttl_ms
        });
    }
}
```

## 🧪 测试策略

### 事务测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_saga_transaction_success() {
        let mut saga = SagaTransaction::new();
        
        // 添加步骤
        saga.add_step(Box::new(PaymentStep::new("user1".to_string(), 100)));
        saga.add_step(Box::new(InventoryStep::new("product1".to_string(), 1)));
        
        // 执行事务
        let result = saga.execute().await;
        assert!(result.is_ok());
        
        // 验证所有步骤都已执行
        assert_eq!(saga.executed_steps.len(), 2);
    }
    
    #[tokio::test]
    async fn test_saga_transaction_failure_and_compensation() {
        let mut saga = SagaTransaction::new();
        
        // 添加会失败的步骤
        saga.add_step(Box::new(PaymentStep::new("user1".to_string(), 100)));
        saga.add_step(Box::new(FailingStep::new()));
        
        // 执行事务
        let result = saga.execute().await;
        assert!(result.is_err());
        
        // 执行补偿
        let compensation_result = saga.compensate().await;
        assert!(compensation_result.is_ok());
        
        // 验证补偿已执行
        assert!(saga.compensation_required);
    }
    
    #[tokio::test]
    async fn test_tcc_transaction() {
        let mut tcc = TCCTransaction::new("tx1".to_string());
        
        // 添加参与者
        tcc.add_participant(Box::new(MockTCCParticipant::new("participant1".to_string())));
        tcc.add_participant(Box::new(MockTCCParticipant::new("participant2".to_string())));
        
        // 执行事务
        let result = tcc.execute().await;
        assert!(result.is_ok());
        assert_eq!(tcc.phase, TCCPhase::Committed);
    }
    
    #[tokio::test]
    async fn test_idempotency_manager() {
        let mut manager = IdempotencyManager::new(Duration::from_secs(60));
        
        let operation_id = "op1".to_string();
        
        // 第一次执行
        let result1 = manager.execute_idempotent(operation_id.clone(), || async {
            Ok(b"result".to_vec())
        }).await;
        
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), b"result");
        
        // 第二次执行（应该返回相同结果）
        let result2 = manager.execute_idempotent(operation_id, || async {
            Ok(b"different_result".to_vec())
        }).await;
        
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), b"result"); // 返回第一次的结果
    }
}

// 测试用的失败步骤
struct FailingStep {
    executed: bool,
}

impl FailingStep {
    fn new() -> Self {
        Self { executed: false }
    }
}

impl SagaStep for FailingStep {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn execute(&mut self) -> Result<(), Self::Error> {
        self.executed = true;
        Err("Step failed".into())
    }
    
    async fn compensate(&mut self) -> Result<(), Self::Error> {
        self.executed = false;
        Ok(())
    }
    
    fn is_executed(&self) -> bool {
        self.executed
    }
    
    fn step_id(&self) -> &str {
        "failing_step"
    }
}

// 测试用的 TCC 参与者
struct MockTCCParticipant {
    id: String,
    prepared: bool,
    committed: bool,
}

impl MockTCCParticipant {
    fn new(id: String) -> Self {
        Self {
            id,
            prepared: false,
            committed: false,
        }
    }
}

impl TCCParticipant for MockTCCParticipant {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn try(&mut self, _context: &TCCContext) -> Result<(), Self::Error> {
        self.prepared = true;
        Ok(())
    }
    
    async fn confirm(&mut self, _context: &TCCContext) -> Result<(), Self::Error> {
        self.committed = true;
        Ok(())
    }
    
    async fn cancel(&mut self, _context: &TCCContext) -> Result<(), Self::Error> {
        self.prepared = false;
        Ok(())
    }
}
```

## 🔍 性能优化

### 异步事务处理

```rust
pub struct AsyncTransactionProcessor {
    transaction_queue: mpsc::UnboundedSender<Box<dyn Transaction>>,
    worker_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl AsyncTransactionProcessor {
    pub fn new(worker_count: usize) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let mut worker_handles = Vec::new();
        for i in 0..worker_count {
            let mut worker_rx = rx.clone();
            let handle = tokio::spawn(async move {
                while let Some(transaction) = worker_rx.recv().await {
                    if let Err(e) = transaction.execute().await {
                        eprintln!("Transaction execution failed: {}", e);
                    }
                }
            });
            worker_handles.push(handle);
        }
        
        Self {
            transaction_queue: tx,
            worker_handles,
        }
    }
    
    pub async fn submit_transaction(&self, transaction: Box<dyn Transaction>) -> Result<(), Box<dyn std::error::Error>> {
        self.transaction_queue.send(transaction)?;
        Ok(())
    }
}

pub trait Transaction {
    async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
```

## 📚 进一步阅读

- [一致性模型](../consistency/README.md) - 一致性级别和保证
- [故障处理](../failure/README.md) - 故障检测和恢复
- [共识机制](../consensus/README.md) - 分布式共识算法
- [性能优化](../performance/OPTIMIZATION.md) - 事务性能优化

## 🔗 相关文档

- [一致性模型](../consistency/README.md)
- [故障处理](../failure/README.md)
- [共识机制](../consensus/README.md)
- [性能优化](../performance/OPTIMIZATION.md)
- [测试策略](../testing/README.md)

---

**文档版本**: v1.0.0  
**最后更新**: 2025-10-15  
**维护者**: Rust 分布式系统项目组

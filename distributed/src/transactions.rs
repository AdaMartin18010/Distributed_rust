//! 分布式事务（Saga 为主）
//!
//! 设计目标：
//! - 提供 Saga 模式的最小执行/补偿框架，适合长事务与跨服务编排。
//! - 通过按序执行与逆序补偿，获得最终一致性；结合幂等与去重存储可避免重试副作用。
//!
//! 不变量与失败语义（草图）：
//! - 原子可补偿性：若步骤 i 失败，则必须存在定义良好的补偿将 1..i-1 的副作用回滚至可接受状态。
//! - 幂等执行：`execute` 与 `compensate` 应可在网络重试下安全重入。
//! - 有界回滚：补偿序列严格逆序于已完成步骤，避免状态错乱。
//!
//! 参考：
//! - Garcia-Molina & Salem, Sagas, 1987.
//! - Pat Helland, Life beyond Distributed Transactions, 2007.
use crate::core::errors::DistributedError;

pub trait SagaStep {
    fn execute(&mut self) -> Result<(), DistributedError>;
    fn compensate(&mut self) -> Result<(), DistributedError>;
}

pub struct Saga {
    steps: Vec<Box<dyn SagaStep + Send>>,
}

impl Default for Saga {
    fn default() -> Self {
        Self::new()
    }
}

impl Saga {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }
    pub fn then(mut self, step: Box<dyn SagaStep + Send>) -> Self {
        self.steps.push(step);
        self
    }

    pub fn run(self) -> Result<(), DistributedError> {
        let mut done: Vec<Box<dyn SagaStep + Send>> = Vec::new();
        for mut s in self.steps.into_iter() {
            match s.execute() {
                Ok(_) => done.push(s),
                Err(e) => {
                    // rollback in reverse
                    while let Some(mut step) = done.pop() {
                        let _ = step.compensate();
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

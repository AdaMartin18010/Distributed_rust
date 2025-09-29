//! 错误分类与语义
//!
//! 目标：
//! - 提供统一的错误类型 `DistributedError`，用于网络、配置、共识、存储与状态错误的表达。
//! - 为上层重试/降级/回滚提供依据：例如网络错误可重试，配置/状态错误通常不可重试。
//!
//! 工程化注意：
//! - 建议在边界处尽早分类错误并携带上下文（request id、node id、shard 等）。
//! - 与监控结合：按错误类别与来源维度产出指标与追踪。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DistributedError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("consensus error: {0}")]
    Consensus(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
}

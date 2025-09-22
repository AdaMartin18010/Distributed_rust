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

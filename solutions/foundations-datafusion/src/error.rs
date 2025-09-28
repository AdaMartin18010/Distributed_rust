use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("DataFusion 错误: {0}")]
    DataFusion(#[from] datafusion::error::DataFusionError),
    
    #[error("Tonic 错误: {0}")]
    Tonic(#[from] tonic::Status),
    
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("网络错误: {0}")]
    Network(String),
    
    #[error("查询超时")]
    QueryTimeout,
    
    #[error("无效的 SQL 查询: {0}")]
    InvalidQuery(String),
}

impl From<AppError> for tonic::Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Tonic(status) => status,
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}

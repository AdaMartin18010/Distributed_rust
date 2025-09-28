use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_address: String,
    pub data_path: String,
    pub log_level: String,
    pub max_connections: u32,
    pub query_timeout_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_address: "0.0.0.0:50051".to_string(),
            data_path: "./data".to_string(),
            log_level: "info".to_string(),
            max_connections: 100,
            query_timeout_seconds: 300,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // 从环境变量加载配置
        let config = AppConfig {
            server_address: env::var("SERVER_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:50051".to_string()),
            data_path: env::var("DATA_PATH")
                .unwrap_or_else(|_| "./data".to_string()),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            query_timeout_seconds: env::var("QUERY_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
        };
        
        Ok(config)
    }
}

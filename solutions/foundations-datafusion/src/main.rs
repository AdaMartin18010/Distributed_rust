use arrow_flight::flight_service_server::{FlightServiceServer, FlightService};
use datafusion::prelude::*;
use foundations::{service, telemetry};
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{info, error};

mod config;
mod error;
mod service_impl;

use config::AppConfig;
use error::AppError;
use service_impl::DfFlightService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化可观测性
    telemetry::init_default();
    
    // 加载配置
    let config = AppConfig::load()?;
    info!("配置加载完成: {:?}", config);
    
    // 构建 DataFusion 上下文
    let ctx = SessionContext::new();
    
    // 注册示例数据表
    if let Err(e) = register_sample_tables(&ctx).await {
        error!("注册示例表失败: {}", e);
        return Err(e.into());
    }
    
    // 创建服务实例
    let svc = DfFlightService::new(ctx);
    
    // 启动服务
    let addr: SocketAddr = config.server_address.parse()?;
    info!("启动 DataFusion 服务在地址: {}", addr);
    
    service::spawn_with_health(
        Server::builder()
            .add_service(FlightServiceServer::new(svc))
            .serve(addr),
    )
    .await?;
    
    Ok(())
}

async fn register_sample_tables(ctx: &SessionContext) -> Result<(), AppError> {
    // 创建示例 CSV 数据
    let sample_data = r#"id,name,age,city
1,Alice,25,New York
2,Bob,30,San Francisco
3,Charlie,35,Chicago
4,Diana,28,Boston
5,Eve,32,Seattle"#;
    
    // 写入临时文件
    let temp_file = tempfile::NamedTempFile::new()?;
    std::fs::write(&temp_file, sample_data)?;
    
    // 注册为 CSV 表
    let df = ctx.read_csv(temp_file.path().to_str().unwrap(), Default::default()).await?;
    ctx.register_table("users", df)?;
    
    info!("示例表 'users' 注册成功");
    Ok(())
}

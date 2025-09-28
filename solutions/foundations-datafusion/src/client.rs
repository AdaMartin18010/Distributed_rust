use arrow_flight::{FlightClient, Ticket};
use tonic::transport::Channel;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 连接到服务
    let channel = Channel::from_static("http://localhost:50051")
        .connect()
        .await?;
    
    let mut client = FlightClient::new(channel);
    
    // 执行示例查询
    let queries = vec![
        "SELECT * FROM users LIMIT 5",
        "SELECT name, age FROM users WHERE age > 30",
        "SELECT city, COUNT(*) as user_count FROM users GROUP BY city",
    ];
    
    for sql in queries {
        info!("执行查询: {}", sql);
        
        match execute_query(&mut client, sql).await {
            Ok(_) => info!("查询执行成功"),
            Err(e) => error!("查询执行失败: {}", e),
        }
        
        println!();
    }
    
    Ok(())
}

async fn execute_query(
    client: &mut FlightClient<Channel>,
    sql: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ticket = Ticket {
        ticket: sql.as_bytes().to_vec(),
    };
    
    let mut stream = client.do_get(ticket).await?;
    
    while let Some(flight_data) = stream.message().await? {
        info!("收到数据: {:?}", flight_data);
    }
    
    Ok(())
}

use arrow_flight::{
    flight_service_server::FlightService,
    FlightData, FlightDescriptor, FlightInfo, HandshakeRequest, HandshakeResponse,
    PutResult, SchemaResult, Ticket,
};
use datafusion::prelude::*;
use std::pin::Pin;
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};
use tracing::{info, error, warn};

use crate::error::AppError;

pub struct DfFlightService {
    ctx: Arc<SessionContext>,
}

impl DfFlightService {
    pub fn new(ctx: SessionContext) -> Self {
        Self {
            ctx: Arc::new(ctx),
        }
    }
}

#[tonic::async_trait]
impl FlightService for DfFlightService {
    type HandshakeStream = Pin<Box<dyn futures::Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
    type ListFlightsStream = Pin<Box<dyn futures::Stream<Item = Result<FlightInfo, Status>> + Send>>;
    type GetFlightInfoStream = Pin<Box<dyn futures::Stream<Item = Result<FlightInfo, Status>> + Send>>;
    type GetSchemaStream = Pin<Box<dyn futures::Stream<Item = Result<SchemaResult, Status>> + Send>>;
    type DoGetStream = Pin<Box<dyn futures::Stream<Item = Result<FlightData, Status>> + Send>>;
    type DoPutStream = Pin<Box<dyn futures::Stream<Item = Result<PutResult, Status>> + Send>>;
    type DoActionStream = Pin<Box<dyn futures::Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
    type ListActionsStream = Pin<Box<dyn futures::Stream<Item = Result<arrow_flight::ActionType, Status>> + Send>>;
    type DoExchangeStream = Pin<Box<dyn futures::Stream<Item = Result<FlightData, Status>> + Send>>;

    async fn handshake(
        &self,
        _request: Request<Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented("handshake not implemented"))
    }

    async fn list_flights(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented("list_flights not implemented"))
    }

    async fn get_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<Self::GetFlightInfoStream>, Status> {
        Err(Status::unimplemented("get_flight_info not implemented"))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<Self::GetSchemaStream>, Status> {
        Err(Status::unimplemented("get_schema not implemented"))
    }

    async fn do_get(
        &self,
        request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        let ticket = request.into_inner();
        let sql = String::from_utf8_lossy(&ticket.ticket);
        
        info!("收到 SQL 查询: {}", sql);
        
        // 验证 SQL 查询
        if sql.trim().is_empty() {
            return Err(Status::invalid_argument("SQL 查询不能为空"));
        }
        
        // 执行查询
        match self.execute_query(&sql).await {
            Ok(stream) => {
                info!("查询执行成功");
                Ok(Response::new(stream))
            }
            Err(e) => {
                error!("查询执行失败: {}", e);
                Err(Status::internal(e.to_string()))
            }
        }
    }

    async fn do_put(
        &self,
        _request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented("do_put not implemented"))
    }

    async fn do_action(
        &self,
        _request: Request<arrow_flight::Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented("do_action not implemented"))
    }

    async fn list_actions(
        &self,
        _request: Request<arrow_flight::Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented("list_actions not implemented"))
    }

    async fn do_exchange(
        &self,
        _request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        Err(Status::unimplemented("do_exchange not implemented"))
    }
}

impl DfFlightService {
    async fn execute_query(&self, sql: &str) -> Result<Self::DoGetStream, AppError> {
        let ctx = self.ctx.clone();
        let sql = sql.to_string();
        
        let stream = async_stream::stream! {
            match ctx.sql(&sql).await {
                Ok(df) => {
                    match df.stream().await {
                        Ok(mut stream) => {
                            while let Some(batch) = stream.next().await {
                                match batch {
                                    Ok(batch) => {
                                        let flight_data = FlightData {
                                            data_header: vec![],
                                            app_metadata: vec![],
                                            data_body: vec![],
                                            flight_descriptor: None,
                                        };
                                        yield Ok(flight_data);
                                    }
                                    Err(e) => {
                                        error!("批次处理错误: {}", e);
                                        yield Err(Status::internal(e.to_string()));
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("流处理错误: {}", e);
                            yield Err(Status::internal(e.to_string()));
                        }
                    }
                }
                Err(e) => {
                    error!("SQL 执行错误: {}", e);
                    yield Err(Status::internal(e.to_string()));
                }
            }
        };
        
        Ok(Box::pin(stream))
    }
}

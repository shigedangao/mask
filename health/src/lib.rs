use proto::{health_service_server::{HealthService, HealthServiceServer}, HealthResponse, HealthParam};
use tonic::{Request, Response, Status};
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("healthcheck");
}

#[derive(Default)]
struct HealthHandler {}

#[tonic::async_trait]
impl HealthService for HealthHandler {
    async fn is_healthy(
        &self,
        _: Request<HealthParam>
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            is_healthy: true
        }))
    }
}

pub async fn run_health_server() -> Result<(), tonic::transport::Error> {    
    let addr = utils::get_server_addr(5001).parse().unwrap();
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<HealthServiceServer<HealthHandler>>()
        .await;

    let health = HealthHandler::default();

    Server::builder()
        .add_service(health_service)
        .add_service(HealthServiceServer::new(health))
        .serve(addr)
        .await
}

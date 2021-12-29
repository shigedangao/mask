use tonic::transport::{Server, Identity, ServerTlsConfig};
use tonic_health::ServingStatus;
use std::sync::Arc;
use utils;

#[macro_use]
extern crate log;

mod hospital;
mod err;

use hospital::proto_newcase::case_service_server::CaseServiceServer;
use hospital::proto_hospital::care_status_server::CareStatusServer;
use hospital::status::CareService;
use hospital::case::CaseServiceHandle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = utils::setup_services(9000)?;

    info!("Connecting to the database");
    let db_pool = db::connect("../config.toml").await?;
    let db_handle = Arc::new(db_pool);
    
    // load tls certificate
    let (cert, key) = utils::get_certificates()?; 
    let identity = Identity::from_pem(cert, key);

    // creating healthcheck service
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_service_status("Hospital", ServingStatus::Serving)
        .await;

    // setup the server
    let addr = addr.parse()?;
    info!("Server is running on port 9000");

    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(health_service)
        .add_service(CareStatusServer::new(CareService{
            pool: Arc::clone(&db_handle)
        }))
        .add_service(CaseServiceServer::new(CaseServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .serve(addr)
        .await?;

    Ok(())
}

use tonic::transport::{Server, Identity, ServerTlsConfig};
use std::sync::Arc;
use utils;

#[macro_use]
extern crate log;

mod hospital;
mod err;

use hospital::care::care_status_server::CareStatusServer;
use hospital::status::CareService;
use hospital::case::cases::case_service_server::CaseServiceServer;
use hospital::case::CaseServiceHandle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = utils::setup_services(9000)?;

    info!("Connecting to the database");
    let db_pool = db::connect("../config.toml").await?;
    let db_handle = Arc::new(db_pool);
    
    // load tls certificate
    let cert = tokio::fs::read("../keys/server-cert.pem").await?;
    let key = tokio::fs::read("../keys/server-key.key").await?;
    let identity = Identity::from_pem(cert, key);

    // setup the server
    let addr = addr.parse()?;
    info!("Server is running on port 9000");

    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
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

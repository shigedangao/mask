use tonic::transport::{Server, Identity, ServerTlsConfig};
use std::sync::Arc;

#[macro_use]
extern crate log;

mod hospital;
mod err;
mod mix;
mod icu;

use hospital::proto_newcase::case_service_server::CaseServiceServer;
use hospital::proto_hospital::care_status_server::CareStatusServer;
use hospital::status::CareService;
use hospital::case::CaseServiceHandle;
use mix::proto_mix::mix_service_server::MixServiceServer;
use mix::mix::MixHandler;
use icu::proto_icu::icu_service_server::IcuServiceServer;
use icu::icu::IcuHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    utils::setup_services("mask")?;

    info!("Connecting to the database");
    let db_pool = db::connect("../config.toml").await?;
    let db_handle = Arc::new(db_pool);
    
    // load tls certificate
    let (cert, key) = utils::get_certificates()?; 
    let identity = Identity::from_pem(cert, key);

    // setup the server
    let addr = utils::get_server_addr(9000).parse()?;
    let server = Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(CareStatusServer::new(CareService{
            pool: Arc::clone(&db_handle)
        }))
        .add_service(CaseServiceServer::new(CaseServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(MixServiceServer::new(MixHandler {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(IcuServiceServer::new(IcuHandler {
            pool: Arc::clone(&db_handle)
        }))
        .serve(addr);

    info!("Server is running on port 9000 & Healthcheck server port 5601");
    tokio::try_join!(server, health::run_health_server())?;

    Ok(())
}

use tonic::transport::{Server, Identity, ServerTlsConfig};
use tonic_health::ServingStatus;
use std::sync::Arc;

#[macro_use]
extern crate log;

mod pcr;
mod positivity;
mod err;

use pcr::{
    region::PcrServiceHandle,
    dep::PcrServiceDepHandle,
    proto::pcr_service_region_server::PcrServiceRegionServer,
    proto::pcr_service_department_server::PcrServiceDepartmentServer
};
use positivity::{
    dep::PosServiceHandle,
    proto::positivity_rate_server::PositivityRateServer
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = utils::setup_services(9090)?;

    info!("Connecting to the database");
    let db_pool = db::connect("../config.toml").await?;
    let db_handle = Arc::new(db_pool);

    // load tls certificate
    let (cert, key) = utils::get_certificates()?;
    let identity = Identity::from_pem(cert, key);

    // creating healthcheck service
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_service_status("Pcr", ServingStatus::Serving)
        .await;

    let addr = addr.parse()?;
    info!("Starting the server port 9090");
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(health_service)
        .add_service(PcrServiceRegionServer::new(PcrServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(PcrServiceDepartmentServer::new(PcrServiceDepHandle {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(PositivityRateServer::new(PosServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .serve(addr)
        .await?;
    
    Ok(())
}

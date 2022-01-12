use tonic::transport::{Server, Identity, ServerTlsConfig};
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
    utils::setup_services("pcr")?;

    info!("Connecting to the database");
    let db_pool = db::connect("../config.toml").await?;
    let db_handle = Arc::new(db_pool);

    // load tls certificate
    let (cert, key) = utils::get_certificates()?;
    let identity = Identity::from_pem(cert, key);

    let addr = utils::get_server_addr(9090).parse()?;
    let server = Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(PcrServiceRegionServer::new(PcrServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(PcrServiceDepartmentServer::new(PcrServiceDepHandle {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(PositivityRateServer::new(PosServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .serve(addr);

    info!("Starting the server port 9090 & Healthcheck server port 5601");
    tokio::try_join!(server, health::run_health_server())?;

    Ok(())
}

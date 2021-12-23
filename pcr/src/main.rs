use tonic::transport::{Server, Identity, ServerTlsConfig};
use std::sync::Arc;

#[macro_use]
extern crate log;

mod pcr;
mod err;

use pcr::{
    region::PcrServiceHandle,
    dep::PcrServiceDepHandle,
    pcr_test::pcr_service_region_server::PcrServiceRegionServer,
    pcr_test::pcr_service_department_server::PcrServiceDepartmentServer
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = utils::setup_services(9090)?;

    info!("Connecting to the database");
    let db_pool = db::connect("../config.toml").await?;
    let db_handle = Arc::new(db_pool);

    // load tls certificate
    let cert = tokio::fs::read("../keys/server-cert.pem").await?;
    let key = tokio::fs::read("../keys/server-key.key").await?;
    let identity = Identity::from_pem(cert, key);

    let addr = addr.parse()?;
    info!("Starting the server port 9090");
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(PcrServiceRegionServer::new(PcrServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(PcrServiceDepartmentServer::new(PcrServiceDepHandle {
            pool: Arc::clone(&db_handle)
        }))
        .serve(addr)
        .await?;
    
    Ok(())
}
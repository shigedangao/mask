use tonic::transport::Server;
use std::sync::Arc;

#[macro_use]
extern crate log;

mod pcr;
mod positivity;
mod common;

use pcr::{
    polymerase::PcrServiceHandle,
    proto::pcr_service_server::PcrServiceServer
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

    let addr = utils::get_server_addr(9090).parse()?;
    let server = Server::builder()
        .add_service(PcrServiceServer::new(PcrServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .add_service(PositivityRateServer::new(PosServiceHandle {
            pool: Arc::clone(&db_handle)
        }))
        .serve(addr);

    info!("Starting the server port 9090 & Healthcheck server port 5601");
    tokio::try_join!(server, health::run_health_server())?;
    //tokio::try_join!(server)?;

    Ok(())
}

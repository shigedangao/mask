use color_eyre::eyre::Result;
use tonic::transport::Server;
use std::sync::Arc;

#[macro_use]
extern crate log;

mod hospital;
mod err;
mod util;

use hospital::care::care_status_server::CareStatusServer;
use hospital::CareService;
use hospital::case::cases::case_service_server::CaseServiceServer;
use hospital::case::CaseServiceHandle;

// Setup the logging library
fn setup() -> Result<()> {
    // set RUST_LOG based on the environment variable
    match std::env::var("mode") {
        Ok(res) => {
            if res == "prod" {
                std::env::set_var("RUST_LOG", "warn");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }
        },
        Err(_) => std::env::set_var("RUST_LOG", "info")
    }

    // set environment variable for log debugging 
    std::env::set_var("RUST_BACKTRACE", "1");

    color_eyre::install()?;
    env_logger::init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;

    info!("Connecting to the database");
    let db_pool = db::connect("config.toml").await?;
    let db_handle = Arc::new(db_pool); 

    info!("Server is running on port 9000");
    // setup the server
    let addr = "[::1]:9000".parse()?;
    Server::builder()
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

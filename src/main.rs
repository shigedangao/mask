use color_eyre::eyre::Result;
use tonic::transport::Server;

#[macro_use]
extern crate log;

mod hospital;

use hospital::care::care_status_server::CareStatusServer;
use hospital::CareService;

// Setup the logging library
fn setup() -> Result<()> {
    // set environment variable for log debugging
    std::env::set_var("RUST_LOG", "INFO");
    std::env::set_var("RUST_BACKTRACE", "1");

    color_eyre::install()?;
    env_logger::init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;

    info!("Server is running on port 9000");
    // setup the server
    let addr = "[::1]:9000".parse()?;
    Server::builder()
        .add_service(CareStatusServer::new(CareService::default()))
        .serve(addr)
        .await?;

    Ok(())
}

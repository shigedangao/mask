use err::DBError;
use sqlx::{postgres::{PgPoolOptions}, Postgres, Pool};
use toml::Value;
use std::fs;

#[macro_use]
extern crate log;

pub mod err;
pub mod query;

pub type PGPool = Pool<Postgres>;

// Constant
const MAX_CONNECTIONS: u32 = 5;
const ENV_DB_USERNAME: &str = "db_username";
const ENV_DB_PASSWORD: &str = "db_password";
const ENV_DB_HOST: &str = "db_host";
const ENV_DB_PORT: &str = "db_port";
const ENV_DB_NAME: &str = "db_name";

/// Build the production database URI based os environment variables
fn build_database_uri_from_env() -> Result<String, DBError> {
    let username = std::env::var(ENV_DB_USERNAME)?;
    let password = std::env::var(ENV_DB_PASSWORD)?;
    let host = std::env::var(ENV_DB_HOST)?;
    let port = std::env::var(ENV_DB_PORT)?;
    let db_name = std::env::var(ENV_DB_NAME)?;

    let connection_uri = format!(
        "postgres://{}:{}@{}:{}/{}",
        username,
        password,
        host,
        port,
        db_name
    );

    info!(target: "util", "Load environment variables from env");

    Ok(connection_uri)
}

/// Build environment variable based on the config.toml located at the root of project
/// 
/// # Arguments
/// * `filepath` - &str
/// 
/// # Examples
/// Example of config.toml
/// 
/// ```toml
/// db_username=""
/// db_password=""
/// db_host=""
/// db_port=""
/// db_name=""
/// ```
fn build_dev_database_uri(filepath: &str) -> Result<String, DBError> {
    let config = fs::read_to_string(filepath)?;
    let values = config.parse::<Value>()
        .map_err(|err| DBError::IO(err.to_string()))?;

    let username = values[ENV_DB_USERNAME].as_str()
        .ok_or_else(|| DBError::MissingEnv(ENV_DB_USERNAME.into()))?;

    let password = values[ENV_DB_PASSWORD].as_str()
        .ok_or_else(|| DBError::MissingEnv(ENV_DB_PASSWORD.into()))?;

    let host = values[ENV_DB_HOST].as_str()
        .ok_or_else(|| DBError::MissingEnv(ENV_DB_HOST.into()))?;

    let port = values[ENV_DB_PORT].as_str()
        .ok_or_else(|| DBError::MissingEnv(ENV_DB_PORT.into()))?;

    let db_name = values[ENV_DB_NAME].as_str()
        .ok_or_else(|| DBError::MissingEnv(ENV_DB_NAME.into()))?;

    let connection_uri = format!(
        "postgres://{}:{}@{}:{}/{}",
        username,
        password,
        host,
        port,
        db_name
    );

    info!(target: "util", "Load environment variables from config.toml");

    Ok(connection_uri)
}

/// Build connection uri based on the `env` environment variable
/// 
/// # Arguments
/// * `filepath` - &str
fn get_connection_uri(filepath: &str) -> Result<String, DBError> {
    if let Ok(mode) = std::env::var("rust_env") {
        if mode == "prod" {
            return build_database_uri_from_env();
        }
    }

    // return dev otherwise
    // if error load production var
    build_dev_database_uri(filepath)
        .or(build_database_uri_from_env())
}

/// Create a connection handler with the targeted database
/// 
/// # Arguments
/// * `filepath` - &str
pub async fn connect(filepath: &str) -> Result<Pool<Postgres>, DBError> {
    let database_uri = get_connection_uri(filepath)?;
    let pool = PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(&database_uri)
        .await?;

    Ok(pool)
}


#[derive(Debug)]
pub enum DBError {
    MissingEnv(String),
    IO(String),
    Connection(String),
    Exec,
}

impl std::error::Error for DBError {}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DBError::MissingEnv(msg) => write!(f, "Unable to build database uri. Error: {}", msg),
            DBError::IO(msg) => write!(f, "Unable to perform IO operation, {}", msg),
            DBError::Connection(msg) => write!(f, "Unable to connect to the database, reason: {}", msg),
            DBError::Exec => write!(f, "Error while parsing result")
        }
    }
}

impl From<std::env::VarError> for DBError {
    fn from(err: std::env::VarError) -> Self {
        DBError::MissingEnv(err.to_string())
    }
}

impl From<std::io::Error> for DBError {
    fn from(err: std::io::Error) -> Self {
        DBError::IO(err.to_string())
    }
}

impl From<sqlx::Error> for DBError {
    fn from(err: sqlx::Error) -> Self {
        DBError::Connection(err.to_string())
    }
}


#[derive(Debug)]
pub enum PcrErr {
    QueryError(String)
}

impl std::fmt::Display for PcrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcrErr::QueryError(reason) => write!(f, "An error happened while fetching data, {:?}", reason)
        }
    }
}

impl std::error::Error for PcrErr {}

impl From<sqlx::Error> for PcrErr {
    fn from(err: sqlx::Error) -> Self {
        PcrErr::QueryError(err.to_string())
    }
}

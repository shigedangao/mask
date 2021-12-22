#[derive(Debug)]
pub enum MaskErr {
    QueryError(String)
}

impl std::fmt::Display for MaskErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaskErr::QueryError(reason) => write!(f, "An error happened while fetching data, {:?}", reason)
        }
    }
}

impl std::error::Error for MaskErr {}

impl From<sqlx::Error> for MaskErr {
    fn from(err: sqlx::Error) -> Self {
        MaskErr::QueryError(err.to_string())
    }
}

#[derive(Debug)]
pub enum MaskErr {
    MalformattedDate,
    QueryError(String)
}

impl std::fmt::Display for MaskErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaskErr::MalformattedDate => write!(f, "Date is malformatted. Check the parameter"),
            MaskErr::QueryError(reason) => write!(f, "An error happened while fetching data, {:?}", reason)
        }
    }
}

impl std::error::Error for MaskErr {}

impl From<chrono::ParseError> for MaskErr {
    fn from(_: chrono::ParseError) -> Self {
        MaskErr::MalformattedDate
    }
}


impl From<sqlx::Error> for MaskErr {
    fn from(err: sqlx::Error) -> Self {
        MaskErr::QueryError(err.to_string())
    }
}

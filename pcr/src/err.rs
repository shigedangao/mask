use tonic::Status;

#[derive(Debug)]
pub enum PcrErr {
    QueryError(String),
    InvalidDate,
    MissingParam(String)
}

impl std::fmt::Display for PcrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcrErr::QueryError(reason) => write!(f, "An error happened while fetching data, {:?}", reason),
            PcrErr::InvalidDate => write!(f, "The date is invalid"),
            PcrErr::MissingParam(name) => write!(f, "The request is not valid. Param {:?} is missing", name)
        }
    }
}

impl std::error::Error for PcrErr {}

impl From<sqlx::Error> for PcrErr {
    fn from(err: sqlx::Error) -> Self {
        PcrErr::QueryError(err.to_string())
    }
}

impl From<PcrErr> for Status {
    fn from(err: PcrErr) -> Self {
        match err {
            PcrErr::QueryError(msg) => Status::internal(msg),
            PcrErr::InvalidDate => Status::invalid_argument("The date is invalid"),
            PcrErr::MissingParam(msg) => Status::invalid_argument(format!("The request is not valid. Param {} is missing", msg))
        }
    }
}

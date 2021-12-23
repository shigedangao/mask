use tonic::{Code, Status};

#[derive(Debug)]
pub enum PcrErr {
    QueryError(String),
    InvalidDate
}

impl std::fmt::Display for PcrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcrErr::QueryError(reason) => write!(f, "An error happened while fetching data, {:?}", reason),
            PcrErr::InvalidDate => write!(f, "The date is invalid")
        }
    }
}

impl std::error::Error for PcrErr {}

impl From<sqlx::Error> for PcrErr {
    fn from(err: sqlx::Error) -> Self {
        PcrErr::QueryError(err.to_string())
    }
}

impl Into<Status> for PcrErr {
    fn into(self) -> Status {
        match self {
            PcrErr::InvalidDate => Status::new(Code::InvalidArgument, "Date is invalid"),
            PcrErr::QueryError(msg) => Status::new(Code::Internal, format!("An error happened while getting {}", msg))
        }
    }
}

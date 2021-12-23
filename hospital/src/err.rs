use tonic::{Code, Status};

#[derive(Debug)]
pub enum MaskErr {
    QueryError(String),
    InvalidDate
}

impl std::fmt::Display for MaskErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaskErr::QueryError(reason) => write!(f, "An error happened while fetching data, {:?}", reason),
            MaskErr::InvalidDate => write!(f, "The date is invalid")
        }
    }
}

impl std::error::Error for MaskErr {}

impl From<sqlx::Error> for MaskErr {
    fn from(err: sqlx::Error) -> Self {
        MaskErr::QueryError(err.to_string())
    }
}

impl Into<Status> for MaskErr {
    fn into(self) -> Status {
        match self {
            MaskErr::InvalidDate => Status::new(Code::InvalidArgument, "Date is invalid"),
            MaskErr::QueryError(msg) => Status::new(Code::Internal, format!("An error happened while getting {}", msg))
        }
    }
}

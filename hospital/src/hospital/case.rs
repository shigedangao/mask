use tonic::{Request, Response, Status, Code};
use cases::case_service_server::CaseService;
use cases::{CaseInput, NewCases, NewCase};
use std::sync::Arc;
use futures::TryStreamExt;
use db::PGPool;
use crate::err::MaskErr;

use super::util;

// import generated struct by tonic
pub mod cases {
    tonic::include_proto!("newcase");
}

pub struct CaseServiceHandle {
    pub pool: Arc<PGPool>
}

#[derive(sqlx::FromRow, Debug)]
struct QueryResult {
    jour: Option<String>,
    incid_hosp: Option<i64>,
    incid_rea: Option<i64>,
    incid_dc: Option<i64>,
    incid_rad: Option<i64>
}

impl From<QueryResult> for NewCase {
    fn from(q: QueryResult) -> Self {
        Self {
            date: q.jour.unwrap_or_default(),
            new_entry_hospital: q.incid_hosp.unwrap_or_default(),
            new_entry_icu: q.incid_rea.unwrap_or_default(),
            death: q.incid_dc.unwrap_or_default(),
            healed: q.incid_rad.unwrap_or_default()
        }
    }
}

impl util::Date for CaseInput {
    fn build_date(&self) -> (String, bool) {
        match self.day.to_owned() {
            Some(day) => (format!("{}-{}-{}", self.year, self.month, day), true),
            None => (format!("{}-{}", self.year, self.month), false)
        }
    }
}

#[tonic::async_trait]
impl CaseService for CaseServiceHandle {
    /// Return the number of new case by department.
    /// The day is optional. Hence we can query either per day or per month
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<CaseInput>
    async fn get_new_case_by_department(
        &self,
        request: Request<CaseInput>
    ) -> Result<Response<NewCases>, Status> {
        let input = request.into_inner();
        let date = match util::is_date_valid(&input) {
            Ok(date) => date,
            Err(_) => {
                return Err(Status::new(Code::InvalidArgument, "The date is invalid"))
            }
        };        

        let reply = match get_new_cases_by_department(&self.pool, date, input.department).await {
            Ok(cases) => NewCases { cases },
            Err(err) => {
                error!("fetch new cases error: {:?}", err);
                return Err(Status::new(Code::Internal, "Unable to fetch new cases in hospital"));
            }
        };

        Ok(Response::new(reply))
    }
}

/// Query the database to get the new cases rate in hospital by department
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `date` - String
/// * `department` - String 
async fn get_new_cases_by_department(pool: &PGPool, date: String, department: String) -> Result<Vec<NewCase>, MaskErr> {
    let mut cases = Vec::new();
    let date_like = format!("{}%", date);

    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM cases WHERE jour LIKE $1 AND dep = $2")
        .bind(date_like)
        .bind(department)
        .fetch(pool);

    while let Some(row) = stream.try_next().await? {
        cases.push(NewCase::from(row))
    }
    
    Ok(cases)
}

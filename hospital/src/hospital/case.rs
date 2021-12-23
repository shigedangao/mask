use tonic::{Request, Response, Status};
use cases::case_service_server::CaseService;
use cases::{CaseInput, NewCases, NewCase};
use std::sync::Arc;
use futures::TryStreamExt;
use db::PGPool;
use crate::err::MaskErr;
use utils::Date;

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

impl Date for CaseInput {
    fn get_year(&self) -> i32 {
        self.year
    }

    fn get_month(&self) -> i32 {
        self.month
    }

    fn get_day(&self) -> Option<i32> {
        self.day
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
        let date = match input.build_date() {
            Some(date) => date,
            None => return Err(MaskErr::InvalidDate.into())
        };        

        match get_new_cases_by_department(&self.pool, date, input.department).await {
            Ok(cases) => Ok(Response::new(NewCases { cases })),
            Err(err) => {
                error!("fetch new cases error: {:?}", err);
                return Err(MaskErr::QueryError("new case by department".into()).into());
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_to_query_new_cases() {
        let pool = db::connect("../config.toml").await.unwrap();
        let res = get_new_cases_by_department(
            &pool,
            "2021-12-12".to_owned(),
            "77".to_owned()
        ).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_response() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let case_service = CaseServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = CaseInput {
            day: Some(12),
            month: 12,
            year: 2021,
            department: "77".to_owned()
        };

        let request = Request::new(input);
        let res = case_service.get_new_case_by_department(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_error() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let case_service = CaseServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = CaseInput {
            day: Some(50),
            month: 12,
            year: 2021,
            department: "77".to_owned()
        };

        let request = Request::new(input);
        let res = case_service.get_new_case_by_department(request).await;

        assert!(res.is_err());
    }
}

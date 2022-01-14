use sqlx::Row;
use sqlx::postgres::PgRow;
use tonic::{Request, Response, Status};
use std::sync::Arc;
use db::PGPool;
use db::query;
use utils::Date;
use crate::err::MaskErr;

// import generated struct by tonic
use super::proto_newcase::{CaseInput, NewCases, CaseResult};
use super::proto_newcase::case_service_server::CaseService;

pub struct CaseServiceHandle {
    pub pool: Arc<PGPool>
}

impl TryFrom<PgRow> for CaseResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let res = Self {
            date: value.try_get("jour")?,
            new_entry_hospital: value.try_get("incid_hosp")?,
            new_entry_icu: value.try_get("incid_rea")?,
            death: value.try_get("incid_dc")?,
            back_home: value.try_get("incid_rad")?
        };

        Ok(res)
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
        let date = match input.build_date_sql_like() {
            Some(date) => date,
            None => return Err(MaskErr::InvalidDate.into())
        };

        match query::get_all_by_date_and_gen_field::<CaseResult, String>(
            &self.pool,
            "SELECT * FROM cases WHERE jour LIKE $1 AND dep = $2",
            &date,
            input.department
        ).await {
            Ok(cases) => Ok(Response::new(NewCases { cases })),
            Err(err) => {
                error!("fetch new cases error: {:?}", err);
                return Err(MaskErr::QueryError("new case by department".into()).into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

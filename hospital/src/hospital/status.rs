use std::sync::Arc;
use sqlx::{
    postgres::{PgRow},
    Row
};
use tonic::{Request, Response, Status};
use db::{PGPool, query};
use utils::Date;
use crate::err::MaskErr;

use super::proto_hospital::{CareStatusResult, CareStatusInput, CareStatusOutput};
use super::proto_hospital::care_status_server::CareStatus;

// Hold a pool of connection
#[derive(Debug)]
pub struct CareService {
    pub pool: Arc<PGPool>
}

impl TryFrom<PgRow> for CareStatusResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let res = Self {
            region: value.try_get("reg")?,
            age: value.try_get("cl_age90")?,
            hospitalization: value.try_get("hosp").unwrap_or_default(),
            icu: value.try_get("rea").unwrap_or_default(),
            back_home: value.try_get("rad").unwrap_or_default(),
            death: value.try_get("dc").unwrap_or_default(),
            different_care_services: value.try_get("ssr_usld").ok(),
            conventional_care: value.try_get("hospconv").ok(),
            other_care_district: value.try_get("autres").ok(),
            day: value.try_get("jour")?
        };

        Ok(res)
    }
}

impl Date for CareStatusInput {
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
impl CareStatus for CareService {
    /// Return the number of case in hospital for a date and a region
    /// The day is optional. Hence we can query either per day or per month
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<CareStatusInput>
    async fn get_hospital_status_by_region(
        &self,
        request: Request<CareStatusInput>
    ) -> Result<Response<CareStatusOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(date) => date,
            None => return Err(MaskErr::InvalidDate.into())
        };

        match query::get_all_by_date_and_gen_field::<CareStatusResult, i32>(
            &self.pool,
            "SELECT * FROM hospitalization WHERE jour LIKE $1 AND reg = $2",
            &date,
            input.region
        ).await {
            Ok(cases) => Ok(Response::new(CareStatusOutput { cases })),
            Err(err) => {
                error!("fetch hospitalization {:?}", err);
                return Err(MaskErr::QueryError("hospitalization by region".into()).into());
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
        let care_service = CareService {
            pool: Arc::clone(&pool_arc)
        };

        let input = CareStatusInput {
            day: Some(12),
            month: 12,
            year: 2021,
            region: 11 
        };

        let request = Request::new(input);
        let res = care_service.get_hospital_status_by_region(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_error() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let care_service = CareService {
            pool: Arc::clone(&pool_arc)
        };

        let input = CareStatusInput {
            day: None,
            month: 32,
            year: 2021,
            region: 11 
        };

        let request = Request::new(input);
        let res = care_service.get_hospital_status_by_region(request).await;

        assert!(res.is_err());
    }
}

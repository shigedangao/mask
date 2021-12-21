use tonic::{Request, Response, Status, Code};
use futures::TryStreamExt;
use care::care_status_server::CareStatus;
use care::{CareStatusPayload, CareStatusInput, CareStatusOutput};
use db::PGPool;
use std::sync::Arc;
use super::util;
use super::err::MaskErr;

pub mod case;

// import generated struct by tonic
pub mod care {
    tonic::include_proto!("hospital");
}

// Hold a pool of connection
#[derive(Debug)]
pub struct CareService {
    pub pool: Arc<PGPool>
}

#[derive(sqlx::FromRow, Debug)]
struct QueryResult {
    reg: Option<i64>,
    cl_age90: Option<i64>,
    hosp: Option<i64>,
    rea: Option<i64>,
    hospconv: Option<f64>,
    ssr_usld: Option<f64>,
    autres: Option<f64>,
    rad: Option<i64>,
    dc: Option<i64>,
    jour: Option<String>
}

impl From<QueryResult> for CareStatusPayload {
    fn from(q: QueryResult) -> Self {
        Self {
            region: q.reg.unwrap_or_default(),
            age: q.cl_age90.unwrap_or_default(),
            hospitalization: q.hosp.unwrap_or_default(),
            icu: q.rea.unwrap_or_default(),
            healed: q.rad.unwrap_or_default(),
            death: q.dc.unwrap_or_default(),
            different_care_services: q.ssr_usld,
            conventional_care: q.hospconv,
            other_care_district: q.autres,
            day: q.jour.unwrap_or_default()
        }
    }
}

impl util::Date for CareStatusInput {
    fn build_date(&self) -> (String, bool) {
        match self.day.to_owned() {
            Some(day) => (format!("{}-{}-{}", self.year, self.month, day), true),
            None => (format!("{}-{}", self.year, self.month), false)
        }
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
        let date = match util::is_date_valid(&input) {
            Ok(date) => date,
            Err(_) => {
                return Err(Status::new(Code::InvalidArgument, "The date is invalid"))
            }
        };

        let reply = match get_cases_by_day_and_region(&self.pool, date, input.region).await {
            Ok(cases) => CareStatusOutput { cases },
            Err(err) => {
                error!("fetch hospitalization {:?}", err);
                return Err(Status::new(Code::Internal, "Unable to retrieve hospitalization cases by day"));
            }
        };

        Ok(Response::new(reply))
    }
}

/// Query the database to get the hospitalization rate for a day and a region
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `input` - CareStatusInput
async fn get_cases_by_day_and_region(pool: &PGPool, date: String, region: i32) -> Result<Vec<CareStatusPayload>, MaskErr> {
    let mut cases = Vec::new();
    let date_like = format!("{}%", date);

    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM hospitalization WHERE jour LIKE $1 AND reg = $2")
        .bind(date_like)
        .bind(region)
        .fetch(pool);

    while let Some(row) = stream.try_next().await? {
        let case = CareStatusPayload::from(row);
        cases.push(case);
    }

    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_to_query_hospitalization_rate() {
        let pool = db::connect("../config.toml").await.unwrap();
        let res = get_cases_by_day_and_region(
            &pool, 
            "2021-12-12".to_owned(), 
            11
        ).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_response() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let care_service = CareService {
            pool: Arc::clone(&pool_arc)
        };

        let input = CareStatusInput {
            day: Some("12".to_owned()),
            month: "12".to_owned(),
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
            month: "32".to_owned(),
            year: 2021,
            region: 11 
        };

        let request = Request::new(input);
        let res = care_service.get_hospital_status_by_region(request).await;

        assert!(res.is_err());
    }
}

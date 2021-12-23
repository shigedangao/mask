use std::sync::Arc;
use futures::TryStreamExt;
use tonic::{Request, Response, Status, Code};
use db::PGPool;
use utils::Date;
use crate::err::PcrErr;

use super::pcr_test::pcr_service_region_server::PcrServiceRegion;
use super::pcr_test::{PcrInputRegion, PcrOutput, PcrResult};

pub struct PcrServiceHandle {
    pub pool: Arc<PGPool>
}

#[derive(sqlx::FromRow, Debug)]
pub struct QueryResult {
    reg: Option<i64>,
    jour: Option<String>,
    p_f: Option<i64>,
    p_h: Option<i64>,
    p: Option<i64>,
    t: Option<i64>,
    t_f: Option<i64>,
    t_h: Option<i64>,
    cl_age90: Option<i64>,
    pop: Option<f64>
}

impl From<QueryResult> for PcrResult {
    fn from(q: QueryResult) -> Self {
        Self {
            day: q.jour.unwrap_or_default(),
            age: q.cl_age90.unwrap_or_default(),
            region: q.reg,
            population_by_region: q.pop,
            positive_pcr_test_male: q.p_h,
            positive_pcr_test_female: q.p_f,
            total_positive_pcr_test: q.p,
            pcr_test_male: q.t_h,
            pcr_test_female: q.t_f,
            total_pcr_test_done: q.t,
            ..Default::default()
        }
    }
}

impl Date for PcrInputRegion {
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
impl PcrServiceRegion for PcrServiceHandle {
    /// Retrieve PCR test made by region
    /// The day is optional. Hence we can query either per day or per month
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<PcrInput>
    async fn get_pcr_test_made_by_region(
        &self,
        request: Request<PcrInputRegion>
    ) -> Result<Response<PcrOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date() {
            Some(res) => res,
            None => return Err(PcrErr::InvalidDate.into())
        };

        match get_pcr_test_by_region(&self.pool, date, input.region).await {
            Ok(pcr) => Ok(Response::new(PcrOutput { pcr })),
            Err(err) => {
                error!("fetch pcr test by region {:?}", err);
                Err(Status::new(Code::Internal, "Unable to retrieve pcr case by region"))
            }
        }
    }
}

/// Retrieve the PCR test by Region
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `date` - String
/// * `region` - i32
async fn get_pcr_test_by_region(pool: &PGPool, date: String, region: i32) -> Result<Vec<PcrResult>, PcrErr> {
    let mut tests = Vec::new();
    let date = format!("{}%", date);

    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM pcr_test_region WHERE jour LIKE $1 AND reg = $2")
        .bind(date)
        .bind(region)
        .fetch(pool);

    while let Some(row) = stream.try_next().await? {
        let test = PcrResult::from(row);
        tests.push(test) 
    }

    Ok(tests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_to_query_pcr_case() {
        let pool = db::connect("../config.toml").await.unwrap();
        let res = get_pcr_test_by_region(
            &pool,
            "2021-12-09".to_string(),
            93
        ).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_response() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PcrServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInputRegion {
            day: Some(12),
            month: 12,
            year: 2021,
            region: 93
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_by_region(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_error() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PcrServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInputRegion {
            day: Some(12),
            month: -1000,
            year: 2021,
            region: 93
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_by_region(request).await;

        assert!(res.is_err());
    }
}

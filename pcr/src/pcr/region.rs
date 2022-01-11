use std::sync::Arc;
use tonic::{Request, Response, Status, Code};
use db::{PGPool, query};
use utils::Date;
use crate::err::PcrErr;

use super::proto::pcr_service_region_server::PcrServiceRegion;
use super::proto::{PcrInputRegion, PcrOutput, PcrResult};

pub struct PcrServiceHandle {
    pub pool: Arc<PGPool>
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
        let date = match input.build_date_sql_like() {
            Some(res) => res,
            None => return Err(PcrErr::InvalidDate.into())
        };

        match query::get_all_by_date_and_gen_field::<PcrResult, i32>(
            &self.pool,
            "SELECT * FROM pcr_test_region WHERE jour LIKE $1 AND reg = $2",
            &date,
            input.region
        ).await {
            Ok(pcr) => Ok(Response::new(PcrOutput { pcr })),
            Err(err) => {
                error!("fetch pcr test by region {:?}", err);
                Err(Status::new(Code::Internal, "Unable to retrieve pcr case by region"))
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

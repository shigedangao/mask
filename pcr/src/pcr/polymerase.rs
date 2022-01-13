use std::sync::Arc;
use db::{PGPool, query};
use sqlx::{
    postgres::PgRow,
    Row
};
use tonic::{Request, Response, Status};
use utils::Date;
use crate::err::PcrErr;
use super::proto::{
    pcr_service_server::PcrService,
    PcrInput, PcrOutput, PcrResult
};

pub struct PcrServiceHandle {
    pub pool: Arc<PGPool>
}

impl TryFrom<PgRow> for PcrResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let res = Self {
            age: value.try_get("cl_age90")?,
            day: value.try_get("jour")?,
            department: value.try_get("dep").ok(),
            population_by_department: value.try_get("pop").ok(),
            total_pcr_test_done: value.try_get("t").ok(),
            total_positive_pcr_test: value.try_get("p").ok(),
            // usually used by region
            positive_pcr_test_female: value.try_get("p_f").ok(),
            positive_pcr_test_male: value.try_get("p_h").ok(),
            pcr_test_female: value.try_get("t_f").ok(),
            pcr_test_male: value.try_get("t_h").ok(),
            region: value.try_get("reg").ok(),
            population_by_region: value.try_get("pop").ok()
        };

        Ok(res)
    }
}

impl Date for PcrInput {
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
impl PcrService for PcrServiceHandle {
    /// Get the list of pcr test made per department
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<PcrInputDepartment>
    async fn get_pcr_test_made_by_department(
        &self,
        request: Request<PcrInput>
    ) -> Result<Response<PcrOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(d) => d,
            None => return Err(PcrErr::InvalidDate.into())
        };

        let department = match input.department {
            Some(dep) => dep,
            None => return Err(PcrErr::MissingParam("department".to_owned()).into())
        };

        match query::get_all_by_date_and_gen_field::<PcrResult, &str>(
            &self.pool,
            "SELECT * FROM pcr_test_department WHERE jour LIKE $1 AND dep = $2",
            &date,
            &department
        ).await {
            Ok(pcr) => Ok(Response::new(PcrOutput { pcr })),
            Err(err) => {
                error!("fetch pcr by department {:?}", err);
                return Err(PcrErr::QueryError("pcr by department".into()).into());
            }
        }
    }

    /// Retrieve PCR test made by region
    /// The day is optional. Hence we can query either per day or per month
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<PcrInput>
    async fn get_pcr_test_made_by_region(
        &self,
        request: Request<PcrInput>
    ) -> Result<Response<PcrOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(res) => res,
            None => return Err(PcrErr::InvalidDate.into())
        };

        let region = match input.region {
            Some(reg) => reg,
            None => return Err(PcrErr::MissingParam("region".to_owned()).into())
        };

        match query::get_all_by_date_and_gen_field::<PcrResult, i32>(
            &self.pool,
            "SELECT * FROM pcr_test_region WHERE jour LIKE $1 AND reg = $2",
            &date,
            region
        ).await {
            Ok(pcr) => Ok(Response::new(PcrOutput { pcr })),
            Err(err) => {
                error!("fetch pcr test by region {:?}", err);
                return Err(PcrErr::QueryError("pcr by region".into()).into());
            }
        }
    }

    /// Retrieve the pcr made in the whole country
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<PcrInput>
    async fn get_pcr_test_made_country(
        &self,
        request: Request<PcrInput>
    ) -> Result<Response<PcrOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(res) => res,
            None => return Err(PcrErr::InvalidDate.into())
        };

        match query::get_all_by_date_only::<PcrResult>(
            &self.pool,
            "SELECT * FROM pcr_country WHERE jour LIKE $1",
            &date
        ).await {
            Ok(pcr) => Ok(Response::new(PcrOutput { pcr })),
            Err(err) => {
                error!("fetch pcr test in the whole country {:?}", err);
                return Err(PcrErr::QueryError("pcr pcr test in the whole country".into()).into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_grpc_dep_to_return_ok() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PcrServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInput {
            day: Some(1),
            month: 12,
            year: 2021,
            department: Some("75".to_string()),
            region: None
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_by_department(request).await;
        
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_region_to_return_ok() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PcrServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInput {
            day: Some(12),
            month: 12,
            year: 2021,
            region: Some(93),
            department: None
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_by_region(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_country_to_return_ok() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PcrServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInput {
            day: Some(12),
            month: 12,
            year: 2021,
            region: None,
            department: None
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_country(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_error() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PcrServiceHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInput {
            day: Some(2222222),
            month: 12,
            year: 2021,
            department: Some("75".to_string()),
            region: None
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_by_department(request).await;
        
        assert!(res.is_err());
    }
}

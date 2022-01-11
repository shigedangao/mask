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
    pcr_service_department_server::PcrServiceDepartment,
    PcrInputDepartment, PcrOutput, PcrResult
};

pub struct PcrServiceDepHandle {
    pub pool: Arc<PGPool>
}

impl TryFrom<PgRow> for PcrResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let res = Self {
            department: value.try_get("dep").unwrap_or_default(),
            day: value.try_get("jour").unwrap_or_default(),
            population_by_department: value.try_get("pop").unwrap_or_default(),
            total_pcr_test_done: value.try_get("t").unwrap_or_default(),
            total_positive_pcr_test: value.try_get("p").unwrap_or_default(),
            age: value.try_get("cl_age90").unwrap_or_default(),
            // usually used by region
            positive_pcr_test_female: value.try_get("p_f").unwrap_or_default(),
            positive_pcr_test_male: value.try_get("p_h").unwrap_or_default(),
            pcr_test_female: value.try_get("t_f").unwrap_or_default(),
            pcr_test_male: value.try_get("t_h").unwrap_or_default(),
            region: value.try_get("reg").unwrap_or_default(),
            population_by_region: value.try_get("pop").unwrap_or_default()
        };

        Ok(res)
    }
}

impl Date for PcrInputDepartment {
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
impl PcrServiceDepartment for PcrServiceDepHandle {
    /// Get the list of pcr test made per department
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<PcrInputDepartment>
    async fn get_pcr_test_made_by_department(
        &self,
        request: Request<PcrInputDepartment>
    ) -> Result<Response<PcrOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(d) => d,
            None => return Err(PcrErr::InvalidDate.into())
        };

        match query::get_all_by_date_and_gen_field::<PcrResult, &str>(
            &self.pool,
            "SELECT * FROM pcr_test_department WHERE jour LIKE $1 AND dep = $2",
            &date,
            &input.department
        ).await {
            Ok(pcr) => Ok(Response::new(PcrOutput { pcr })),
            Err(err) => {
                error!("fetch pcr by department {:?}", err);
                return Err(PcrErr::QueryError("pcr by department".into()).into());
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
        let service = PcrServiceDepHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInputDepartment {
            day: Some(1),
            month: 12,
            year: 2021,
            department: "75".to_string()
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_by_department(request).await;
        
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_error() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PcrServiceDepHandle {
            pool: Arc::clone(&pool_arc)
        };

        let input = PcrInputDepartment {
            day: Some(2222222),
            month: 12,
            year: 2021,
            department: "75".to_string()
        };

        let request = Request::new(input);
        let res = service.get_pcr_test_made_by_department(request).await;
        
        assert!(res.is_err());
    }
}

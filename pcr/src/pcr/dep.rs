use std::sync::Arc;
use futures::TryStreamExt;
use db::PGPool;
use tonic::{Request, Response, Status, Code};
use utils::Date;

use crate::err::PcrErr;

use super::pcr_test::{
    pcr_service_department_server::PcrServiceDepartment,
    PcrInputDepartment, PcrOutput, PcrResult
};

pub struct PcrServiceDepHandle {
    pub pool: Arc<PGPool>
}

#[derive(sqlx::FromRow, Debug)]
pub struct QueryResult {
    dep: Option<String>,
    jour: Option<String>,
    pop: Option<f64>,
    t: Option<i64>,
    p: Option<i64>,
    cl_age90: Option<i64>
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

impl From<QueryResult> for PcrResult {
    fn from(q: QueryResult) -> Self {
        Self {
            department: q.dep,
            day: q.jour.unwrap_or_default(),
            population_by_department: q.pop,
            total_positive_pcr_test: q.p,
            total_pcr_test_done: q.t,
            age: q.cl_age90.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[tonic::async_trait]
impl PcrServiceDepartment for PcrServiceDepHandle {
    async fn get_pcr_test_made_by_department(
        &self,
        request: Request<PcrInputDepartment>
    ) -> Result<Response<PcrOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date() {
            Some(d) => d,
            None => {
                return Err(Status::new(Code::InvalidArgument, "Date is not valid"))
            }
        };

        let reply = match get_pcr_test_by_department(&self.pool, date, input.department).await {
            Ok(pcr) => PcrOutput { pcr },
            Err(err) => {
                error!("fetch pcr by department {:?}", err);
                return Err(Status::new(Code::Internal, "Unable to retrieve pcr test by department"));
            }
        };

        Ok(Response::new(reply))
    }
}

/// Retrieve pcr test by department
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `date` - String
/// * `department` -String
async fn get_pcr_test_by_department(pool: &PGPool, date: String, department: String) -> Result<Vec<PcrResult>, PcrErr> {
    let mut tests = Vec::new();
    let sql_date_like = format!("{}%", date);

    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM pcr_test_department WHERE jour LIKE $1 AND dep = $2")
        .bind(sql_date_like)
        .bind(department)
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
    async fn expect_to_query_dep_pcr_case() {
        let pool = db::connect("../config.toml").await.unwrap();
        let res = get_pcr_test_by_department(
            &pool,
            "2021-12-20".to_string(),
            "75".to_string()
        ).await;

        assert!(res.is_ok());
    }

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

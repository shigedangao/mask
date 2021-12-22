use std::sync::Arc;
use futures::TryStreamExt;
use tonic::{Request, Response, Status, Code};
use db::PGPool;
use utils::Date;
use crate::err::PcrErr;

use pcr_test::pcr_service_server::PcrService;
use pcr_test::{PcrInput, PcrOutput, PcrResult};

pub mod pcr_test {
    tonic::include_proto!("pcr");
}

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
    pop: Option<f64>,
    pop_f: Option<f64>,
    pop_h: Option<f64>,
    cl_age90: Option<i64>
}

impl From<QueryResult> for PcrResult {
    fn from(q: QueryResult) -> Self {
        Self {
            region: q.reg.unwrap_or_default(),
            day: q.jour.unwrap_or_default(),
            population_by_region: q.pop.unwrap_or_default(),
            positive_pcr_test_male: q.p_h.unwrap_or_default(),
            positive_pcr_test_female: q.p_f.unwrap_or_default(),
            age: q.cl_age90.unwrap_or_default(),
            population_male: q.pop_h.unwrap_or_default(),
            population_female: q.pop_f.unwrap_or_default(),
            total_positive_pcr_test: q.p.unwrap_or_default()
        }
    }
}

impl Date for PcrInput {
    fn get_year(&self) -> i32 {
        self.year
    }

    fn get_month(&self) -> String {
        self.month.clone()
    }

    fn get_day(&self) -> Option<String> {
        self.day.clone()
    }
}

#[tonic::async_trait]
impl PcrService for PcrServiceHandle {
    async fn get_pcr_test_made_by_region(
        &self,
        request: Request<PcrInput>
    ) -> Result<Response<PcrOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date() {
            Some(res) => res,
            None => {
                return Err(Status::new(Code::InvalidArgument, "Date is invalid"))
            }
        };

        let reply = match get_pcr_test_by_region(&self.pool, date, input.region).await {
            Ok(res) => PcrOutput { pcr: res },
            Err(err) => {
                error!("fetch pcr test by region {:?}", err);
                return Err(Status::new(Code::Internal, "Unable to retrieve pcr case by region"));
            }
        };

        Ok(Response::new(reply))
    }
}

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

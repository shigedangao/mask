use std::sync::Arc;
use db::PGPool;
use tonic::{Request, Response, Status, Code};
use futures::TryStreamExt;
use crate::err::PcrErr;

use super::pos_schema::{
    positivity_rate_server::PositivityRate,
    PositivityInput,
    PositivityCollection,
    PositivityDayResult,
};
use utils::Date;

pub struct PosServiceHandle {
    pub pool: Arc<PGPool>
}

#[derive(sqlx::FromRow, Debug)]
struct QueryResult {
    dep: Option<String>,
    jour: Option<String>,
    pop: Option<i64>,
    p: Option<i64>,
    tx_std: Option<f64>
}

impl Date for PositivityInput {
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

impl From<QueryResult> for PositivityDayResult {
    fn from(q: QueryResult) -> Self {
        Self {
            department: q.dep.unwrap_or_default(),
            day: q.jour.unwrap_or_default(),
            population_reference: q.pop.unwrap_or_default(),
            pcr_positive: q.p.unwrap_or_default(),
            infection_rate: q.tx_std.unwrap_or_default()
        }
    }
}

#[tonic::async_trait]
impl PositivityRate for PosServiceHandle {
    async fn get_positivity_by_department_per_day(
        &self,
        request: Request<PositivityInput>
    ) -> Result<Response<PositivityCollection>, Status> {
        let input = request.into_inner();
        let date = match input.build_date() {
            Some(d) => d,
            None => {
                return Err(Status::new(Code::InvalidArgument, "Date is invalid"));
            }
        };

        match get_positivity_per_day(&self.pool, date, input.department).await {
            Ok(rates) => Ok(Response::new(PositivityCollection { rates })),
            Err(err) => {
                error!("fetch positivity cases {:?}", err);
                Err(Status::new(Code::Internal, "Unable to retrieve positivity cases"))
            }
        }
    }
}

async fn get_positivity_per_day(pool: &PGPool, date: String, department: String) -> Result<Vec<PositivityDayResult>, PcrErr> {
    let mut rates = Vec::new();
    let formatted_date = format!("{}%", date);

    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM positivity_rate_per_dep_by_day WHERE jour LIKE $1 AND dep = $2")
        .bind(formatted_date)
        .bind(department)
        .fetch(pool);

    while let Some(row) = stream.try_next().await? {
        let test = PositivityDayResult::from(row);
        rates.push(test) 
    }

    Ok(rates)
}

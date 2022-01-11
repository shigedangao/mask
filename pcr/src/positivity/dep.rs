use std::sync::Arc;
use db::{PGPool, query};
use sqlx::{postgres::PgRow, Row};
use tonic::{Request, Response, Status};
use crate::err::PcrErr;
use super::proto::{
    positivity_rate_server::PositivityRate,
    PositivityInput,
    PositivityCollection,
    PositivityDayResult,
    PositivityWeekCollection
};
use utils::Date;

pub struct PosServiceHandle {
    pub pool: Arc<PGPool>
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

impl TryFrom<PgRow> for PositivityDayResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let res = Self {
            department: value.try_get("dep")?,
            day: value.try_get("jour")?,
            population_reference: value.try_get("pop")?,
            pcr_positive: value.try_get("p")?,
            infection_rate: value.try_get("tx_std")?
        };

        Ok(res)
    }
}

#[tonic::async_trait]
impl PositivityRate for PosServiceHandle {
    /// Retrieve the positivity rate by department and by day
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<PositivityInput>
    async fn get_positivity_by_department_per_day(
        &self,
        request: Request<PositivityInput>
    ) -> Result<Response<PositivityCollection>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(d) => d,
            None => return Err(PcrErr::InvalidDate.into())
        };

        match query::get_all_by_date_and_gen_field::<PositivityDayResult, &str>(
            &self.pool,
            "SELECT * FROM positivity_rate_per_dep_by_day WHERE jour LIKE $1 AND dep = $2",
            &date,
            &input.department
        ).await {
            Ok(rates) => Ok(Response::new(PositivityCollection { rates })),
            Err(err) => {
                error!("fetch positivity cases {:?}", err);
                Err(PcrErr::QueryError("positivity per day".into()).into())
            }
        }
    }

    /// Get Positivity by the department for a week.
    /// Based on a given date. We're calculating the covid case / 100k for the last 7 days
    /// including the given day
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<PositivityInput>
    async fn get_positivity_by_department_per_week(
        &self,
        request: Request<PositivityInput>
    ) -> Result<Response<PositivityWeekCollection>, Status> {
        let input = request.into_inner();
        let dates = match input.get_previous_seven_date_from_day() {
            Some(d) => d,
            None => return Err(PcrErr::InvalidDate.into())
        };

        let res = match get_positivity_for_week(&self.pool, dates, &input.department).await {
            Ok(res) => res,
            Err(err) => {
                error!("fetch positivity cases per week {:?}", err);
                return Err(PcrErr::QueryError("positivity cases per week".into()).into());
            }
        };

        let week_infection_rate = calculate_positivity_per_week(&res);
        Ok(Response::new(PositivityWeekCollection {
            rates: res,
            week_infection_rate
        }))
    }
}

/// SQL query to get the positivity per week
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `dates` - Vec<String>
/// * `department` - &str
async fn get_positivity_for_week(pool: &PGPool, dates: Vec<String>, department: &str) -> Result<Vec<PositivityDayResult>, PcrErr> {
    let mut rates = Vec::new();
    for date in dates.iter() {
        // @Warning
        // We can't query the date between 2 date as they're string...
        // It would be nice to convert the date to a datetime on the import.py script.
        let res: Result<PgRow, sqlx::Error> = sqlx::query("SELECT * FROM positivity_rate_per_dep_by_day WHERE jour = $1 AND dep = $2")
            .bind(date)
            .bind(&department)
            .fetch_one(pool)
            .await;

        match res {
            Ok(data) => rates.push(PositivityDayResult::try_from(data)?),
            Err(err) => {
                match err {
                    sqlx::error::Error::RowNotFound => continue,
                    _ => return Err(err.into())
                }
            }
        }
    }
    
    Ok(rates)
}

/// Calculate the positivity based on the list of result
/// The list can contain from [0..7] item. The size can vary depending
/// if si-dep updated their CSV.
///
/// # Arguments
/// * `cases` - &[PositivityDayResult]
fn calculate_positivity_per_week(cases: &[PositivityDayResult]) -> f64 {
    cases
        .iter()
        .fold(0.0, |acc, c| acc + c.infection_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_grpc_to_return_positivity_per_day_ok() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PosServiceHandle { pool: Arc::clone(&pool_arc) };
        
        let input = PositivityInput {
            day: Some(10),
            month: 12,
            year: 2021,
            department: "94".to_owned()
        };

        let request = Request::new(input);
        let res = service.get_positivity_by_department_per_day(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_positivity_per_day_error() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PosServiceHandle { pool: Arc::clone(&pool_arc) };
        
        let input = PositivityInput {
            day: None,
            month: 30,
            year: 2021,
            department: "94".to_owned()
        };

        let request = Request::new(input);
        let res = service.get_positivity_by_department_per_day(request).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_positivity_per_week_ok() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PosServiceHandle { pool: Arc::clone(&pool_arc) };
        
        let input = PositivityInput {
            day: Some(10),
            month: 12,
            year: 2021,
            department: "94".to_owned()
        };

        let request = Request::new(input);
        let res = service.get_positivity_by_department_per_week(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_positivity_per_week_error() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let service = PosServiceHandle { pool: Arc::clone(&pool_arc) };
        
        let input = PositivityInput {
            day: None,
            month: 12,
            year: 2021,
            department: "80".to_owned()
        };

        let request = Request::new(input);
        let res = service.get_positivity_by_department_per_week(request).await;

        assert!(res.is_err());
    }
}

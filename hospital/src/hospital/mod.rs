use tonic::{
    Request,
    Response,
    Status,
    Code
};
use futures::TryStreamExt;
use care::care_status_server::CareStatus;
use care::{CareStatusPayload, CareStatusInput, CareStatusOutput, CareStatusMonthInput};
use chrono::{NaiveDate, Datelike};
use db::PGPool;

pub mod care {
    tonic::include_proto!("hospital");
}

// Hold a pool of connection
#[derive(Debug)]
pub struct CareService {
    pub pool: PGPool
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
        CareStatusPayload {
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

#[tonic::async_trait]
impl CareStatus for CareService {
    /// Return the number of case in hospital for a date and a region
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<CareStatusInput>
    async fn get_status_by_region(
        &self,
        request: Request<CareStatusInput>
    ) -> Result<Response<CareStatusOutput>, Status> {
        let input = request.into_inner();
        // check date validity
        if let Err(_) = NaiveDate::parse_from_str(&input.date, "%Y-%m-%d") {
            return Err(Status::new(Code::InvalidArgument, "Date is malformated. Please use a date like 2021-12-19"));
        }

        let reply = match get_cases_by_day_and_region(&self.pool, input).await {
            Ok(cases) => CareStatusOutput { cases },
            Err(err) => {
                error!("fetch hospitalization {:?}", err);
                return Err(Status::new(Code::Internal, "Unable to retrieve hospitalization cases by day"));
            }
        };

        Ok(Response::new(reply))
    }

    async fn get_status_by_month_for_region(
        &self,
        request: Request<CareStatusMonthInput>
    ) -> Result<Response<CareStatusOutput>, Status> {
        let input = request.into_inner();
        let date = NaiveDate::parse_from_str(&format!("{}-{}-{}", input.year, input.month, "1"), "%Y-%m-%d")
            .map_err(|_| Status::new(Code::InvalidArgument, "Either invalid month or year"))?;

        let formatted_date = format!("{}-{}%", date.year(), date.month());
        let reply = match get_cases_by_month_by_region(&self.pool, formatted_date, input.region).await {
            Ok(cases) => CareStatusOutput { cases },
            Err(err) => {
                error!("fetch hospitalization {:?}", err);
                return Err(Status::new(Code::Internal, "Unable to retrieve hospitalization cases by month"));
            }
        };

        Ok(Response::new(reply))
    }
}

/// Query the database to return the hospitalization rate for a day and a region
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `input` - CareStatusInput
async fn get_cases_by_day_and_region(pool: &PGPool, input: CareStatusInput) -> Result<Vec<CareStatusPayload>, Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM hospitalization WHERE jour = $1 AND reg = $2")
        .bind(input.date)
        .bind(input.region)
        .fetch(pool);

    while let Some(row) = stream.try_next().await? {
        let case = CareStatusPayload::from(row);
        cases.push(case);
    }

    Ok(cases)
}

/// Query the database to return the hospitalization rate for a month and a region
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `date` - String
/// * `region` - i32
async fn get_cases_by_month_by_region(pool: &PGPool, date: String, region: i32) -> Result<Vec<CareStatusPayload>, Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM hospitalization WHERE jour LIKE $1 AND reg = $2")
        .bind(date)
        .bind(region)
        .fetch(pool);
    
    while let Some(row) = stream.try_next().await? {
        let case = CareStatusPayload::from(row);
        cases.push(case);
    }

    Ok(cases)
}

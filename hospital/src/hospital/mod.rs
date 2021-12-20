use tonic::{
    Request,
    Response,
    Status,
    Code
};
use futures::TryStreamExt;
use care::care_status_server::CareStatus;
use care::{CareStatusPayload, CareStatusInput, CareStatusOutput};
use chrono::NaiveDate;
use db::PGPool;
use super::err::MaskErr;

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
        let (date, has_day) = match input.day {
            Some(day) => (format!("{}-{}-{}", input.year, input.month, day), true),
            None => (format!("{}-{}", input.year, input.month), false)
        };

        if date_valid(&date, has_day).is_err() {
            return Err(Status::new(Code::InvalidArgument, "The date is invalid"));
        }

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

/// Check whenever the date is valid
/// 
/// # Arguments
/// * `date` - &str
/// * `is_day` - bool
fn date_valid(date: &str, is_day: bool) -> Result<(), MaskErr> {
    let updated_date = match is_day {
        true => date.into(),
        false => format!("{}-1", date)
    };

    if let Err(err) = NaiveDate::parse_from_str(&updated_date, "%Y-%m-%d") {
        return Err(MaskErr::from(err));
    }

    Ok(())
} 

/// Query the database to return the hospitalization rate for a day and a region
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

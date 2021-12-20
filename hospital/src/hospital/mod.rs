use tonic::{
    Request,
    Response,
    Status,
    Code
};
use futures::TryStreamExt;
use care::care_status_server::CareStatus;
use care::{CareStatusPayload, CareStatusInput, CareStatusOutput};
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
    dc: Option<i64>
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
            other_care_district: q.autres
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

        let reply = match query_database(&self.pool, input).await {
            Ok(cases) => CareStatusOutput { cases },
            Err(err) => {
                error!("fetch hospitalization {:?}", err);
                return Err(Status::new(Code::Internal, "Unable to retrieve hospitalization cases"));
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
async fn query_database(pool: &PGPool, input: CareStatusInput) -> Result<Vec<CareStatusPayload>, Box<dyn std::error::Error>> {
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

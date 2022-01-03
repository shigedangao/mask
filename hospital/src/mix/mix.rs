use std::sync::Arc;
use db::PGPool;
use tonic::{Request, Response, Status};
use utils::Date;
use futures::TryStreamExt;
use crate::err::MaskErr;
use super::proto_mix::mix_service_server::MixService;
use super::proto_mix::{MixInput, MixOutput, MixPayload};

pub struct MixHandler {
    pub pool: Arc<PGPool>
}

#[derive(sqlx::FromRow, Debug)]
struct QueryResult {
    date: String,
    vac_statut: String,
    nb_pcr: f64,
    nb_pcr_sympt: f64,
    #[sqlx(rename = "nb_pcr+")]
    pcr_positive: f64,
    #[sqlx(rename = "nb_pcr+_sympt")]
    pcr_symptom_positive: f64,
    hc: f64,
    #[sqlx(rename = "hc_pcr+")]
    hospital_entry_pcr_positive: f64,
    sc: f64,
    #[sqlx(rename = "sc_pcr+")]
    icu_entry_pcr_positive: f64,
    dc: f64,
    #[sqlx(rename = "dc_pcr+")]
    pcr_positive_death: f64,
    effectif: i64
}

impl From<QueryResult> for MixPayload {
    fn from(q: QueryResult) -> Self {
        Self {
            date: q.date,
            vaxx_status: q.vac_statut,
            pcr_done: q.nb_pcr,
            pcr_symptom: q.nb_pcr_sympt,
            pcr_positive: q.pcr_positive,
            pcr_symptom_positive: q.pcr_symptom_positive,
            hospital_entry: q.hc,
            hospital_entry_pcr_positive: q.hospital_entry_pcr_positive,
            icu_entry: q.sc,
            icu_entry_pcr_positive: q.icu_entry_pcr_positive,
            death: q.dc,
            pcr_positive_death: q.pcr_positive_death,
            resident_population: q.effectif
        }
    }
}

impl Date for MixInput {
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
impl MixService for MixHandler {
    /// Return the global covid mix data by date. It's a mix of 
    /// VAC-SI, SI-DEP & VAC-SI
    /// 
    /// # Arguments
    /// * `&self` - &Self
    /// * `request` - Request<MixInput>
    async fn get_global_covid_data_by_date(
        &self,
        request: Request<MixInput>
    ) -> Result<Response<MixOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(date) => date,
            None => return Err(MaskErr::InvalidDate.into())
        };

        match get_global_cases_by_date(&self.pool, date).await {
            Ok(data) => Ok(Response::new(MixOutput { data })),
            Err(err) => {
                error!("fetch covid mix data error: {:?}", err);
                return Err(MaskErr::QueryError("fetch covid mix data".to_owned()).into());
            }
        }
    }
}

/// Query the database to get the mix covid cases
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `date` - String
async fn get_global_cases_by_date(pool: &PGPool, date: String) -> Result<Vec<MixPayload>, MaskErr> {
    let mut data = Vec::new();
    
    let mut stream = sqlx::query_as::<_, QueryResult>("SELECT * FROM data_mix WHERE date LIKE $1")
        .bind(date)
        .fetch(pool);

    while let Some(row) = stream.try_next().await? {
        data.push(MixPayload::from(row));
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_to_query_cases() {
        let pool = db::connect("../config.toml").await.unwrap();
        let res = get_global_cases_by_date(
            &pool,
            "2021-10-10".to_string()
        ).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_response() {
        let pool = db::connect("../config.toml").await.unwrap();
        let db_handler = Arc::new(pool);
        let mix_service = MixHandler { pool: Arc::clone(&db_handler) };

        let input = MixInput {
            day: Some(10),
            month: 10,
            year: 2021
        };

        let request = Request::new(input);
        let res = mix_service.get_global_covid_data_by_date(request).await;

        assert!(res.is_ok());
    }
}

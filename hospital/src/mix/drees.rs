use std::sync::Arc;
use db::{PGPool, query};
use sqlx::{
    postgres::PgRow,
    Row
};
use tonic::{Request, Response, Status};
use utils::Date;
use crate::err::MaskErr;
use super::proto_mix::mix_service_server::MixService;
use super::proto_mix::{MixInput, MixOutput, MixResult};

pub struct MixHandler {
    pub pool: Arc<PGPool>
}

impl TryFrom<PgRow> for MixResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let res = Self {
            date: value.try_get("date")?,
            vaxx_status: value.try_get("vac_statut")?,
            pcr_done: value.try_get("nb_pcr")?,
            pcr_symptom: value.try_get("nb_pcr_sympt")?,
            pcr_positive: value.try_get("nb_pcr+")?,
            pcr_symptom_positive: value.try_get("nb_pcr+_sympt")?,
            hospital_entry: value.try_get("hc")?,
            hospital_entry_pcr_positive: value.try_get("hc_pcr+")?,
            icu_entry: value.try_get("sc")?,
            icu_entry_pcr_positive: value.try_get("sc_pcr+")?,
            death: value.try_get("dc")?,
            pcr_positive_death: value.try_get("dc_pcr+")?,
            resident_population: value.try_get("effectif")?
        };

        Ok(res)
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

        match query::get_all_by_date_only::<MixResult>(
            &self.pool,
            "SELECT * FROM data_mix WHERE date LIKE $1",
            &date
        ).await {
            Ok(data) => Ok(Response::new(MixOutput { data })),
            Err(err) => {
                error!("fetch covid mix data error: {:?}", err);
                return Err(MaskErr::QueryError("fetch covid mix data".to_owned()).into());
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

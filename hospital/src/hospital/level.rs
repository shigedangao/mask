use std::sync::Arc;
use db::{PGPool, query};
use sqlx::postgres::PgRow;
use sqlx::Row;
use tonic::{Request, Response, Status};
use utils::Date;
use crate::err::MaskErr;

use super::proto_hospital::{
    level_service_server::LevelService,
    LevelInput,
    LevelOutput, LevelResult, level_result::Sex
};

pub struct LevelHandler {
    pub pool: Arc<PGPool>
}

impl TryFrom<PgRow> for LevelResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let sex = match value.try_get::<i64, &str>("sexe") {
            Ok(s) => match s {
                0 => Sex::Both,
                1 => Sex::Male,
                2 => Sex::Female,
                _ => Sex::Both
            },
            Err(_) => Sex::Male
        };
        
        let res = Self {
            department: value.try_get("dep")?,
            sex: sex.into(),
            date: value.try_get("jour")?,
            hospitalization: value.try_get("hosp")?,
            icu: value.try_get("rea")?,
            conventional_care: value.try_get("hospconv").ok(),
            different_care_services: value.try_get("ssr_usld").ok(),
            other_care_services: value.try_get("autres").ok(),
            back_home: value.try_get("rad")?,
            death: value.try_get("dc")?,
        };

        Ok(res)
    }
}

impl Date for LevelInput {
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
impl LevelService for LevelHandler {
    async fn get_hospital_level_by_department(
        &self,
        request: Request<LevelInput>
    ) -> Result<Response<LevelOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(date) => date,
            None => return Err(MaskErr::InvalidDate.into())
        };

        match query::get_all_by_date_and_gen_field::<LevelResult, String>(
            &self.pool,
            "SELECT * FROM hospital_dep WHERE jour LIKE $1 AND dep = $2",
            &date,
            input.department
        ).await {
            Ok(data) => Ok(Response::new(LevelOutput { data })),
            Err(err) => {
                error!("fetch level in hospital fail {:?}", err);
                return Err(MaskErr::QueryError("level in hospital".into()).into());
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
        let level_handle = LevelHandler {
            pool: Arc::clone(&pool_arc)
        };

        let input = LevelInput {
            day: Some(8),
            month: 1,
            year: 2022,
            department: "77".to_owned()
        };

        let request = Request::new(input);
        let res = level_handle.get_hospital_level_by_department(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_to_get_collection_of_month() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let level_handle = LevelHandler {
            pool: Arc::clone(&pool_arc)
        };

        let input = LevelInput {
            day: None,
            month: 1,
            year: 2022,
            department: "77".to_owned()
        };

        let request = Request::new(input);
        let res = level_handle.get_hospital_level_by_department(request).await;

        assert!(res.is_ok());
    }
}

use std::sync::Arc;
use db::{PGPool, query};
use sqlx::{
    postgres::PgRow,
    Row
};
use tonic::{Request, Response, Status};
use utils::Date;
use crate::err::MaskErr;
use super::proto_icu::icu_service_server::IcuService;
use super::proto_icu::{IcuInput, IcuOutput, IcuResult};

pub struct IcuHandler {
    pub pool: Arc<PGPool>
}

impl TryFrom<PgRow> for IcuResult {
    type Error = sqlx::Error;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        // value is saved a string in the database
        let rate: String = value.try_get("value")?;

        let res = Self {
            day: value.try_get("date")?,
            rate: rate.parse::<f64>().unwrap_or_default()
        };

        Ok(res)
    }
}

impl Date for IcuInput {
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
impl IcuService for IcuHandler {
    /// Get the ICU level in the whole country for unvaxx people. A dataset for region and department exist
    /// but too lazy to implement it right now
    /// 
    /// # Arguments
    /// * `&self`
    /// * `request` - Request<IcuInput>
    async fn get_france_icu_level_for_non_vaxx(
        &self,
        request: Request<IcuInput>
    ) -> Result<Response<IcuOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(date) => date,
            None => return Err(MaskErr::InvalidDate.into())
        };

        match query::get_all_by_date_only(
            &self.pool,
            "SELECT * FROM unvaxx WHERE date LIKE $1",
            &date
        ).await {
            Ok(data) => Ok(Response::new(IcuOutput { data })),
            Err(err) => {
                error!("fetch unvaccinated people error {:?}", err);
                Err(MaskErr::from(err).into())
            }
        }
    }

    /// Get the ICU level in the whole country for vaxx people.
    /// 
    /// # Arguments
    /// * `&self` - Self
    /// * `request` - Request<IcuInput>
    async fn get_france_icu_level_for_vaxx(
        &self,
        request: Request<IcuInput>
    ) -> Result<Response<IcuOutput>, Status> {
        let input = request.into_inner();
        let date = match input.build_date_sql_like() {
            Some(date) => date,
            None => return Err(MaskErr::InvalidDate.into())
        };

        match query::get_all_by_date_only(
            &self.pool,
            "SELECT * FROM vaxx WHERE date LIKE $1",
            &date
        ).await {
            Ok(data) => Ok(Response::new(IcuOutput { data })),
            Err(err) => {
                error!("fetch vaccinated people error {:?}", err);
                Err(MaskErr::from(err).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_grpc_to_return_response_for_unvaxx() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let icu_service = IcuHandler {
            pool: Arc::clone(&pool_arc)
        };

        let input = IcuInput {
            day: Some(18),
            month: 12,
            year: 2021
        };

        let request = Request::new(input);
        let res = icu_service.get_france_icu_level_for_non_vaxx(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn expect_grpc_to_return_response_for_vaxx() {
        let pool = db::connect("../config.toml").await.unwrap();
        let pool_arc = Arc::new(pool);
        let icu_service = IcuHandler {
            pool: Arc::clone(&pool_arc)
        };

        let input = IcuInput {
            day: Some(18),
            month: 12,
            year: 2021
        };

        let request = Request::new(input);
        let res = icu_service.get_france_icu_level_for_vaxx(request).await;

        assert!(res.is_ok());
    }
}

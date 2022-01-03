use std::sync::Arc;
use db::PGPool;
use tonic::{Request, Response, Status};
use utils::Date;
use futures::TryStreamExt;
use crate::err::MaskErr;
use super::proto_icu::icu_service_server::IcuService;
use super::proto_icu::{IcuInput, IcuOutput, IcuResult};

pub struct IcuHandler {
    pub pool: Arc<PGPool>
}

#[derive(sqlx::FromRow, Debug)]
struct QueryResult {
    value: String,
    date: String
}

impl TryFrom<QueryResult> for IcuResult {
    type Error = MaskErr;
    
    fn try_from(q: QueryResult) -> Result<Self, Self::Error> {
        Ok(Self {
            day: q.date,
            rate: q.value.parse::<f64>()?
        })
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

        match get_icu_level_by_date(&self.pool, "unvaxx", date).await {
            Ok(data) => Ok(Response::new(IcuOutput { data })),
            Err(err) => {
                error!("fetch non vaxx error {:?}", err);
                Err(err.into())
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

        match get_icu_level_by_date(&self.pool, "vaxx", date).await {
            Ok(data) => Ok(Response::new(IcuOutput { data })),
            Err(err) => {
                error!("fetch vaxx error {:?}", err);
                Err(err.into())
            }
        }
    }
}

/// Get the level of ICU for vaxx / non vaxx
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `table` - &str
/// * `date` - String
async fn get_icu_level_by_date(pool: &PGPool, table: &str, date: String) -> Result<Vec<IcuResult>, MaskErr> {
    let mut data = Vec::new();
    let query = format!("SELECT * FROM {} WHERE date LIKE $1", table);
    let mut stream = sqlx::query_as::<_, QueryResult>(&query)
        .bind(date)
        .fetch(pool);

    while let Some(row) = stream.try_next().await? {
        data.push(IcuResult::try_from(row)?);
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_to_query_level() {
        let pool = db::connect("../config.toml").await.unwrap();
        let res = get_icu_level_by_date(
            &pool, 
            "unvaxx", 
            "2021-12-10".to_owned()
        ).await;

        assert!(res.is_ok());
    }

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

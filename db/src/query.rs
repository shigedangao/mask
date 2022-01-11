use futures::TryStreamExt;
use sqlx::postgres::PgRow;
use super::err::DBError;

/// Generic helper method which helps to query the database
/// based on a date and a second parameter. The method take as parameter
/// 2 generic field:
///     - T which should implement TryFrom<PgRow>
///     - I which should be a valid type that can be converted by slqx::Postgres
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `query` - &'q str
/// * `date` - &'q str
/// * `other` - I
pub async fn get_all_by_date_and_gen_field<'q, T, I>(
    pool: &super::PGPool,
    query: &'q str,
    date: &'q str,
    other: I
) -> Result<Vec<T>, DBError> 
where
    T: TryFrom<PgRow>,
    I: 'q + sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + std::marker::Send
{
    let mut vec = Vec::new();
    let mut stream = sqlx::query(query)
        .bind(date)
        .bind(other)
        .fetch(pool);

    while let Some(row) = stream.try_next().await.unwrap() {
        let value = T::try_from(row)
            .map_err(|_| DBError::Exec)?;

        vec.push(value);
    }

    Ok(vec)
}

/// Generic helper method which helps to query the get all the data
/// only based on the date. The method takes as parameter
/// 
/// # Arguments
/// * `pool` - &PGPool
/// * `query` - &str
/// * `date` - &str
pub async fn get_all_by_date_only<T>(
    pool: &super::PGPool,
    query: &str,
    date: &str
) -> Result<Vec<T>, DBError>
where
    T: TryFrom<PgRow>
{
    let mut vec = Vec::new();
    let mut stream = sqlx::query(query)
        .bind(date)
        .fetch(pool);

    while let Some(row) = stream.try_next().await.unwrap() {
        let value = T::try_from(row)
            .map_err(|_| DBError::Exec)?;

        vec.push(value);
    }

    Ok(vec)
}

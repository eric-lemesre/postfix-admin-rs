use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::{MySqlPool, PgPool};

use crate::error::DbError;

/// Create a `PostgreSQL` connection pool.
///
/// # Errors
/// Returns `DbError` if the connection fails.
pub async fn create_pg_pool(url: &str, max_connections: u32) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await?;
    Ok(pool)
}

/// Create a `MySQL` connection pool.
///
/// # Errors
/// Returns `DbError` if the connection fails.
pub async fn create_mysql_pool(url: &str, max_connections: u32) -> Result<MySqlPool, DbError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await?;
    Ok(pool)
}

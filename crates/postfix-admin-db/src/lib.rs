//! Data access layer for postfix-admin-rs.
//!
//! Provides repository implementations for `PostgreSQL` and `MySQL`
//! using sqlx with runtime-checked queries.

pub mod error;
pub mod mysql;
pub mod pool;
pub mod postgres;
pub mod rows;
pub mod transaction;

pub use error::DbError;
pub use pool::{create_mysql_pool, create_pg_pool};
pub use transaction::{mysql_transaction, pg_transaction};

pub use postgres::{
    PgAdminRepository, PgAliasDomainRepository, PgAliasRepository, PgAppPasswordRepository,
    PgDkimRepository, PgDomainRepository, PgFetchmailRepository, PgLogRepository,
    PgMailboxRepository, PgVacationRepository,
};

pub use mysql::{
    MysqlAdminRepository, MysqlAliasDomainRepository, MysqlAliasRepository,
    MysqlAppPasswordRepository, MysqlDkimRepository, MysqlDomainRepository,
    MysqlFetchmailRepository, MysqlLogRepository, MysqlMailboxRepository, MysqlVacationRepository,
};

/// Run `PostgreSQL` migrations.
///
/// # Errors
/// Returns `DbError` if migration execution fails.
pub async fn run_pg_migrations(pool: &sqlx::PgPool) -> Result<(), DbError> {
    sqlx::raw_sql(include_str!("migrations/postgres/0001_initial_schema.sql"))
        .execute(pool)
        .await?;
    tracing::info!("PostgreSQL migrations completed successfully");
    Ok(())
}

/// Run `MySQL` migrations.
///
/// # Errors
/// Returns `DbError` if migration execution fails.
pub async fn run_mysql_migrations(pool: &sqlx::MySqlPool) -> Result<(), DbError> {
    let migration = include_str!("migrations/mysql/0001_initial_schema.sql");
    for statement in migration.split(';') {
        let stmt = statement.trim();
        let has_sql = stmt.lines().any(|line| {
            let l = line.trim();
            !l.is_empty() && !l.starts_with("--")
        });
        if !has_sql {
            continue;
        }
        sqlx::query(stmt).execute(pool).await?;
    }
    tracing::info!("MySQL migrations completed successfully");
    Ok(())
}

//! Transaction helpers for executing multiple operations atomically.
//!
//! Provides `pg_transaction` and `mysql_transaction` functions that begin
//! a database transaction, pass a connection to a callback, and commit on
//! success or roll back on error (or panic).
//!
//! # Usage
//!
//! ```ignore
//! use postfix_admin_db::pg_transaction;
//!
//! pg_transaction(&pool, |conn| Box::pin(async move {
//!     sqlx::query("INSERT INTO domain (domain) VALUES ($1)")
//!         .bind("example.com")
//!         .execute(&mut *conn)
//!         .await?;
//!     sqlx::query("INSERT INTO alias (address, goto, domain) VALUES ($1, $2, $3)")
//!         .bind("postmaster@example.com")
//!         .bind("admin@example.com")
//!         .bind("example.com")
//!         .execute(&mut *conn)
//!         .await?;
//!     Ok(())
//! }))
//! .await?;
//! ```

use std::future::Future;
use std::pin::Pin;

use crate::error::DbError;

/// Execute a callback within a `PostgreSQL` transaction.
///
/// Begins a transaction, passes a mutable connection reference to the callback,
/// and commits on success. The transaction is automatically rolled back if the
/// callback returns an error or if a panic occurs.
///
/// # Errors
///
/// Returns `DbError` if the transaction cannot be started, the callback fails,
/// or the commit fails.
pub async fn pg_transaction<F, T>(pool: &sqlx::PgPool, callback: F) -> Result<T, DbError>
where
    F: for<'c> FnOnce(
        &'c mut sqlx::PgConnection,
    ) -> Pin<Box<dyn Future<Output = Result<T, DbError>> + Send + 'c>>,
{
    let mut tx = pool.begin().await?;
    let result = callback(&mut tx).await;
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => Err(err),
    }
}

/// Execute a callback within a `MySQL` transaction.
///
/// Begins a transaction, passes a mutable connection reference to the callback,
/// and commits on success. The transaction is automatically rolled back if the
/// callback returns an error or if a panic occurs.
///
/// # Errors
///
/// Returns `DbError` if the transaction cannot be started, the callback fails,
/// or the commit fails.
pub async fn mysql_transaction<F, T>(pool: &sqlx::MySqlPool, callback: F) -> Result<T, DbError>
where
    F: for<'c> FnOnce(
        &'c mut sqlx::MySqlConnection,
    ) -> Pin<Box<dyn Future<Output = Result<T, DbError>> + Send + 'c>>,
{
    let mut tx = pool.begin().await?;
    let result = callback(&mut tx).await;
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => Err(err),
    }
}

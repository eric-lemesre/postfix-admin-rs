use async_trait::async_trait;
use sqlx::MySqlPool;
use uuid::Uuid;

use postfix_admin_core::dto::{AppPasswordResponse, CreateAppPassword};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::repository::AppPasswordRepository;
use postfix_admin_core::EmailAddress;

use crate::rows::AppPasswordRow;

pub struct MysqlAppPasswordRepository {
    pool: MySqlPool,
}

impl MysqlAppPasswordRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppPasswordRepository for MysqlAppPasswordRepository {
    async fn find_by_username(
        &self,
        username: &EmailAddress,
    ) -> Result<Vec<AppPasswordResponse>, CoreError> {
        let rows = sqlx::query_as::<_, AppPasswordRow>(
            "SELECT id, username, description, last_used, created_at \
             FROM mailbox_app_password WHERE username = ? \
             ORDER BY id ASC",
        )
        .bind(username.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, dto: &CreateAppPassword) -> Result<AppPasswordResponse, CoreError> {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO mailbox_app_password (id, username, description, password_hash) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(dto.username.as_str())
        .bind(&dto.description)
        .bind(&dto.password_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        let row = sqlx::query_as::<_, AppPasswordRow>(
            "SELECT id, username, description, last_used, created_at \
             FROM mailbox_app_password WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM mailbox_app_password WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("app_password", id.to_string()));
        }
        Ok(())
    }

    async fn update_last_used(&self, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query(
            "UPDATE mailbox_app_password SET last_used = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("app_password", id.to_string()));
        }
        Ok(())
    }
}

use async_trait::async_trait;
use sqlx::MySqlPool;

use postfix_admin_core::dto::{AdminResponse, CreateAdmin, UpdateAdmin};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::AdminRepository;
use postfix_admin_core::EmailAddress;

use crate::rows::{AdminRow, CountRow};

const ADMIN_SELECT: &str = "SELECT username, superadmin, \
    (totp_secret IS NOT NULL) AS totp_enabled, \
    active, created_at, updated_at FROM admin";

pub struct MysqlAdminRepository {
    pool: MySqlPool,
}

impl MysqlAdminRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminRepository for MysqlAdminRepository {
    async fn find_by_username(
        &self,
        username: &EmailAddress,
    ) -> Result<Option<AdminResponse>, CoreError> {
        let query = format!("{ADMIN_SELECT} WHERE username = ?");
        let row = sqlx::query_as::<_, AdminRow>(&query)
            .bind(username.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_all(&self, page: &PageRequest) -> Result<PageResponse<AdminResponse>, CoreError> {
        let total = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM admin")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_wrap)]
        let offset = page.offset() as i64;
        let query = format!("{ADMIN_SELECT} ORDER BY username ASC LIMIT ? OFFSET ?");
        let rows = sqlx::query_as::<_, AdminRow>(&query)
            .bind(i64::from(page.per_page()))
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        let items = rows.into_iter().map(Into::into).collect();
        #[allow(clippy::cast_sign_loss)]
        Ok(PageResponse::new(items, total.count as u64, page))
    }

    async fn create(&self, dto: &CreateAdmin) -> Result<AdminResponse, CoreError> {
        sqlx::query(
            "INSERT INTO admin (username, password, superadmin, active) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(dto.username.as_str())
        .bind(dto.password.as_str())
        .bind(dto.superadmin.unwrap_or(false))
        .bind(dto.active.unwrap_or(true))
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_by_username(&dto.username)
            .await?
            .ok_or_else(|| CoreError::not_found("admin", dto.username.as_str()))
    }

    async fn update(
        &self,
        username: &EmailAddress,
        dto: &UpdateAdmin,
    ) -> Result<AdminResponse, CoreError> {
        let result = sqlx::query(
            "UPDATE admin SET \
             password = COALESCE(?, password), \
             superadmin = COALESCE(?, superadmin), \
             active = COALESCE(?, active), \
             updated_at = CURRENT_TIMESTAMP \
             WHERE username = ?",
        )
        .bind(
            dto.password
                .as_ref()
                .map(postfix_admin_core::Password::as_str),
        )
        .bind(dto.superadmin)
        .bind(dto.active)
        .bind(username.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("admin", username.as_str()));
        }

        self.find_by_username(username)
            .await?
            .ok_or_else(|| CoreError::not_found("admin", username.as_str()))
    }

    async fn delete(&self, username: &EmailAddress) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM admin WHERE username = ?")
            .bind(username.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("admin", username.as_str()));
        }
        Ok(())
    }
}

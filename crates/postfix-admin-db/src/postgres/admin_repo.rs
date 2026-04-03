use async_trait::async_trait;
use sqlx::PgPool;

use postfix_admin_core::dto::{AdminResponse, CreateAdmin, UpdateAdmin};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::AdminRepository;
use postfix_admin_core::EmailAddress;

use crate::rows::{AdminRow, CountRow};

pub struct PgAdminRepository {
    pool: PgPool,
}

impl PgAdminRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminRepository for PgAdminRepository {
    async fn find_by_username(
        &self,
        username: &EmailAddress,
    ) -> Result<Option<AdminResponse>, CoreError> {
        let row = sqlx::query_as::<_, AdminRow>(
            "SELECT username, superadmin, \
             (totp_secret IS NOT NULL) AS totp_enabled, \
             active, created_at, updated_at \
             FROM admin WHERE username = $1",
        )
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
        let rows = sqlx::query_as::<_, AdminRow>(
            "SELECT username, superadmin, \
             (totp_secret IS NOT NULL) AS totp_enabled, \
             active, created_at, updated_at \
             FROM admin ORDER BY username ASC LIMIT $1 OFFSET $2",
        )
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
        let row = sqlx::query_as::<_, AdminRow>(
            "INSERT INTO admin (username, password, superadmin, active) \
             VALUES ($1, $2, $3, $4) \
             RETURNING username, superadmin, \
             (totp_secret IS NOT NULL) AS totp_enabled, \
             active, created_at, updated_at",
        )
        .bind(dto.username.as_str())
        .bind(dto.password.as_str())
        .bind(dto.superadmin.unwrap_or(false))
        .bind(dto.active.unwrap_or(true))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn update(
        &self,
        username: &EmailAddress,
        dto: &UpdateAdmin,
    ) -> Result<AdminResponse, CoreError> {
        let row = sqlx::query_as::<_, AdminRow>(
            "UPDATE admin SET \
             password = COALESCE($1, password), \
             superadmin = COALESCE($2, superadmin), \
             active = COALESCE($3, active), \
             updated_at = NOW() \
             WHERE username = $4 \
             RETURNING username, superadmin, \
             (totp_secret IS NOT NULL) AS totp_enabled, \
             active, created_at, updated_at",
        )
        .bind(
            dto.password
                .as_ref()
                .map(postfix_admin_core::Password::as_str),
        )
        .bind(dto.superadmin)
        .bind(dto.active)
        .bind(username.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?
        .ok_or_else(|| CoreError::not_found("admin", username.as_str()))?;

        Ok(row.into())
    }

    async fn delete(&self, username: &EmailAddress) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM admin WHERE username = $1")
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

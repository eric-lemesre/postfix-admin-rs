use async_trait::async_trait;
use sqlx::PgPool;

use postfix_admin_core::dto::{UpdateVacation, VacationResponse};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::repository::VacationRepository;
use postfix_admin_core::EmailAddress;

use crate::rows::VacationRow;

pub struct PgVacationRepository {
    pool: PgPool,
}

impl PgVacationRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VacationRepository for PgVacationRepository {
    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<VacationResponse>, CoreError> {
        let row = sqlx::query_as::<_, VacationRow>(
            "SELECT email, subject, body, domain, active, active_from, active_until, \
             interval_time, created_at, updated_at \
             FROM vacation WHERE email = $1",
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn upsert(
        &self,
        email: &EmailAddress,
        dto: &UpdateVacation,
    ) -> Result<VacationResponse, CoreError> {
        let domain_part = email.domain_part();

        let row = sqlx::query_as::<_, VacationRow>(
            "INSERT INTO vacation (email, subject, body, domain, active, \
             active_from, active_until, interval_time) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (email) DO UPDATE SET \
             subject = COALESCE($2, vacation.subject), \
             body = COALESCE($3, vacation.body), \
             active = COALESCE($5, vacation.active), \
             active_from = COALESCE($6, vacation.active_from), \
             active_until = COALESCE($7, vacation.active_until), \
             interval_time = COALESCE($8, vacation.interval_time), \
             updated_at = NOW() \
             RETURNING email, subject, body, domain, active, active_from, \
             active_until, interval_time, created_at, updated_at",
        )
        .bind(email.as_str())
        .bind(dto.subject.as_deref().unwrap_or(""))
        .bind(dto.body.as_deref().unwrap_or(""))
        .bind(domain_part)
        .bind(dto.active.unwrap_or(true))
        .bind(dto.active_from)
        .bind(dto.active_until)
        .bind(dto.interval_time.unwrap_or(0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn delete(&self, email: &EmailAddress) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM vacation WHERE email = $1")
            .bind(email.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("vacation", email.as_str()));
        }
        Ok(())
    }
}

use async_trait::async_trait;
use sqlx::MySqlPool;

use postfix_admin_core::dto::{UpdateVacation, VacationResponse};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::repository::VacationRepository;
use postfix_admin_core::EmailAddress;

use crate::rows::VacationRow;

const VACATION_COLS: &str = "email, subject, body, domain, active, active_from, \
    active_until, interval_time, created_at, updated_at";

pub struct MysqlVacationRepository {
    pool: MySqlPool,
}

impl MysqlVacationRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VacationRepository for MysqlVacationRepository {
    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<VacationResponse>, CoreError> {
        let query = format!("SELECT {VACATION_COLS} FROM vacation WHERE email = ?");
        let row = sqlx::query_as::<_, VacationRow>(&query)
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

        sqlx::query(
            "INSERT INTO vacation (email, subject, body, domain, active, \
             active_from, active_until, interval_time) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             ON DUPLICATE KEY UPDATE \
             subject = COALESCE(VALUES(subject), subject), \
             body = COALESCE(VALUES(body), body), \
             active = COALESCE(VALUES(active), active), \
             active_from = COALESCE(VALUES(active_from), active_from), \
             active_until = COALESCE(VALUES(active_until), active_until), \
             interval_time = COALESCE(VALUES(interval_time), interval_time), \
             updated_at = CURRENT_TIMESTAMP",
        )
        .bind(email.as_str())
        .bind(dto.subject.as_deref().unwrap_or(""))
        .bind(dto.body.as_deref().unwrap_or(""))
        .bind(domain_part)
        .bind(dto.active.unwrap_or(true))
        .bind(dto.active_from)
        .bind(dto.active_until)
        .bind(dto.interval_time.unwrap_or(0))
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_by_email(email)
            .await?
            .ok_or_else(|| CoreError::not_found("vacation", email.as_str()))
    }

    async fn delete(&self, email: &EmailAddress) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM vacation WHERE email = ?")
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

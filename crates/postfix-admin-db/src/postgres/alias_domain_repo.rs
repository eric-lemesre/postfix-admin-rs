use async_trait::async_trait;
use sqlx::PgPool;

use postfix_admin_core::dto::{AliasDomainResponse, CreateAliasDomain};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::repository::AliasDomainRepository;
use postfix_admin_core::DomainName;

use crate::rows::AliasDomainRow;

pub struct PgAliasDomainRepository {
    pool: PgPool,
}

impl PgAliasDomainRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasDomainRepository for PgAliasDomainRepository {
    async fn find_by_alias(
        &self,
        alias_domain: &DomainName,
    ) -> Result<Option<AliasDomainResponse>, CoreError> {
        let row = sqlx::query_as::<_, AliasDomainRow>(
            "SELECT alias_domain, target_domain, active, created_at, updated_at \
             FROM alias_domain WHERE alias_domain = $1",
        )
        .bind(alias_domain.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_target(
        &self,
        target_domain: &DomainName,
    ) -> Result<Vec<AliasDomainResponse>, CoreError> {
        let rows = sqlx::query_as::<_, AliasDomainRow>(
            "SELECT alias_domain, target_domain, active, created_at, updated_at \
             FROM alias_domain WHERE target_domain = $1 \
             ORDER BY alias_domain ASC",
        )
        .bind(target_domain.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, dto: &CreateAliasDomain) -> Result<AliasDomainResponse, CoreError> {
        let row = sqlx::query_as::<_, AliasDomainRow>(
            "INSERT INTO alias_domain (alias_domain, target_domain, active) \
             VALUES ($1, $2, $3) \
             RETURNING alias_domain, target_domain, active, created_at, updated_at",
        )
        .bind(dto.alias_domain.as_str())
        .bind(dto.target_domain.as_str())
        .bind(dto.active.unwrap_or(true))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn delete(&self, alias_domain: &DomainName) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM alias_domain WHERE alias_domain = $1")
            .bind(alias_domain.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("alias_domain", alias_domain.as_str()));
        }
        Ok(())
    }
}

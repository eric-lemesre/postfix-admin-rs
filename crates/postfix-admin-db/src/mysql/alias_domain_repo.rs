use async_trait::async_trait;
use sqlx::MySqlPool;

use postfix_admin_core::dto::{AliasDomainResponse, CreateAliasDomain};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::repository::AliasDomainRepository;
use postfix_admin_core::DomainName;

use crate::rows::AliasDomainRow;

const AD_COLS: &str = "alias_domain, target_domain, active, created_at, updated_at";

pub struct MysqlAliasDomainRepository {
    pool: MySqlPool,
}

impl MysqlAliasDomainRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasDomainRepository for MysqlAliasDomainRepository {
    async fn find_by_alias(
        &self,
        alias_domain: &DomainName,
    ) -> Result<Option<AliasDomainResponse>, CoreError> {
        let query = format!("SELECT {AD_COLS} FROM alias_domain WHERE alias_domain = ?");
        let row = sqlx::query_as::<_, AliasDomainRow>(&query)
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
        let query = format!(
            "SELECT {AD_COLS} FROM alias_domain WHERE target_domain = ? \
             ORDER BY alias_domain ASC"
        );
        let rows = sqlx::query_as::<_, AliasDomainRow>(&query)
            .bind(target_domain.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, dto: &CreateAliasDomain) -> Result<AliasDomainResponse, CoreError> {
        sqlx::query(
            "INSERT INTO alias_domain (alias_domain, target_domain, active) \
             VALUES (?, ?, ?)",
        )
        .bind(dto.alias_domain.as_str())
        .bind(dto.target_domain.as_str())
        .bind(dto.active.unwrap_or(true))
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_by_alias(&dto.alias_domain)
            .await?
            .ok_or_else(|| CoreError::not_found("alias_domain", dto.alias_domain.as_str()))
    }

    async fn delete(&self, alias_domain: &DomainName) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM alias_domain WHERE alias_domain = ?")
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

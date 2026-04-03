use async_trait::async_trait;
use sqlx::MySqlPool;

use postfix_admin_core::dto::{AliasResponse, CreateAlias, UpdateAlias};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::AliasRepository;
use postfix_admin_core::{DomainName, EmailAddress};

use crate::rows::{AliasRow, CountRow};

const ALIAS_COLS: &str = "address, goto, domain, active, created_at, updated_at";

pub struct MysqlAliasRepository {
    pool: MySqlPool,
}

impl MysqlAliasRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasRepository for MysqlAliasRepository {
    async fn find_by_address(
        &self,
        address: &EmailAddress,
    ) -> Result<Option<AliasResponse>, CoreError> {
        let query = format!("SELECT {ALIAS_COLS} FROM alias WHERE address = ?");
        let row = sqlx::query_as::<_, AliasRow>(&query)
            .bind(address.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_domain(
        &self,
        domain: &DomainName,
        page: &PageRequest,
    ) -> Result<PageResponse<AliasResponse>, CoreError> {
        let total =
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM alias WHERE domain = ?")
                .bind(domain.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_wrap)]
        let offset = page.offset() as i64;
        let query = format!(
            "SELECT {ALIAS_COLS} FROM alias WHERE domain = ? \
             ORDER BY address ASC LIMIT ? OFFSET ?"
        );
        let rows = sqlx::query_as::<_, AliasRow>(&query)
            .bind(domain.as_str())
            .bind(i64::from(page.per_page()))
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        let items = rows.into_iter().map(Into::into).collect();
        #[allow(clippy::cast_sign_loss)]
        Ok(PageResponse::new(items, total.count as u64, page))
    }

    async fn create(&self, dto: &CreateAlias) -> Result<AliasResponse, CoreError> {
        let domain_part = dto.address.domain_part();

        sqlx::query("INSERT INTO alias (address, goto, domain, active) VALUES (?, ?, ?, ?)")
            .bind(dto.address.as_str())
            .bind(&dto.goto)
            .bind(domain_part)
            .bind(dto.active.unwrap_or(true))
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_by_address(&dto.address)
            .await?
            .ok_or_else(|| CoreError::not_found("alias", dto.address.as_str()))
    }

    async fn update(
        &self,
        address: &EmailAddress,
        dto: &UpdateAlias,
    ) -> Result<AliasResponse, CoreError> {
        let result = sqlx::query(
            "UPDATE alias SET \
             goto = COALESCE(?, goto), \
             active = COALESCE(?, active), \
             updated_at = CURRENT_TIMESTAMP \
             WHERE address = ?",
        )
        .bind(dto.goto.as_deref())
        .bind(dto.active)
        .bind(address.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("alias", address.as_str()));
        }

        self.find_by_address(address)
            .await?
            .ok_or_else(|| CoreError::not_found("alias", address.as_str()))
    }

    async fn delete(&self, address: &EmailAddress) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM alias WHERE address = ?")
            .bind(address.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("alias", address.as_str()));
        }
        Ok(())
    }

    async fn count_by_domain(&self, domain: &DomainName) -> Result<i32, CoreError> {
        let row =
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM alias WHERE domain = ?")
                .bind(domain.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_truncation)]
        Ok(row.count as i32)
    }
}

use async_trait::async_trait;
use sqlx::PgPool;

use postfix_admin_core::dto::{AliasResponse, CreateAlias, UpdateAlias};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::AliasRepository;
use postfix_admin_core::{DomainName, EmailAddress};

use crate::rows::{AliasRow, CountRow};

pub struct PgAliasRepository {
    pool: PgPool,
}

impl PgAliasRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasRepository for PgAliasRepository {
    async fn find_by_address(
        &self,
        address: &EmailAddress,
    ) -> Result<Option<AliasResponse>, CoreError> {
        let row = sqlx::query_as::<_, AliasRow>(
            "SELECT address, goto, domain, active, created_at, updated_at \
             FROM alias WHERE address = $1",
        )
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
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM alias WHERE domain = $1")
                .bind(domain.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_wrap)]
        let offset = page.offset() as i64;
        let rows = sqlx::query_as::<_, AliasRow>(
            "SELECT address, goto, domain, active, created_at, updated_at \
             FROM alias WHERE domain = $1 \
             ORDER BY address ASC LIMIT $2 OFFSET $3",
        )
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

        let row = sqlx::query_as::<_, AliasRow>(
            "INSERT INTO alias (address, goto, domain, active) \
             VALUES ($1, $2, $3, $4) \
             RETURNING address, goto, domain, active, created_at, updated_at",
        )
        .bind(dto.address.as_str())
        .bind(&dto.goto)
        .bind(domain_part)
        .bind(dto.active.unwrap_or(true))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn update(
        &self,
        address: &EmailAddress,
        dto: &UpdateAlias,
    ) -> Result<AliasResponse, CoreError> {
        let row = sqlx::query_as::<_, AliasRow>(
            "UPDATE alias SET \
             goto = COALESCE($1, goto), \
             active = COALESCE($2, active), \
             updated_at = NOW() \
             WHERE address = $3 \
             RETURNING address, goto, domain, active, created_at, updated_at",
        )
        .bind(dto.goto.as_deref())
        .bind(dto.active)
        .bind(address.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?
        .ok_or_else(|| CoreError::not_found("alias", address.as_str()))?;

        Ok(row.into())
    }

    async fn delete(&self, address: &EmailAddress) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM alias WHERE address = $1")
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
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM alias WHERE domain = $1")
                .bind(domain.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_truncation)]
        Ok(row.count as i32)
    }
}

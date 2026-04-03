use async_trait::async_trait;
use sqlx::PgPool;

use postfix_admin_core::dto::{CreateDomain, DomainResponse, UpdateDomain};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::DomainRepository;
use postfix_admin_core::DomainName;

use crate::rows::{CountRow, DomainRow};

pub struct PgDomainRepository {
    pool: PgPool,
}

impl PgDomainRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DomainRepository for PgDomainRepository {
    async fn find_by_name(&self, name: &DomainName) -> Result<Option<DomainResponse>, CoreError> {
        let row = sqlx::query_as::<_, DomainRow>(
            "SELECT domain, description, aliases, mailboxes, maxquota, quota, \
             transport, backupmx, password_expiry, active, created_at, updated_at \
             FROM domain WHERE domain = $1",
        )
        .bind(name.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_all(
        &self,
        page: &PageRequest,
    ) -> Result<PageResponse<DomainResponse>, CoreError> {
        let total = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM domain")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_wrap)]
        let offset = page.offset() as i64;
        let rows = sqlx::query_as::<_, DomainRow>(
            "SELECT domain, description, aliases, mailboxes, maxquota, quota, \
             transport, backupmx, password_expiry, active, created_at, updated_at \
             FROM domain ORDER BY domain ASC LIMIT $1 OFFSET $2",
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

    async fn create(&self, dto: &CreateDomain) -> Result<DomainResponse, CoreError> {
        let row = sqlx::query_as::<_, DomainRow>(
            "INSERT INTO domain (domain, description, aliases, mailboxes, maxquota, quota, \
             transport, backupmx, password_expiry, active) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             RETURNING domain, description, aliases, mailboxes, maxquota, quota, \
             transport, backupmx, password_expiry, active, created_at, updated_at",
        )
        .bind(dto.domain.as_str())
        .bind(dto.description.as_deref().unwrap_or(""))
        .bind(dto.aliases.unwrap_or(0))
        .bind(dto.mailboxes.unwrap_or(0))
        .bind(dto.maxquota.unwrap_or(0))
        .bind(dto.quota.unwrap_or(0))
        .bind(dto.transport.as_deref())
        .bind(dto.backupmx.unwrap_or(false))
        .bind(dto.password_expiry.unwrap_or(0))
        .bind(dto.active.unwrap_or(true))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn update(
        &self,
        name: &DomainName,
        dto: &UpdateDomain,
    ) -> Result<DomainResponse, CoreError> {
        let row = sqlx::query_as::<_, DomainRow>(
            "UPDATE domain SET \
             description = COALESCE($1, description), \
             aliases = COALESCE($2, aliases), \
             mailboxes = COALESCE($3, mailboxes), \
             maxquota = COALESCE($4, maxquota), \
             quota = COALESCE($5, quota), \
             transport = COALESCE($6, transport), \
             backupmx = COALESCE($7, backupmx), \
             password_expiry = COALESCE($8, password_expiry), \
             active = COALESCE($9, active), \
             updated_at = NOW() \
             WHERE domain = $10 \
             RETURNING domain, description, aliases, mailboxes, maxquota, quota, \
             transport, backupmx, password_expiry, active, created_at, updated_at",
        )
        .bind(dto.description.as_deref())
        .bind(dto.aliases)
        .bind(dto.mailboxes)
        .bind(dto.maxquota)
        .bind(dto.quota)
        .bind(dto.transport.as_deref())
        .bind(dto.backupmx)
        .bind(dto.password_expiry)
        .bind(dto.active)
        .bind(name.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?
        .ok_or_else(|| CoreError::not_found("domain", name.as_str()))?;

        Ok(row.into())
    }

    async fn delete(&self, name: &DomainName) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM domain WHERE domain = $1")
            .bind(name.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("domain", name.as_str()));
        }
        Ok(())
    }

    async fn count(&self) -> Result<i64, CoreError> {
        let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM domain")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.count)
    }
}

use async_trait::async_trait;
use sqlx::MySqlPool;

use postfix_admin_core::dto::{CreateDomain, DomainResponse, UpdateDomain};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::DomainRepository;
use postfix_admin_core::DomainName;

use crate::rows::{CountRow, DomainRow};

const DOMAIN_COLS: &str = "domain, description, aliases, mailboxes, maxquota, quota, \
    transport, backupmx, password_expiry, active, created_at, updated_at";

pub struct MysqlDomainRepository {
    pool: MySqlPool,
}

impl MysqlDomainRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DomainRepository for MysqlDomainRepository {
    async fn find_by_name(&self, name: &DomainName) -> Result<Option<DomainResponse>, CoreError> {
        let query = format!("SELECT {DOMAIN_COLS} FROM domain WHERE domain = ?");
        let row = sqlx::query_as::<_, DomainRow>(&query)
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
        let query =
            format!("SELECT {DOMAIN_COLS} FROM domain ORDER BY domain ASC LIMIT ? OFFSET ?");
        let rows = sqlx::query_as::<_, DomainRow>(&query)
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
        sqlx::query(
            "INSERT INTO domain (domain, description, aliases, mailboxes, maxquota, quota, \
             transport, backupmx, password_expiry, active) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_by_name(&dto.domain)
            .await?
            .ok_or_else(|| CoreError::not_found("domain", dto.domain.as_str()))
    }

    async fn update(
        &self,
        name: &DomainName,
        dto: &UpdateDomain,
    ) -> Result<DomainResponse, CoreError> {
        let result = sqlx::query(
            "UPDATE domain SET \
             description = COALESCE(?, description), \
             aliases = COALESCE(?, aliases), \
             mailboxes = COALESCE(?, mailboxes), \
             maxquota = COALESCE(?, maxquota), \
             quota = COALESCE(?, quota), \
             transport = COALESCE(?, transport), \
             backupmx = COALESCE(?, backupmx), \
             password_expiry = COALESCE(?, password_expiry), \
             active = COALESCE(?, active), \
             updated_at = CURRENT_TIMESTAMP \
             WHERE domain = ?",
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
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("domain", name.as_str()));
        }

        self.find_by_name(name)
            .await?
            .ok_or_else(|| CoreError::not_found("domain", name.as_str()))
    }

    async fn delete(&self, name: &DomainName) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM domain WHERE domain = ?")
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

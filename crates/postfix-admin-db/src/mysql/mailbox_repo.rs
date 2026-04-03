use async_trait::async_trait;
use sqlx::MySqlPool;

use postfix_admin_core::dto::{CreateMailbox, MailboxResponse, UpdateMailbox};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::MailboxRepository;
use postfix_admin_core::validation::generate_maildir;
use postfix_admin_core::{DomainName, EmailAddress};

use crate::rows::{CountRow, MailboxRow};

const MAILBOX_SELECT: &str = "SELECT username, name, maildir, quota, local_part, domain, \
    password_expiry, (totp_secret IS NOT NULL) AS totp_enabled, \
    active, created_at, updated_at FROM mailbox";

pub struct MysqlMailboxRepository {
    pool: MySqlPool,
}

impl MysqlMailboxRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MailboxRepository for MysqlMailboxRepository {
    async fn find_by_username(
        &self,
        username: &EmailAddress,
    ) -> Result<Option<MailboxResponse>, CoreError> {
        let query = format!("{MAILBOX_SELECT} WHERE username = ?");
        let row = sqlx::query_as::<_, MailboxRow>(&query)
            .bind(username.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_domain(
        &self,
        domain: &DomainName,
        page: &PageRequest,
    ) -> Result<PageResponse<MailboxResponse>, CoreError> {
        let total =
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM mailbox WHERE domain = ?")
                .bind(domain.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_wrap)]
        let offset = page.offset() as i64;
        let query =
            format!("{MAILBOX_SELECT} WHERE domain = ? ORDER BY username ASC LIMIT ? OFFSET ?");
        let rows = sqlx::query_as::<_, MailboxRow>(&query)
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

    async fn create(&self, dto: &CreateMailbox) -> Result<MailboxResponse, CoreError> {
        let local_part = dto.username.local_part();
        let domain_part = dto.username.domain_part();
        let maildir = generate_maildir(domain_part, local_part);

        sqlx::query(
            "INSERT INTO mailbox (username, password, name, maildir, quota, \
             local_part, domain, active) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(dto.username.as_str())
        .bind(dto.password.as_str())
        .bind(dto.name.as_deref().unwrap_or(""))
        .bind(&maildir)
        .bind(dto.quota.unwrap_or(0))
        .bind(local_part)
        .bind(domain_part)
        .bind(dto.active.unwrap_or(true))
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_by_username(&dto.username)
            .await?
            .ok_or_else(|| CoreError::not_found("mailbox", dto.username.as_str()))
    }

    async fn update(
        &self,
        username: &EmailAddress,
        dto: &UpdateMailbox,
    ) -> Result<MailboxResponse, CoreError> {
        let result = sqlx::query(
            "UPDATE mailbox SET \
             password = COALESCE(?, password), \
             name = COALESCE(?, name), \
             quota = COALESCE(?, quota), \
             active = COALESCE(?, active), \
             updated_at = CURRENT_TIMESTAMP \
             WHERE username = ?",
        )
        .bind(
            dto.password
                .as_ref()
                .map(postfix_admin_core::Password::as_str),
        )
        .bind(dto.name.as_deref())
        .bind(dto.quota)
        .bind(dto.active)
        .bind(username.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("mailbox", username.as_str()));
        }

        self.find_by_username(username)
            .await?
            .ok_or_else(|| CoreError::not_found("mailbox", username.as_str()))
    }

    async fn delete(&self, username: &EmailAddress) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM mailbox WHERE username = ?")
            .bind(username.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("mailbox", username.as_str()));
        }
        Ok(())
    }

    async fn count_by_domain(&self, domain: &DomainName) -> Result<i32, CoreError> {
        let row =
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM mailbox WHERE domain = ?")
                .bind(domain.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_truncation)]
        Ok(row.count as i32)
    }
}

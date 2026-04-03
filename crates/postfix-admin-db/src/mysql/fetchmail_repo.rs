use async_trait::async_trait;
use sqlx::MySqlPool;
use uuid::Uuid;

use postfix_admin_core::dto::{CreateFetchmail, FetchmailResponse, UpdateFetchmail};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::FetchmailRepository;
use postfix_admin_core::EmailAddress;

use crate::rows::{CountRow, FetchmailRow};

const FETCHMAIL_SELECT_COLS: &str = "id, domain, mailbox, src_server, src_auth, src_user, \
    src_folder, poll_time, fetchall, keep_mail AS keep, protocol, usessl, sslcertck, \
    extra_options, mda, returned_text, active, date, created_at, updated_at";

pub struct MysqlFetchmailRepository {
    pool: MySqlPool,
}

impl MysqlFetchmailRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FetchmailRepository for MysqlFetchmailRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FetchmailResponse>, CoreError> {
        let query = format!("SELECT {FETCHMAIL_SELECT_COLS} FROM fetchmail WHERE id = ?");
        let row = sqlx::query_as::<_, FetchmailRow>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_mailbox(
        &self,
        mailbox: &EmailAddress,
        page: &PageRequest,
    ) -> Result<PageResponse<FetchmailResponse>, CoreError> {
        let total = sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM fetchmail WHERE mailbox = ?",
        )
        .bind(mailbox.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        #[allow(clippy::cast_possible_wrap)]
        let offset = page.offset() as i64;
        let query = format!(
            "SELECT {FETCHMAIL_SELECT_COLS} FROM fetchmail \
             WHERE mailbox = ? ORDER BY id ASC LIMIT ? OFFSET ?"
        );
        let rows = sqlx::query_as::<_, FetchmailRow>(&query)
            .bind(mailbox.as_str())
            .bind(i64::from(page.per_page()))
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        let items = rows.into_iter().map(Into::into).collect();
        #[allow(clippy::cast_sign_loss)]
        Ok(PageResponse::new(items, total.count as u64, page))
    }

    async fn create(&self, dto: &CreateFetchmail) -> Result<FetchmailResponse, CoreError> {
        let domain_part = dto.mailbox.domain_part();
        let id = Uuid::now_v7();

        sqlx::query(
            "INSERT INTO fetchmail (id, domain, mailbox, src_server, src_auth, src_user, \
             src_password, src_folder, poll_time, fetchall, keep_mail, protocol, usessl, \
             sslcertck, extra_options, mda, active) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(domain_part)
        .bind(dto.mailbox.as_str())
        .bind(&dto.src_server)
        .bind(dto.src_auth.as_deref().unwrap_or("password"))
        .bind(&dto.src_user)
        .bind(&dto.src_password)
        .bind(dto.src_folder.as_deref().unwrap_or(""))
        .bind(dto.poll_time.unwrap_or(10))
        .bind(dto.fetchall.unwrap_or(false))
        .bind(dto.keep.unwrap_or(true))
        .bind(dto.protocol.as_deref().unwrap_or("IMAP"))
        .bind(dto.usessl.unwrap_or(true))
        .bind(dto.sslcertck.unwrap_or(true))
        .bind(dto.extra_options.as_deref())
        .bind(dto.mda.as_deref().unwrap_or(""))
        .bind(dto.active.unwrap_or(true))
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| CoreError::not_found("fetchmail", id.to_string()))
    }

    async fn update(
        &self,
        id: Uuid,
        dto: &UpdateFetchmail,
    ) -> Result<FetchmailResponse, CoreError> {
        let result = sqlx::query(
            "UPDATE fetchmail SET \
             src_server = COALESCE(?, src_server), \
             src_auth = COALESCE(?, src_auth), \
             src_user = COALESCE(?, src_user), \
             src_password = COALESCE(?, src_password), \
             src_folder = COALESCE(?, src_folder), \
             poll_time = COALESCE(?, poll_time), \
             fetchall = COALESCE(?, fetchall), \
             keep_mail = COALESCE(?, keep_mail), \
             protocol = COALESCE(?, protocol), \
             usessl = COALESCE(?, usessl), \
             sslcertck = COALESCE(?, sslcertck), \
             extra_options = COALESCE(?, extra_options), \
             mda = COALESCE(?, mda), \
             active = COALESCE(?, active), \
             updated_at = CURRENT_TIMESTAMP \
             WHERE id = ?",
        )
        .bind(dto.src_server.as_deref())
        .bind(dto.src_auth.as_deref())
        .bind(dto.src_user.as_deref())
        .bind(dto.src_password.as_deref())
        .bind(dto.src_folder.as_deref())
        .bind(dto.poll_time)
        .bind(dto.fetchall)
        .bind(dto.keep)
        .bind(dto.protocol.as_deref())
        .bind(dto.usessl)
        .bind(dto.sslcertck)
        .bind(dto.extra_options.as_deref())
        .bind(dto.mda.as_deref())
        .bind(dto.active)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("fetchmail", id.to_string()));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| CoreError::not_found("fetchmail", id.to_string()))
    }

    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM fetchmail WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("fetchmail", id.to_string()));
        }
        Ok(())
    }
}

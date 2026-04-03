use async_trait::async_trait;
use sqlx::MySqlPool;
use uuid::Uuid;

use postfix_admin_core::dto::{
    CreateDkimKey, CreateDkimSigning, DkimKeyResponse, DkimSigningResponse,
};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::repository::DkimRepository;
use postfix_admin_core::DomainName;

use crate::rows::{DkimKeyRow, DkimSigningRow};

const DKIM_KEY_COLS: &str = "id, domain_name, description, selector, public_key, \
    created_at, updated_at";
const DKIM_SIGNING_COLS: &str = "id, author, dkim_id, created_at, updated_at";

pub struct MysqlDkimRepository {
    pool: MySqlPool,
}

impl MysqlDkimRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DkimRepository for MysqlDkimRepository {
    async fn find_key_by_id(&self, id: Uuid) -> Result<Option<DkimKeyResponse>, CoreError> {
        let query = format!("SELECT {DKIM_KEY_COLS} FROM dkim_key WHERE id = ?");
        let row = sqlx::query_as::<_, DkimKeyRow>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_keys_by_domain(
        &self,
        domain: &DomainName,
    ) -> Result<Vec<DkimKeyResponse>, CoreError> {
        let query =
            format!("SELECT {DKIM_KEY_COLS} FROM dkim_key WHERE domain_name = ? ORDER BY id ASC");
        let rows = sqlx::query_as::<_, DkimKeyRow>(&query)
            .bind(domain.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create_key(&self, dto: &CreateDkimKey) -> Result<DkimKeyResponse, CoreError> {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO dkim_key (id, domain_name, description, selector, private_key, \
             public_key) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(dto.domain_name.as_str())
        .bind(dto.description.as_deref().unwrap_or(""))
        .bind(dto.selector.as_deref().unwrap_or("default"))
        .bind(&dto.private_key)
        .bind(&dto.public_key)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        self.find_key_by_id(id)
            .await?
            .ok_or_else(|| CoreError::not_found("dkim_key", id.to_string()))
    }

    async fn delete_key(&self, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM dkim_key WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("dkim_key", id.to_string()));
        }
        Ok(())
    }

    async fn find_signings_by_key_id(
        &self,
        dkim_id: Uuid,
    ) -> Result<Vec<DkimSigningResponse>, CoreError> {
        let query = format!(
            "SELECT {DKIM_SIGNING_COLS} FROM dkim_signing WHERE dkim_id = ? ORDER BY id ASC"
        );
        let rows = sqlx::query_as::<_, DkimSigningRow>(&query)
            .bind(dkim_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create_signing(
        &self,
        dto: &CreateDkimSigning,
    ) -> Result<DkimSigningResponse, CoreError> {
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO dkim_signing (id, author, dkim_id) VALUES (?, ?, ?)")
            .bind(id)
            .bind(&dto.author)
            .bind(dto.dkim_id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        let query = format!("SELECT {DKIM_SIGNING_COLS} FROM dkim_signing WHERE id = ?");
        let row = sqlx::query_as::<_, DkimSigningRow>(&query)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn delete_signing(&self, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM dkim_signing WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::not_found("dkim_signing", id.to_string()));
        }
        Ok(())
    }
}

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::{CreateFetchmail, FetchmailResponse, UpdateFetchmail};
use crate::error::CoreError;
use crate::pagination::{PageRequest, PageResponse};
use crate::types::EmailAddress;

#[async_trait]
pub trait FetchmailRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FetchmailResponse>, CoreError>;
    async fn find_by_mailbox(
        &self,
        mailbox: &EmailAddress,
        page: &PageRequest,
    ) -> Result<PageResponse<FetchmailResponse>, CoreError>;
    async fn create(&self, dto: &CreateFetchmail) -> Result<FetchmailResponse, CoreError>;
    async fn update(&self, id: Uuid, dto: &UpdateFetchmail)
        -> Result<FetchmailResponse, CoreError>;
    async fn delete(&self, id: Uuid) -> Result<(), CoreError>;
}

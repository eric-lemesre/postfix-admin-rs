use async_trait::async_trait;

use crate::dto::{CreateFetchmail, FetchmailResponse, UpdateFetchmail};
use crate::error::CoreError;
use crate::pagination::{PageRequest, PageResponse};
use crate::types::EmailAddress;

#[async_trait]
pub trait FetchmailRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<FetchmailResponse>, CoreError>;
    async fn find_by_mailbox(
        &self,
        mailbox: &EmailAddress,
        page: &PageRequest,
    ) -> Result<PageResponse<FetchmailResponse>, CoreError>;
    async fn create(&self, dto: &CreateFetchmail) -> Result<FetchmailResponse, CoreError>;
    async fn update(&self, id: i32, dto: &UpdateFetchmail) -> Result<FetchmailResponse, CoreError>;
    async fn delete(&self, id: i32) -> Result<(), CoreError>;
}

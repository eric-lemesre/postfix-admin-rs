use async_trait::async_trait;

use crate::dto::{CreateMailbox, MailboxResponse, UpdateMailbox};
use crate::error::CoreError;
use crate::pagination::{PageRequest, PageResponse};
use crate::types::{DomainName, EmailAddress};

#[async_trait]
pub trait MailboxRepository: Send + Sync {
    async fn find_by_username(
        &self,
        username: &EmailAddress,
    ) -> Result<Option<MailboxResponse>, CoreError>;
    async fn find_by_domain(
        &self,
        domain: &DomainName,
        page: &PageRequest,
    ) -> Result<PageResponse<MailboxResponse>, CoreError>;
    async fn create(&self, dto: &CreateMailbox) -> Result<MailboxResponse, CoreError>;
    async fn update(
        &self,
        username: &EmailAddress,
        dto: &UpdateMailbox,
    ) -> Result<MailboxResponse, CoreError>;
    async fn delete(&self, username: &EmailAddress) -> Result<(), CoreError>;
    async fn count_by_domain(&self, domain: &DomainName) -> Result<i32, CoreError>;
}

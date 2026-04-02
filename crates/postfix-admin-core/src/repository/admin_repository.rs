use async_trait::async_trait;

use crate::dto::{AdminResponse, CreateAdmin, UpdateAdmin};
use crate::error::CoreError;
use crate::pagination::{PageRequest, PageResponse};
use crate::types::EmailAddress;

#[async_trait]
pub trait AdminRepository: Send + Sync {
    async fn find_by_username(
        &self,
        username: &EmailAddress,
    ) -> Result<Option<AdminResponse>, CoreError>;
    async fn find_all(&self, page: &PageRequest) -> Result<PageResponse<AdminResponse>, CoreError>;
    async fn create(&self, dto: &CreateAdmin) -> Result<AdminResponse, CoreError>;
    async fn update(
        &self,
        username: &EmailAddress,
        dto: &UpdateAdmin,
    ) -> Result<AdminResponse, CoreError>;
    async fn delete(&self, username: &EmailAddress) -> Result<(), CoreError>;
}

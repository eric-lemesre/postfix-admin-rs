use async_trait::async_trait;

use crate::dto::{CreateDomain, DomainResponse, UpdateDomain};
use crate::error::CoreError;
use crate::pagination::{PageRequest, PageResponse};
use crate::types::DomainName;

#[async_trait]
pub trait DomainRepository: Send + Sync {
    async fn find_by_name(&self, name: &DomainName) -> Result<Option<DomainResponse>, CoreError>;
    async fn find_all(&self, page: &PageRequest)
        -> Result<PageResponse<DomainResponse>, CoreError>;
    async fn create(&self, dto: &CreateDomain) -> Result<DomainResponse, CoreError>;
    async fn update(
        &self,
        name: &DomainName,
        dto: &UpdateDomain,
    ) -> Result<DomainResponse, CoreError>;
    async fn delete(&self, name: &DomainName) -> Result<(), CoreError>;
    async fn count(&self) -> Result<i64, CoreError>;
}

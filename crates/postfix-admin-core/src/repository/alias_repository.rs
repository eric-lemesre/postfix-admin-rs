use async_trait::async_trait;

use crate::dto::{AliasResponse, CreateAlias, UpdateAlias};
use crate::error::CoreError;
use crate::pagination::{PageRequest, PageResponse};
use crate::types::{DomainName, EmailAddress};

#[async_trait]
pub trait AliasRepository: Send + Sync {
    async fn find_by_address(
        &self,
        address: &EmailAddress,
    ) -> Result<Option<AliasResponse>, CoreError>;
    async fn find_by_domain(
        &self,
        domain: &DomainName,
        page: &PageRequest,
    ) -> Result<PageResponse<AliasResponse>, CoreError>;
    async fn create(&self, dto: &CreateAlias) -> Result<AliasResponse, CoreError>;
    async fn update(
        &self,
        address: &EmailAddress,
        dto: &UpdateAlias,
    ) -> Result<AliasResponse, CoreError>;
    async fn delete(&self, address: &EmailAddress) -> Result<(), CoreError>;
    async fn count_by_domain(&self, domain: &DomainName) -> Result<i32, CoreError>;
}

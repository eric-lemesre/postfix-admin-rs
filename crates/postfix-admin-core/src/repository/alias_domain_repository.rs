use async_trait::async_trait;

use crate::dto::{AliasDomainResponse, CreateAliasDomain};
use crate::error::CoreError;
use crate::types::DomainName;

#[async_trait]
pub trait AliasDomainRepository: Send + Sync {
    async fn find_by_alias(
        &self,
        alias_domain: &DomainName,
    ) -> Result<Option<AliasDomainResponse>, CoreError>;
    async fn find_by_target(
        &self,
        target_domain: &DomainName,
    ) -> Result<Vec<AliasDomainResponse>, CoreError>;
    async fn create(&self, dto: &CreateAliasDomain) -> Result<AliasDomainResponse, CoreError>;
    async fn delete(&self, alias_domain: &DomainName) -> Result<(), CoreError>;
}

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::{CreateDkimKey, CreateDkimSigning, DkimKeyResponse, DkimSigningResponse};
use crate::error::CoreError;
use crate::types::DomainName;

#[async_trait]
pub trait DkimRepository: Send + Sync {
    async fn find_key_by_id(&self, id: Uuid) -> Result<Option<DkimKeyResponse>, CoreError>;
    async fn find_keys_by_domain(
        &self,
        domain: &DomainName,
    ) -> Result<Vec<DkimKeyResponse>, CoreError>;
    async fn create_key(&self, dto: &CreateDkimKey) -> Result<DkimKeyResponse, CoreError>;
    async fn delete_key(&self, id: Uuid) -> Result<(), CoreError>;

    async fn find_signings_by_key_id(
        &self,
        dkim_id: Uuid,
    ) -> Result<Vec<DkimSigningResponse>, CoreError>;
    async fn create_signing(
        &self,
        dto: &CreateDkimSigning,
    ) -> Result<DkimSigningResponse, CoreError>;
    async fn delete_signing(&self, id: Uuid) -> Result<(), CoreError>;
}

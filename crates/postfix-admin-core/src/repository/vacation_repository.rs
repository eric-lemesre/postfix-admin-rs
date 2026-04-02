use async_trait::async_trait;

use crate::dto::{UpdateVacation, VacationResponse};
use crate::error::CoreError;
use crate::types::EmailAddress;

#[async_trait]
pub trait VacationRepository: Send + Sync {
    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<VacationResponse>, CoreError>;
    async fn upsert(
        &self,
        email: &EmailAddress,
        dto: &UpdateVacation,
    ) -> Result<VacationResponse, CoreError>;
    async fn delete(&self, email: &EmailAddress) -> Result<(), CoreError>;
}

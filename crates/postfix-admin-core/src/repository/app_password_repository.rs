use async_trait::async_trait;

use crate::dto::{AppPasswordResponse, CreateAppPassword};
use crate::error::CoreError;
use crate::types::EmailAddress;

#[async_trait]
pub trait AppPasswordRepository: Send + Sync {
    async fn find_by_username(
        &self,
        username: &EmailAddress,
    ) -> Result<Vec<AppPasswordResponse>, CoreError>;
    async fn create(&self, dto: &CreateAppPassword) -> Result<AppPasswordResponse, CoreError>;
    async fn delete(&self, id: i32) -> Result<(), CoreError>;
    async fn update_last_used(&self, id: i32) -> Result<(), CoreError>;
}

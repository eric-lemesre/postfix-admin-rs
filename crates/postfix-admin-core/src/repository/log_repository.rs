use async_trait::async_trait;

use crate::dto::{CreateLog, LogFilter, LogResponse};
use crate::error::CoreError;
use crate::pagination::{PageRequest, PageResponse};

#[async_trait]
pub trait LogRepository: Send + Sync {
    async fn create(&self, dto: &CreateLog) -> Result<LogResponse, CoreError>;
    async fn find_all(
        &self,
        filter: &LogFilter,
        page: &PageRequest,
    ) -> Result<PageResponse<LogResponse>, CoreError>;
}

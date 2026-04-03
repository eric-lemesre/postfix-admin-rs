use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::types::IpAddress;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateTotpException {
    pub ip: IpAddress,
    pub username: Option<String>,
    #[validate(length(max = 255))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TotpExceptionResponse {
    pub id: Uuid,
    pub ip: IpAddress,
    pub username: Option<String>,
    pub description: Option<String>,
}

impl From<crate::models::TotpExceptionAddress> for TotpExceptionResponse {
    fn from(te: crate::models::TotpExceptionAddress) -> Self {
        Self {
            id: te.id,
            ip: te.ip,
            username: te.username,
            description: te.description,
        }
    }
}

use serde::Serialize;

use crate::types::EmailAddress;

#[derive(Debug, Clone, Serialize)]
pub struct QuotaResponse {
    pub username: EmailAddress,
    pub path: String,
    pub current: i64,
}

impl From<crate::models::Quota> for QuotaResponse {
    fn from(q: crate::models::Quota) -> Self {
        Self {
            username: q.username,
            path: q.path,
            current: q.current,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Quota2Response {
    pub username: EmailAddress,
    pub bytes: i64,
    pub messages: i32,
}

impl From<crate::models::Quota2> for Quota2Response {
    fn from(q: crate::models::Quota2) -> Self {
        Self {
            username: q.username,
            bytes: q.bytes,
            messages: q.messages,
        }
    }
}

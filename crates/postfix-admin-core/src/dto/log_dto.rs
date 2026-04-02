use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateLog {
    #[validate(length(min = 1, max = 255))]
    pub username: String,
    #[validate(length(min = 1, max = 255))]
    pub domain: String,
    #[validate(length(min = 1, max = 255))]
    pub action: String,
    pub data: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogResponse {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub username: String,
    pub domain: String,
    pub action: String,
    pub data: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl From<crate::models::Log> for LogResponse {
    fn from(l: crate::models::Log) -> Self {
        Self {
            id: l.id,
            timestamp: l.timestamp,
            username: l.username,
            domain: l.domain,
            action: l.action,
            data: l.data,
            ip_address: l.ip_address,
            user_agent: l.user_agent,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogFilter {
    pub domain: Option<String>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

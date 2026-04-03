use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub username: String,
    pub domain: String,
    pub action: String,
    pub data: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

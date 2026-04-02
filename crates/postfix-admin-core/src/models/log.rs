use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub username: String,
    pub domain: String,
    pub action: String,
    pub data: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

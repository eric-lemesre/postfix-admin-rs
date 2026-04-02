use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub domain: DomainName,
    pub description: String,
    pub aliases: i32,
    pub mailboxes: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: Option<String>,
    pub backupmx: bool,
    pub password_expiry: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

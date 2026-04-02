use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    pub username: EmailAddress,
    pub password: String,
    pub name: String,
    pub maildir: String,
    pub quota: i64,
    pub local_part: String,
    pub domain: DomainName,
    pub password_expiry: Option<DateTime<Utc>>,
    pub totp_enabled: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

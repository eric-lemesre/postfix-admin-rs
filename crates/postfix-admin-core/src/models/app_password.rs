use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::EmailAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxAppPassword {
    pub id: i32,
    pub username: EmailAddress,
    pub description: String,
    pub password_hash: String,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

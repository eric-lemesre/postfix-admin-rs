use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::types::EmailAddress;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateAppPassword {
    pub username: EmailAddress,
    #[validate(length(min = 1, max = 255))]
    pub description: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppPasswordResponse {
    pub id: Uuid,
    pub username: EmailAddress,
    pub description: String,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<crate::models::MailboxAppPassword> for AppPasswordResponse {
    fn from(ap: crate::models::MailboxAppPassword) -> Self {
        Self {
            id: ap.id,
            username: ap.username,
            description: ap.description,
            last_used: ap.last_used,
            created_at: ap.created_at,
        }
    }
}

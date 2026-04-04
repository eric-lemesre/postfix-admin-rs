use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::types::{DomainName, EmailAddress, Password};

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateMailbox {
    pub username: EmailAddress,
    pub password: Password,
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(range(min = 0))]
    pub quota: Option<i64>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateMailbox {
    pub password: Option<Password>,
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(range(min = 0))]
    pub quota: Option<i64>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MailboxResponse {
    pub username: EmailAddress,
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

impl From<crate::models::Mailbox> for MailboxResponse {
    fn from(m: crate::models::Mailbox) -> Self {
        Self {
            username: m.username,
            name: m.name,
            maildir: m.maildir,
            quota: m.quota,
            local_part: m.local_part,
            domain: m.domain,
            password_expiry: m.password_expiry,
            totp_enabled: m.totp_enabled,
            active: m.active,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

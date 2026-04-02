use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin {
    pub username: EmailAddress,
    pub password: String,
    pub superadmin: bool,
    pub totp_enabled: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAdmin {
    pub username: EmailAddress,
    pub domain: DomainName,
    pub created_at: DateTime<Utc>,
}

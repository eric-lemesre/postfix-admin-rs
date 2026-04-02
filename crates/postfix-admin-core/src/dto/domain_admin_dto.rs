use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDomainAdmin {
    pub username: EmailAddress,
    pub domain: DomainName,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainAdminResponse {
    pub username: EmailAddress,
    pub domain: DomainName,
    pub created_at: DateTime<Utc>,
}

impl From<crate::models::DomainAdmin> for DomainAdminResponse {
    fn from(da: crate::models::DomainAdmin) -> Self {
        Self {
            username: da.username,
            domain: da.domain,
            created_at: da.created_at,
        }
    }
}

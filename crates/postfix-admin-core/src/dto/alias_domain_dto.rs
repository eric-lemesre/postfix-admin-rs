use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::DomainName;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAliasDomain {
    pub alias_domain: DomainName,
    pub target_domain: DomainName,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AliasDomainResponse {
    pub alias_domain: DomainName,
    pub target_domain: DomainName,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::AliasDomain> for AliasDomainResponse {
    fn from(ad: crate::models::AliasDomain) -> Self {
        Self {
            alias_domain: ad.alias_domain,
            target_domain: ad.target_domain,
            active: ad.active,
            created_at: ad.created_at,
            updated_at: ad.updated_at,
        }
    }
}

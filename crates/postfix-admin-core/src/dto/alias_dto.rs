use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateAlias {
    pub address: EmailAddress,
    #[validate(length(min = 1))]
    pub goto: String,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateAlias {
    #[validate(length(min = 1))]
    pub goto: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AliasResponse {
    pub address: EmailAddress,
    pub goto: String,
    pub domain: DomainName,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::Alias> for AliasResponse {
    fn from(a: crate::models::Alias) -> Self {
        Self {
            address: a.address,
            goto: a.goto,
            domain: a.domain,
            active: a.active,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

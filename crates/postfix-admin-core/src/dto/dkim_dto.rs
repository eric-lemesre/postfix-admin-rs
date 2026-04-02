use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::DomainName;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateDkimKey {
    pub domain_name: DomainName,
    #[validate(length(max = 255))]
    pub description: Option<String>,
    #[validate(length(min = 1, max = 63))]
    pub selector: Option<String>,
    pub private_key: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DkimKeyResponse {
    pub id: i32,
    pub domain_name: DomainName,
    pub description: String,
    pub selector: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::DkimKey> for DkimKeyResponse {
    fn from(dk: crate::models::DkimKey) -> Self {
        Self {
            id: dk.id,
            domain_name: dk.domain_name,
            description: dk.description,
            selector: dk.selector,
            public_key: dk.public_key,
            created_at: dk.created_at,
            updated_at: dk.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateDkimSigning {
    #[validate(length(min = 1, max = 255))]
    pub author: String,
    pub dkim_id: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DkimSigningResponse {
    pub id: i32,
    pub author: String,
    pub dkim_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::DkimSigning> for DkimSigningResponse {
    fn from(ds: crate::models::DkimSigning) -> Self {
        Self {
            id: ds.id,
            author: ds.author,
            dkim_id: ds.dkim_id,
            created_at: ds.created_at,
            updated_at: ds.updated_at,
        }
    }
}

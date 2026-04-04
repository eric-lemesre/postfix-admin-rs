use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::types::DomainName;

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateDomain {
    pub domain: DomainName,
    #[validate(length(max = 255))]
    pub description: Option<String>,
    #[validate(range(min = 0))]
    pub aliases: Option<i32>,
    #[validate(range(min = 0))]
    pub mailboxes: Option<i32>,
    #[validate(range(min = 0))]
    pub maxquota: Option<i64>,
    #[validate(range(min = 0))]
    pub quota: Option<i64>,
    pub transport: Option<String>,
    pub backupmx: Option<bool>,
    #[validate(range(min = 0))]
    pub password_expiry: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateDomain {
    #[validate(length(max = 255))]
    pub description: Option<String>,
    #[validate(range(min = 0))]
    pub aliases: Option<i32>,
    #[validate(range(min = 0))]
    pub mailboxes: Option<i32>,
    #[validate(range(min = 0))]
    pub maxquota: Option<i64>,
    #[validate(range(min = 0))]
    pub quota: Option<i64>,
    pub transport: Option<String>,
    pub backupmx: Option<bool>,
    #[validate(range(min = 0))]
    pub password_expiry: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DomainResponse {
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

impl From<crate::models::Domain> for DomainResponse {
    fn from(d: crate::models::Domain) -> Self {
        Self {
            domain: d.domain,
            description: d.description,
            aliases: d.aliases,
            mailboxes: d.mailboxes,
            maxquota: d.maxquota,
            quota: d.quota,
            transport: d.transport,
            backupmx: d.backupmx,
            password_expiry: d.password_expiry,
            active: d.active,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

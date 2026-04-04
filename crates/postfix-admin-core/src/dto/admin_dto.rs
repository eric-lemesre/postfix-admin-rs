use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::types::{EmailAddress, Password};

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateAdmin {
    pub username: EmailAddress,
    pub password: Password,
    pub superadmin: Option<bool>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateAdmin {
    pub password: Option<Password>,
    pub superadmin: Option<bool>,
    pub totp_enabled: Option<bool>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminResponse {
    pub username: EmailAddress,
    pub superadmin: bool,
    pub totp_enabled: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::Admin> for AdminResponse {
    fn from(a: crate::models::Admin) -> Self {
        Self {
            username: a.username,
            superadmin: a.superadmin,
            totp_enabled: a.totp_enabled,
            active: a.active,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

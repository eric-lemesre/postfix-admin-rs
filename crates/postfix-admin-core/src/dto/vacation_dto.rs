use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateVacation {
    #[validate(length(max = 255))]
    pub subject: Option<String>,
    pub body: Option<String>,
    pub active: Option<bool>,
    pub active_from: Option<DateTime<Utc>>,
    pub active_until: Option<DateTime<Utc>>,
    #[validate(range(min = 0))]
    pub interval_time: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VacationResponse {
    pub email: EmailAddress,
    pub subject: String,
    pub body: String,
    pub domain: DomainName,
    pub active: bool,
    pub active_from: Option<DateTime<Utc>>,
    pub active_until: Option<DateTime<Utc>>,
    pub interval_time: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::Vacation> for VacationResponse {
    fn from(v: crate::models::Vacation) -> Self {
        Self {
            email: v.email,
            subject: v.subject,
            body: v.body,
            domain: v.domain,
            active: v.active,
            active_from: v.active_from,
            active_until: v.active_until,
            interval_time: v.interval_time,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}

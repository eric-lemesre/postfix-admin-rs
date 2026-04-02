use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vacation {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VacationNotification {
    pub on_vacation: EmailAddress,
    pub notified: String,
    pub notified_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    pub address: EmailAddress,
    pub goto: String,
    pub domain: DomainName,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

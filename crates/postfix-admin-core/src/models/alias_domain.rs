use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasDomain {
    pub alias_domain: DomainName,
    pub target_domain: DomainName,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

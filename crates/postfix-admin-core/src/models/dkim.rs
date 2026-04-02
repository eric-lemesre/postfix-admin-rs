use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkimKey {
    pub id: i32,
    pub domain_name: DomainName,
    pub description: String,
    pub selector: String,
    pub private_key: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkimSigning {
    pub id: i32,
    pub author: String,
    pub dkim_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

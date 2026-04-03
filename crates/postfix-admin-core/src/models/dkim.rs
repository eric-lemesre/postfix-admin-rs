use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkimKey {
    pub id: Uuid,
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
    pub id: Uuid,
    pub author: String,
    pub dkim_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

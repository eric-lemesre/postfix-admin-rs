use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::IpAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpExceptionAddress {
    pub id: Uuid,
    pub ip: IpAddress,
    pub username: Option<String>,
    pub description: Option<String>,
}

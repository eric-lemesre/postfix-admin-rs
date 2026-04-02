use serde::{Deserialize, Serialize};

use crate::types::IpAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpExceptionAddress {
    pub id: i32,
    pub ip: IpAddress,
    pub username: Option<String>,
    pub description: Option<String>,
}

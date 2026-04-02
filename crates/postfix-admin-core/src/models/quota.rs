use serde::{Deserialize, Serialize};

use crate::types::EmailAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    pub username: EmailAddress,
    pub path: String,
    pub current: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota2 {
    pub username: EmailAddress,
    pub bytes: i64,
    pub messages: i32,
}

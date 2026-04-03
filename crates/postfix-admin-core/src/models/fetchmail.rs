use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Fetchmail {
    pub id: Uuid,
    pub domain: DomainName,
    pub mailbox: EmailAddress,
    pub src_server: String,
    pub src_auth: String,
    pub src_user: String,
    pub src_password: String,
    pub src_folder: String,
    pub poll_time: i32,
    pub fetchall: bool,
    pub keep: bool,
    pub protocol: String,
    pub usessl: bool,
    pub sslcertck: bool,
    pub extra_options: Option<String>,
    pub mda: String,
    pub returned_text: Option<String>,
    pub active: bool,
    pub date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

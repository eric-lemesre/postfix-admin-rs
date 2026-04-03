use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::types::{DomainName, EmailAddress};

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateFetchmail {
    pub mailbox: EmailAddress,
    #[validate(length(min = 1, max = 255))]
    pub src_server: String,
    pub src_auth: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub src_user: String,
    #[validate(length(min = 1, max = 255))]
    pub src_password: String,
    pub src_folder: Option<String>,
    #[validate(range(min = 1))]
    pub poll_time: Option<i32>,
    pub fetchall: Option<bool>,
    pub keep: Option<bool>,
    pub protocol: Option<String>,
    pub usessl: Option<bool>,
    pub sslcertck: Option<bool>,
    pub extra_options: Option<String>,
    pub mda: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateFetchmail {
    #[validate(length(min = 1, max = 255))]
    pub src_server: Option<String>,
    pub src_auth: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub src_user: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub src_password: Option<String>,
    pub src_folder: Option<String>,
    #[validate(range(min = 1))]
    pub poll_time: Option<i32>,
    pub fetchall: Option<bool>,
    pub keep: Option<bool>,
    pub protocol: Option<String>,
    pub usessl: Option<bool>,
    pub sslcertck: Option<bool>,
    pub extra_options: Option<String>,
    pub mda: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct FetchmailResponse {
    pub id: Uuid,
    pub domain: DomainName,
    pub mailbox: EmailAddress,
    pub src_server: String,
    pub src_auth: String,
    pub src_user: String,
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

impl From<crate::models::Fetchmail> for FetchmailResponse {
    fn from(f: crate::models::Fetchmail) -> Self {
        Self {
            id: f.id,
            domain: f.domain,
            mailbox: f.mailbox,
            src_server: f.src_server,
            src_auth: f.src_auth,
            src_user: f.src_user,
            src_folder: f.src_folder,
            poll_time: f.poll_time,
            fetchall: f.fetchall,
            keep: f.keep,
            protocol: f.protocol,
            usessl: f.usessl,
            sslcertck: f.sslcertck,
            extra_options: f.extra_options,
            mda: f.mda,
            returned_text: f.returned_text,
            active: f.active,
            date: f.date,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

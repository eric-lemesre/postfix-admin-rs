use chrono::{DateTime, Utc};
use uuid::Uuid;

use postfix_admin_core::dto::{
    AdminResponse, AliasDomainResponse, AliasResponse, AppPasswordResponse, DkimKeyResponse,
    DkimSigningResponse, DomainResponse, FetchmailResponse, LogResponse, MailboxResponse,
    VacationResponse,
};
use postfix_admin_core::{DomainName, EmailAddress};

#[derive(Debug, sqlx::FromRow)]
pub struct DomainRow {
    pub domain: String,
    pub description: String,
    pub aliases: i32,
    pub mailboxes: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: Option<String>,
    pub backupmx: bool,
    pub password_expiry: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DomainRow> for DomainResponse {
    fn from(r: DomainRow) -> Self {
        Self {
            domain: DomainName::from_trusted(r.domain),
            description: r.description,
            aliases: r.aliases,
            mailboxes: r.mailboxes,
            maxquota: r.maxquota,
            quota: r.quota,
            transport: r.transport,
            backupmx: r.backupmx,
            password_expiry: r.password_expiry,
            active: r.active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AdminRow {
    pub username: String,
    pub superadmin: bool,
    pub totp_enabled: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AdminRow> for AdminResponse {
    fn from(r: AdminRow) -> Self {
        Self {
            username: EmailAddress::from_trusted(r.username),
            superadmin: r.superadmin,
            totp_enabled: r.totp_enabled,
            active: r.active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct MailboxRow {
    pub username: String,
    pub name: String,
    pub maildir: String,
    pub quota: i64,
    pub local_part: String,
    pub domain: String,
    pub password_expiry: Option<DateTime<Utc>>,
    pub totp_enabled: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MailboxRow> for MailboxResponse {
    fn from(r: MailboxRow) -> Self {
        Self {
            username: EmailAddress::from_trusted(r.username),
            name: r.name,
            maildir: r.maildir,
            quota: r.quota,
            local_part: r.local_part,
            domain: DomainName::from_trusted(r.domain),
            password_expiry: r.password_expiry,
            totp_enabled: r.totp_enabled,
            active: r.active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AliasRow {
    pub address: String,
    pub goto: String,
    pub domain: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AliasRow> for AliasResponse {
    fn from(r: AliasRow) -> Self {
        Self {
            address: EmailAddress::from_trusted(r.address),
            goto: r.goto,
            domain: DomainName::from_trusted(r.domain),
            active: r.active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AliasDomainRow {
    pub alias_domain: String,
    pub target_domain: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AliasDomainRow> for AliasDomainResponse {
    fn from(r: AliasDomainRow) -> Self {
        Self {
            alias_domain: DomainName::from_trusted(r.alias_domain),
            target_domain: DomainName::from_trusted(r.target_domain),
            active: r.active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct VacationRow {
    pub email: String,
    pub subject: String,
    pub body: String,
    pub domain: String,
    pub active: bool,
    pub active_from: Option<DateTime<Utc>>,
    pub active_until: Option<DateTime<Utc>>,
    pub interval_time: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<VacationRow> for VacationResponse {
    fn from(r: VacationRow) -> Self {
        Self {
            email: EmailAddress::from_trusted(r.email),
            subject: r.subject,
            body: r.body,
            domain: DomainName::from_trusted(r.domain),
            active: r.active,
            active_from: r.active_from,
            active_until: r.active_until,
            interval_time: r.interval_time,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DkimKeyRow {
    pub id: Uuid,
    pub domain_name: String,
    pub description: String,
    pub selector: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DkimKeyRow> for DkimKeyResponse {
    fn from(r: DkimKeyRow) -> Self {
        Self {
            id: r.id,
            domain_name: DomainName::from_trusted(r.domain_name),
            description: r.description,
            selector: r.selector,
            public_key: r.public_key,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DkimSigningRow {
    pub id: Uuid,
    pub author: String,
    pub dkim_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DkimSigningRow> for DkimSigningResponse {
    fn from(r: DkimSigningRow) -> Self {
        Self {
            id: r.id,
            author: r.author,
            dkim_id: r.dkim_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
#[allow(clippy::struct_excessive_bools)]
pub struct FetchmailRow {
    pub id: Uuid,
    pub domain: String,
    pub mailbox: String,
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

impl From<FetchmailRow> for FetchmailResponse {
    fn from(r: FetchmailRow) -> Self {
        Self {
            id: r.id,
            domain: DomainName::from_trusted(r.domain),
            mailbox: EmailAddress::from_trusted(r.mailbox),
            src_server: r.src_server,
            src_auth: r.src_auth,
            src_user: r.src_user,
            src_folder: r.src_folder,
            poll_time: r.poll_time,
            fetchall: r.fetchall,
            keep: r.keep,
            protocol: r.protocol,
            usessl: r.usessl,
            sslcertck: r.sslcertck,
            extra_options: r.extra_options,
            mda: r.mda,
            returned_text: r.returned_text,
            active: r.active,
            date: r.date,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct LogRow {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub username: String,
    pub domain: String,
    pub action: String,
    pub data: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl From<LogRow> for LogResponse {
    fn from(r: LogRow) -> Self {
        Self {
            id: r.id,
            timestamp: r.timestamp,
            username: r.username,
            domain: r.domain,
            action: r.action,
            data: r.data,
            ip_address: r.ip_address,
            user_agent: r.user_agent,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AppPasswordRow {
    pub id: Uuid,
    pub username: String,
    pub description: String,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<AppPasswordRow> for AppPasswordResponse {
    fn from(r: AppPasswordRow) -> Self {
        Self {
            id: r.id,
            username: EmailAddress::from_trusted(r.username),
            description: r.description,
            last_used: r.last_used,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CountRow {
    pub count: i64,
}

/// Row for fetching only the password hash.
#[derive(Debug, sqlx::FromRow)]
pub struct PasswordRow {
    pub password: String,
}

/// Row for fetching domain names from `domain_admins`.
#[derive(Debug, sqlx::FromRow)]
pub struct DomainNameRow {
    pub domain: String,
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn domain_row_converts_to_response() {
        let row = DomainRow {
            domain: "example.com".to_string(),
            description: "Test domain".to_string(),
            aliases: 10,
            mailboxes: 5,
            maxquota: 1024,
            quota: 2048,
            transport: None,
            backupmx: false,
            password_expiry: 0,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp: DomainResponse = row.into();
        assert_eq!(resp.domain.as_str(), "example.com");
        assert_eq!(resp.aliases, 10);
    }

    #[test]
    fn admin_row_converts_to_response() {
        let row = AdminRow {
            username: "admin@example.com".to_string(),
            superadmin: true,
            totp_enabled: false,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp: AdminResponse = row.into();
        assert_eq!(resp.username.as_str(), "admin@example.com");
        assert!(resp.superadmin);
    }

    #[test]
    fn mailbox_row_converts_to_response() {
        let row = MailboxRow {
            username: "user@example.com".to_string(),
            name: "User".to_string(),
            maildir: "example.com/user/".to_string(),
            quota: 1024,
            local_part: "user".to_string(),
            domain: "example.com".to_string(),
            password_expiry: None,
            totp_enabled: false,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp: MailboxResponse = row.into();
        assert_eq!(resp.username.as_str(), "user@example.com");
        assert_eq!(resp.domain.as_str(), "example.com");
    }

    #[test]
    fn log_row_converts_to_response() {
        let row = LogRow {
            id: Uuid::nil(),
            timestamp: Utc::now(),
            username: "admin@example.com".to_string(),
            domain: "example.com".to_string(),
            action: "create_domain".to_string(),
            data: "{}".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
        };
        let resp: LogResponse = row.into();
        assert_eq!(resp.id, Uuid::nil());
        assert_eq!(resp.action, "create_domain");
    }

    #[test]
    fn app_password_row_converts_to_response() {
        let row = AppPasswordRow {
            id: Uuid::nil(),
            username: "user@example.com".to_string(),
            description: "Thunderbird".to_string(),
            last_used: None,
            created_at: Utc::now(),
        };
        let resp: AppPasswordResponse = row.into();
        assert_eq!(resp.id, Uuid::nil());
        assert_eq!(resp.username.as_str(), "user@example.com");
    }
}

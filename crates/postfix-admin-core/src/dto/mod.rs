mod admin_dto;
mod alias_domain_dto;
mod alias_dto;
mod app_password_dto;
mod dkim_dto;
mod domain_admin_dto;
mod domain_dto;
mod fetchmail_dto;
mod log_dto;
mod mailbox_dto;
mod quota_dto;
mod totp_exception_dto;
mod vacation_dto;

pub use admin_dto::{AdminResponse, CreateAdmin, UpdateAdmin};
pub use alias_domain_dto::{AliasDomainResponse, CreateAliasDomain};
pub use alias_dto::{AliasResponse, CreateAlias, UpdateAlias};
pub use app_password_dto::{AppPasswordResponse, CreateAppPassword};
pub use dkim_dto::{CreateDkimKey, CreateDkimSigning, DkimKeyResponse, DkimSigningResponse};
pub use domain_admin_dto::{CreateDomainAdmin, DomainAdminResponse};
pub use domain_dto::{CreateDomain, DomainResponse, UpdateDomain};
pub use fetchmail_dto::{CreateFetchmail, FetchmailResponse, UpdateFetchmail};
pub use log_dto::{CreateLog, LogFilter, LogResponse};
pub use mailbox_dto::{CreateMailbox, MailboxResponse, UpdateMailbox};
pub use quota_dto::{Quota2Response, QuotaResponse};
pub use totp_exception_dto::{CreateTotpException, TotpExceptionResponse};
pub use vacation_dto::{UpdateVacation, VacationResponse};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DomainName, EmailAddress};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn domain_response_serialization_excludes_nothing_sensitive() {
        let resp = DomainResponse {
            domain: DomainName::from_trusted("example.com"),
            description: String::from("Test"),
            aliases: 10,
            mailboxes: 10,
            maxquota: 1024,
            quota: 2048,
            transport: None,
            backupmx: false,
            password_expiry: 0,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| unreachable!());
        assert!(json.contains("example.com"));
    }

    #[test]
    fn mailbox_response_excludes_password() {
        let resp = MailboxResponse {
            username: EmailAddress::from_trusted("user@example.com"),
            name: String::from("User"),
            maildir: String::from("example.com/user/"),
            quota: 0,
            local_part: String::from("user"),
            domain: DomainName::from_trusted("example.com"),
            password_expiry: None,
            totp_enabled: false,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| unreachable!());
        // MailboxResponse has no "password" field (only "password_expiry" which is unrelated)
        assert!(!json.contains(r#""password":"#));
    }

    #[test]
    fn admin_response_excludes_password() {
        let resp = AdminResponse {
            username: EmailAddress::from_trusted("admin@example.com"),
            superadmin: true,
            totp_enabled: false,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| unreachable!());
        assert!(!json.contains("password"));
    }

    #[test]
    fn fetchmail_response_excludes_src_password() {
        let resp = FetchmailResponse {
            id: Uuid::nil(),
            domain: DomainName::from_trusted("example.com"),
            mailbox: EmailAddress::from_trusted("user@example.com"),
            src_server: String::from("imap.remote.com"),
            src_auth: String::from("password"),
            src_user: String::from("remote_user"),
            src_folder: String::new(),
            poll_time: 10,
            fetchall: false,
            keep: false,
            protocol: String::from("IMAP"),
            usessl: true,
            sslcertck: true,
            extra_options: None,
            mda: String::new(),
            returned_text: None,
            active: true,
            date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| unreachable!());
        assert!(!json.contains("src_password"));
    }

    #[test]
    fn dkim_key_response_excludes_private_key() {
        let resp = DkimKeyResponse {
            id: Uuid::nil(),
            domain_name: DomainName::from_trusted("example.com"),
            description: String::new(),
            selector: String::from("default"),
            public_key: String::from("pub"),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| unreachable!());
        assert!(!json.contains("private_key"));
    }

    #[test]
    fn create_domain_can_be_deserialized() {
        let json = r#"{"domain":"test.org"}"#;
        let result: Result<CreateDomain, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn create_mailbox_password_required() {
        let json = r#"{"username":"user@test.org"}"#;
        let result: Result<CreateMailbox, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn create_domain_validates_negative_aliases() {
        use validator::Validate;
        let json = r#"{"domain":"test.org","aliases":-1}"#;
        let dto: CreateDomain = serde_json::from_str(json).unwrap_or_else(|_| unreachable!());
        assert!(dto.validate().is_err());
    }

    #[test]
    fn log_filter_can_be_deserialized() {
        let json = r#"{"domain":"example.com"}"#;
        let result: Result<LogFilter, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn quota2_response_serializable() {
        let resp = Quota2Response {
            username: EmailAddress::from_trusted("user@example.com"),
            bytes: 1_000_000,
            messages: 42,
        };
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| unreachable!());
        assert!(json.contains("1000000"));
    }
}

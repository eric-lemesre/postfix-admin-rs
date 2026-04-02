mod admin;
mod alias;
mod alias_domain;
mod app_password;
mod dkim;
mod domain;
mod fetchmail;
mod log;
mod mailbox;
mod quota;
mod totp_exception;
mod vacation;

pub use admin::{Admin, DomainAdmin};
pub use alias::Alias;
pub use alias_domain::AliasDomain;
pub use app_password::MailboxAppPassword;
pub use dkim::{DkimKey, DkimSigning};
pub use domain::Domain;
pub use fetchmail::Fetchmail;
pub use log::Log;
pub use mailbox::Mailbox;
pub use quota::{Quota, Quota2};
pub use totp_exception::TotpExceptionAddress;
pub use vacation::{Vacation, VacationNotification};

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::types::{DomainName, EmailAddress};

    use super::*;

    #[test]
    fn domain_can_be_constructed() {
        let domain = Domain {
            domain: DomainName::from_trusted("example.com"),
            description: String::new(),
            aliases: 0,
            mailboxes: 0,
            maxquota: 0,
            quota: 0,
            transport: None,
            backupmx: false,
            password_expiry: 0,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(domain.active);
    }

    #[test]
    fn mailbox_can_be_constructed() {
        let mbox = Mailbox {
            username: EmailAddress::from_trusted("user@example.com"),
            password: String::from("hashed"),
            name: String::from("Test User"),
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
        assert_eq!(mbox.local_part, "user");
    }

    #[test]
    fn alias_can_be_constructed() {
        let alias = Alias {
            address: EmailAddress::from_trusted("info@example.com"),
            goto: String::from("user@example.com"),
            domain: DomainName::from_trusted("example.com"),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(alias.active);
    }

    #[test]
    fn admin_can_be_constructed() {
        let admin = Admin {
            username: EmailAddress::from_trusted("admin@example.com"),
            password: String::from("hashed"),
            superadmin: true,
            totp_enabled: false,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(admin.superadmin);
    }

    #[test]
    fn log_uses_plain_strings() {
        let log = Log {
            id: 1,
            timestamp: Utc::now(),
            username: String::from("admin@example.com"),
            domain: String::from("example.com"),
            action: String::from("create_mailbox"),
            data: String::from("user@example.com"),
            ip_address: Some(String::from("192.168.1.1")),
            user_agent: None,
        };
        assert_eq!(log.action, "create_mailbox");
    }
}

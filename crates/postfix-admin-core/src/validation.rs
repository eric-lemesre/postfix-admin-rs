use std::sync::LazyLock;

use regex::Regex;

use crate::error::{DomainError, ValidationError};
use crate::models::Domain;

static MAILDIR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._/-]+/$").unwrap_or_else(|_| unreachable!()));

/// # Errors
/// Returns `DomainError::QuotaExceeded` if the mailbox quota exceeds the domain limit.
pub fn validate_quota_within_domain(
    domain: &Domain,
    mailbox_quota: i64,
) -> Result<(), DomainError> {
    if domain.maxquota > 0 && mailbox_quota > domain.maxquota * 1024 * 1024 {
        return Err(DomainError::QuotaExceeded {
            domain: domain.domain.to_string(),
            reason: format!(
                "mailbox quota ({mailbox_quota} bytes) exceeds domain maxquota ({} MB)",
                domain.maxquota
            ),
        });
    }
    Ok(())
}

/// # Errors
/// Returns `DomainError::LimitReached` if the mailbox count has reached the domain limit.
pub fn validate_mailbox_count(domain: &Domain, current_count: i32) -> Result<(), DomainError> {
    if domain.mailboxes > 0 && current_count >= domain.mailboxes {
        return Err(DomainError::LimitReached {
            domain: domain.domain.to_string(),
            reason: format!(
                "mailbox limit reached ({current_count}/{})",
                domain.mailboxes
            ),
        });
    }
    Ok(())
}

/// # Errors
/// Returns `DomainError::LimitReached` if the alias count has reached the domain limit.
pub fn validate_alias_count(domain: &Domain, current_count: i32) -> Result<(), DomainError> {
    if domain.aliases > 0 && current_count >= domain.aliases {
        return Err(DomainError::LimitReached {
            domain: domain.domain.to_string(),
            reason: format!("alias limit reached ({current_count}/{})", domain.aliases),
        });
    }
    Ok(())
}

/// # Errors
/// Returns `DomainError::Inactive` if the domain is not active.
pub fn validate_domain_active(domain: &Domain) -> Result<(), DomainError> {
    if !domain.active {
        return Err(DomainError::Inactive {
            domain: domain.domain.to_string(),
        });
    }
    Ok(())
}

/// # Errors
/// Returns `ValidationError` if the destination list is empty or contains invalid addresses.
pub fn validate_alias_destinations(goto: &str) -> Result<Vec<String>, ValidationError> {
    let destinations: Vec<String> = goto
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if destinations.is_empty() {
        return Err(ValidationError::invalid_field(
            "goto",
            "must contain at least one destination",
        ));
    }

    for dest in &destinations {
        if !dest.contains('@') {
            return Err(ValidationError::invalid_field(
                "goto",
                format!("invalid destination address: {dest}"),
            ));
        }
    }

    Ok(destinations)
}

/// # Errors
/// Returns `ValidationError` if the maildir format is invalid or contains path traversal.
pub fn validate_maildir_format(maildir: &str) -> Result<(), ValidationError> {
    if maildir.is_empty() {
        return Err(ValidationError::invalid_field(
            "maildir",
            "must not be empty",
        ));
    }

    if !MAILDIR_REGEX.is_match(maildir) {
        return Err(ValidationError::invalid_field(
            "maildir",
            "invalid maildir format (must end with '/' and contain only alphanumeric, '.', '_', '-', '/' characters)",
        ));
    }

    if maildir.contains("..") {
        return Err(ValidationError::invalid_field(
            "maildir",
            "must not contain '..' (path traversal)",
        ));
    }

    Ok(())
}

#[must_use]
pub fn generate_maildir(domain: &str, local_part: &str) -> String {
    format!("{domain}/{local_part}/")
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::types::DomainName;

    use super::*;

    fn test_domain(maxquota: i64, mailboxes: i32, aliases: i32, active: bool) -> Domain {
        Domain {
            domain: DomainName::from_trusted("example.com"),
            description: String::new(),
            aliases,
            mailboxes,
            maxquota,
            quota: 0,
            transport: None,
            backupmx: false,
            password_expiry: 0,
            active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn validate_quota_within_domain_unlimited_succeeds() {
        let domain = test_domain(0, 0, 0, true);
        assert!(validate_quota_within_domain(&domain, 999_999_999).is_ok());
    }

    #[test]
    fn validate_quota_within_domain_within_limit_succeeds() {
        let domain = test_domain(100, 0, 0, true);
        let quota_bytes = 50 * 1024 * 1024;
        assert!(validate_quota_within_domain(&domain, quota_bytes).is_ok());
    }

    #[test]
    fn validate_quota_within_domain_exceeds_limit_fails() {
        let domain = test_domain(100, 0, 0, true);
        let quota_bytes = 200 * 1024 * 1024;
        assert!(validate_quota_within_domain(&domain, quota_bytes).is_err());
    }

    #[test]
    fn validate_mailbox_count_within_limit_succeeds() {
        let domain = test_domain(0, 10, 0, true);
        assert!(validate_mailbox_count(&domain, 5).is_ok());
    }

    #[test]
    fn validate_mailbox_count_at_limit_fails() {
        let domain = test_domain(0, 10, 0, true);
        assert!(validate_mailbox_count(&domain, 10).is_err());
    }

    #[test]
    fn validate_alias_count_unlimited_succeeds() {
        let domain = test_domain(0, 0, 0, true);
        assert!(validate_alias_count(&domain, 999).is_ok());
    }

    #[test]
    fn validate_domain_active_active_succeeds() {
        let domain = test_domain(0, 0, 0, true);
        assert!(validate_domain_active(&domain).is_ok());
    }

    #[test]
    fn validate_domain_active_inactive_fails() {
        let domain = test_domain(0, 0, 0, false);
        assert!(validate_domain_active(&domain).is_err());
    }

    #[test]
    fn validate_alias_destinations_valid_succeeds() {
        let result = validate_alias_destinations("user@example.com, admin@example.com");
        assert!(result.is_ok());
        let destinations = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(destinations.len(), 2);
    }

    #[test]
    fn validate_alias_destinations_empty_fails() {
        assert!(validate_alias_destinations("").is_err());
    }

    #[test]
    fn validate_alias_destinations_no_at_sign_fails() {
        assert!(validate_alias_destinations("invalid").is_err());
    }

    #[test]
    fn validate_maildir_format_valid_succeeds() {
        assert!(validate_maildir_format("example.com/user/").is_ok());
    }

    #[test]
    fn validate_maildir_format_empty_fails() {
        assert!(validate_maildir_format("").is_err());
    }

    #[test]
    fn validate_maildir_format_no_trailing_slash_fails() {
        assert!(validate_maildir_format("example.com/user").is_err());
    }

    #[test]
    fn validate_maildir_format_path_traversal_fails() {
        assert!(validate_maildir_format("example.com/../etc/").is_err());
    }

    #[test]
    fn generate_maildir_produces_correct_format() {
        let result = generate_maildir("example.com", "user");
        assert_eq!(result, "example.com/user/");
    }
}

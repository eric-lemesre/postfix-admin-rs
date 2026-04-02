use std::sync::LazyLock;
use std::{fmt, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::ValidationError;
use crate::types::DomainName;

static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?i)[a-z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$")
        .unwrap_or_else(|_| unreachable!())
});

const MAX_EMAIL_LENGTH: usize = 255;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EmailAddress(String);

impl EmailAddress {
    /// # Errors
    /// Returns `ValidationError` if the email address is empty, too long, or not RFC 5321 compliant.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into().to_lowercase();

        if value.is_empty() {
            return Err(ValidationError::invalid_field("email", "must not be empty"));
        }

        if value.len() > MAX_EMAIL_LENGTH {
            return Err(ValidationError::invalid_field(
                "email",
                format!("must not exceed {MAX_EMAIL_LENGTH} characters"),
            ));
        }

        // PostfixAdmin catch-all format: @domain.com (no local part)
        let is_catch_all = value.starts_with('@') && value.len() > 1;
        if !is_catch_all && !EMAIL_REGEX.is_match(&value) {
            return Err(ValidationError::invalid_field(
                "email",
                "invalid email address format (RFC 5321)",
            ));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn catch_all(domain: &DomainName) -> Self {
        Self(format!("@{domain}"))
    }

    #[must_use]
    pub fn from_trusted(value: impl Into<String>) -> Self {
        Self(value.into().to_lowercase())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn local_part(&self) -> &str {
        self.0.rsplit_once('@').map_or("", |(local, _)| local)
    }

    #[must_use]
    pub fn domain_part(&self) -> &str {
        self.0.rsplit_once('@').map_or("", |(_, domain)| domain)
    }

    #[must_use]
    pub fn is_catch_all(&self) -> bool {
        self.0.starts_with('@')
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmailAddress({:?})", self.0)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<EmailAddress> for String {
    fn from(e: EmailAddress) -> Self {
        e.0
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for EmailAddress {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid_email_succeeds() {
        let email = EmailAddress::new("user@example.com");
        assert!(email.is_ok());
    }

    #[test]
    fn new_uppercase_normalized() {
        let email = EmailAddress::new("User@Example.COM");
        assert_eq!(
            email.map(|e| e.as_str().to_string()),
            Ok("user@example.com".to_string())
        );
    }

    #[test]
    fn new_empty_fails() {
        assert!(EmailAddress::new("").is_err());
    }

    #[test]
    fn new_no_at_sign_fails() {
        assert!(EmailAddress::new("userexample.com").is_err());
    }

    #[test]
    fn new_too_long_fails() {
        let long = format!("{}@example.com", "a".repeat(244));
        assert!(EmailAddress::new(long).is_err());
    }

    #[test]
    fn local_part_returns_local() {
        let email = EmailAddress::new("user@example.com").unwrap_or_else(|_| unreachable!());
        assert_eq!(email.local_part(), "user");
    }

    #[test]
    fn domain_part_returns_domain() {
        let email = EmailAddress::new("user@example.com").unwrap_or_else(|_| unreachable!());
        assert_eq!(email.domain_part(), "example.com");
    }

    #[test]
    fn catch_all_creates_correct_format() {
        let domain = DomainName::new("example.com").unwrap_or_else(|_| unreachable!());
        let email = EmailAddress::catch_all(&domain);
        assert!(email.is_catch_all());
        assert_eq!(email.as_str(), "@example.com");
    }

    #[test]
    fn is_catch_all_false_for_normal_email() {
        let email = EmailAddress::new("user@example.com").unwrap_or_else(|_| unreachable!());
        assert!(!email.is_catch_all());
    }

    #[test]
    fn from_trusted_skips_validation() {
        let email = EmailAddress::from_trusted("anything@anywhere");
        assert_eq!(email.as_str(), "anything@anywhere");
    }
}

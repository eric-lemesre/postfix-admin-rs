use std::sync::LazyLock;
use std::{fmt, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::ValidationError;

static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?i)[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$")
        .unwrap_or_else(|_| unreachable!())
});

const MAX_DOMAIN_LENGTH: usize = 255;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DomainName(String);

impl DomainName {
    /// # Errors
    /// Returns `ValidationError` if the domain name is empty, too long, or not RFC 1035 compliant.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into().to_lowercase();

        if value.is_empty() {
            return Err(ValidationError::invalid_field(
                "domain",
                "must not be empty",
            ));
        }

        if value.len() > MAX_DOMAIN_LENGTH {
            return Err(ValidationError::invalid_field(
                "domain",
                format!("must not exceed {MAX_DOMAIN_LENGTH} characters"),
            ));
        }

        if !DOMAIN_REGEX.is_match(&value) {
            return Err(ValidationError::invalid_field(
                "domain",
                "invalid domain name format (RFC 1035)",
            ));
        }

        if !value.contains('.') {
            return Err(ValidationError::invalid_field(
                "domain",
                "must contain at least one dot",
            ));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn from_trusted(value: impl Into<String>) -> Self {
        Self(value.into().to_lowercase())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DomainName({:?})", self.0)
    }
}

impl AsRef<str> for DomainName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<DomainName> for String {
    fn from(d: DomainName) -> Self {
        d.0
    }
}

impl TryFrom<String> for DomainName {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for DomainName {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid_domain_succeeds() {
        let domain = DomainName::new("example.com");
        assert!(domain.is_ok());
        assert_eq!(
            domain.map(|d| d.as_str().to_string()),
            Ok("example.com".to_string())
        );
    }

    #[test]
    fn new_subdomain_succeeds() {
        let domain = DomainName::new("mail.example.com");
        assert!(domain.is_ok());
    }

    #[test]
    fn new_uppercase_normalized_to_lowercase() {
        let domain = DomainName::new("Example.COM");
        assert!(domain.is_ok());
        assert_eq!(
            domain.map(|d| d.as_str().to_string()),
            Ok("example.com".to_string())
        );
    }

    #[test]
    fn new_empty_string_fails() {
        let domain = DomainName::new("");
        assert!(domain.is_err());
    }

    #[test]
    fn new_no_dot_fails() {
        let domain = DomainName::new("localhost");
        assert!(domain.is_err());
    }

    #[test]
    fn new_too_long_fails() {
        let long = format!("{}.com", "a".repeat(253));
        let domain = DomainName::new(long);
        assert!(domain.is_err());
    }

    #[test]
    fn new_invalid_characters_fails() {
        let domain = DomainName::new("exam ple.com");
        assert!(domain.is_err());
    }

    #[test]
    fn from_trusted_skips_validation() {
        let domain = DomainName::from_trusted("anything");
        assert_eq!(domain.as_str(), "anything");
    }

    #[test]
    fn display_shows_domain_name() {
        let domain = DomainName::new("test.org");
        assert_eq!(domain.map(|d| d.to_string()), Ok("test.org".to_string()));
    }
}

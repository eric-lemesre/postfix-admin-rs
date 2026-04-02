use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::ValidationError;

const MAX_IP_LENGTH: usize = 46;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct IpAddress(String);

impl IpAddress {
    /// # Errors
    /// Returns `ValidationError` if the IP address is empty, too long, or not a valid IPv4/IPv6 format.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();

        if value.is_empty() {
            return Err(ValidationError::invalid_field(
                "ip_address",
                "must not be empty",
            ));
        }

        if value.len() > MAX_IP_LENGTH {
            return Err(ValidationError::invalid_field(
                "ip_address",
                format!("must not exceed {MAX_IP_LENGTH} characters"),
            ));
        }

        if value.parse::<std::net::IpAddr>().is_err() {
            return Err(ValidationError::invalid_field(
                "ip_address",
                "invalid IP address format (IPv4 or IPv6)",
            ));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn from_trusted(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn is_ipv4(&self) -> bool {
        !self.0.contains(':')
    }

    #[must_use]
    pub fn is_ipv6(&self) -> bool {
        self.0.contains(':')
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IpAddress({:?})", self.0)
    }
}

impl AsRef<str> for IpAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<IpAddress> for String {
    fn from(ip: IpAddress) -> Self {
        ip.0
    }
}

impl TryFrom<String> for IpAddress {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for IpAddress {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid_ipv4_succeeds() {
        let ip = IpAddress::new("192.168.1.1");
        assert!(ip.is_ok());
    }

    #[test]
    fn new_valid_ipv6_succeeds() {
        let ip = IpAddress::new("::1");
        assert!(ip.is_ok());
    }

    #[test]
    fn new_empty_fails() {
        assert!(IpAddress::new("").is_err());
    }

    #[test]
    fn new_invalid_format_fails() {
        assert!(IpAddress::new("not-an-ip").is_err());
    }

    #[test]
    fn is_ipv4_returns_true_for_v4() {
        let ip = IpAddress::new("10.0.0.1").unwrap_or_else(|_| unreachable!());
        assert!(ip.is_ipv4());
        assert!(!ip.is_ipv6());
    }

    #[test]
    fn is_ipv6_returns_true_for_v6() {
        let ip = IpAddress::new("::1").unwrap_or_else(|_| unreachable!());
        assert!(ip.is_ipv6());
        assert!(!ip.is_ipv4());
    }

    #[test]
    fn from_trusted_skips_validation() {
        let ip = IpAddress::from_trusted("anything");
        assert_eq!(ip.as_str(), "anything");
    }
}

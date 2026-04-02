use std::fmt;

use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::ValidationError;

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 256;

#[derive(Clone, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(try_from = "String")]
pub struct Password(String);

impl Password {
    /// # Errors
    /// Returns `ValidationError` if the password is too short or too long.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();

        if value.len() < MIN_PASSWORD_LENGTH {
            return Err(ValidationError::invalid_field(
                "password",
                format!("must be at least {MIN_PASSWORD_LENGTH} characters"),
            ));
        }

        if value.len() > MAX_PASSWORD_LENGTH {
            return Err(ValidationError::invalid_field(
                "password",
                format!("must not exceed {MAX_PASSWORD_LENGTH} characters"),
            ));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(***)")
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl TryFrom<String> for Password {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid_password_succeeds() {
        let pwd = Password::new("securepassword123");
        assert!(pwd.is_ok());
    }

    #[test]
    fn new_too_short_fails() {
        let pwd = Password::new("short");
        assert!(pwd.is_err());
    }

    #[test]
    fn new_too_long_fails() {
        let pwd = Password::new("a".repeat(257));
        assert!(pwd.is_err());
    }

    #[test]
    fn debug_masks_password() {
        let pwd = Password::new("securepassword123").unwrap_or_else(|_| unreachable!());
        let debug = format!("{pwd:?}");
        assert_eq!(debug, "Password(***)");
        assert!(!debug.contains("secure"));
    }

    #[test]
    fn display_masks_password() {
        let pwd = Password::new("securepassword123").unwrap_or_else(|_| unreachable!());
        let display = format!("{pwd}");
        assert_eq!(display, "***");
    }

    // Password does not implement Serialize — this is enforced at compile time.
    // Any attempt to derive or implement Serialize on Password would be a security issue.
}

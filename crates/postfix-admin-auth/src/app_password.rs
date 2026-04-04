//! Application-specific password generation and verification.
//!
//! App passwords allow users to create dedicated passwords for mail clients
//! (IMAP, SMTP) without exposing their main account password.

use rand::Rng;

use crate::error::AuthError;
use crate::password::{hash_password, verify_password, PasswordScheme};

/// Characters used in app passwords (unambiguous alphanumerics).
/// Excludes O/0/l/1/I to avoid confusion.
const APP_PASSWORD_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789";

/// Length of generated app passwords.
const APP_PASSWORD_LEN: usize = 24;

/// Generate a new app password.
///
/// Returns a 24-character string of unambiguous alphanumeric characters.
///
/// # Errors
///
/// Returns `AuthError::HashingError` if random generation fails.
pub fn generate_app_password() -> Result<String, AuthError> {
    let mut rng = rand::rng();
    let password: String = (0..APP_PASSWORD_LEN)
        .map(|_| {
            let idx = rng.random_range(0..APP_PASSWORD_CHARS.len());
            char::from(APP_PASSWORD_CHARS[idx])
        })
        .collect();
    Ok(password)
}

/// Hash an app password for storage.
///
/// Uses Argon2id for strong hashing.
///
/// # Errors
///
/// Returns `AuthError::HashingError` if hashing fails.
pub fn hash_app_password(password: &str) -> Result<String, AuthError> {
    hash_password(password, PasswordScheme::Argon2id)
}

/// Verify an app password against a stored hash.
///
/// # Errors
///
/// Returns `AuthError::HashingError` if verification fails.
pub fn verify_app_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    verify_password(password, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_app_password_has_correct_length() {
        let password = generate_app_password().unwrap_or_else(|_| unreachable!());
        assert_eq!(password.len(), APP_PASSWORD_LEN);
    }

    #[test]
    fn generate_app_password_is_alphanumeric() {
        let password = generate_app_password().unwrap_or_else(|_| unreachable!());
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn generate_app_password_excludes_ambiguous_chars() {
        // Generate many passwords and check no ambiguous chars
        for _ in 0..100 {
            let password = generate_app_password().unwrap_or_else(|_| unreachable!());
            assert!(!password.contains('O'));
            assert!(!password.contains('0'));
            assert!(!password.contains('l'));
            assert!(!password.contains('1'));
            assert!(!password.contains('I'));
        }
    }

    #[test]
    fn generate_app_password_is_unique() {
        let p1 = generate_app_password().unwrap_or_else(|_| unreachable!());
        let p2 = generate_app_password().unwrap_or_else(|_| unreachable!());
        assert_ne!(p1, p2);
    }

    #[test]
    fn hash_and_verify_app_password_roundtrip() {
        let password = generate_app_password().unwrap_or_else(|_| unreachable!());
        let hash = hash_app_password(&password).unwrap_or_else(|_| unreachable!());
        let verified = verify_app_password(&password, &hash).unwrap_or_else(|_| unreachable!());
        assert!(verified);
    }

    #[test]
    fn verify_app_password_rejects_wrong_password() {
        let password = generate_app_password().unwrap_or_else(|_| unreachable!());
        let hash = hash_app_password(&password).unwrap_or_else(|_| unreachable!());
        let verified =
            verify_app_password("wrong_password", &hash).unwrap_or_else(|_| unreachable!());
        assert!(!verified);
    }
}

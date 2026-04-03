//! Multi-scheme password hashing and verification.
//!
//! Supports Argon2id, bcrypt, SHA-512 crypt, and SHA-256 crypt for hashing.
//! Additionally reads legacy MD5 crypt and cleartext (dev only) for verification.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use subtle::ConstantTimeEq;

use crate::error::AuthError;

/// Supported password schemes for hashing new passwords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordScheme {
    Argon2id,
    Bcrypt,
    Sha512Crypt,
    Sha256Crypt,
}

impl PasswordScheme {
    /// Parse a scheme name from configuration.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::UnsupportedScheme` for unknown schemes.
    pub fn from_config(s: &str) -> Result<Self, AuthError> {
        match s {
            "argon2id" => Ok(Self::Argon2id),
            "bcrypt" => Ok(Self::Bcrypt),
            "sha512-crypt" => Ok(Self::Sha512Crypt),
            "sha256-crypt" => Ok(Self::Sha256Crypt),
            other => Err(AuthError::UnsupportedScheme(other.to_string())),
        }
    }
}

/// Hash a password using the specified scheme.
///
/// # Errors
///
/// Returns `AuthError::HashingError` if hashing fails.
pub fn hash_password(password: &str, scheme: PasswordScheme) -> Result<String, AuthError> {
    match scheme {
        PasswordScheme::Argon2id => hash_argon2id(password),
        PasswordScheme::Bcrypt => hash_bcrypt(password),
        PasswordScheme::Sha512Crypt => hash_sha512_crypt(password),
        PasswordScheme::Sha256Crypt => hash_sha256_crypt(password),
    }
}

/// Verify a password against a stored hash.
///
/// Detects the scheme from the hash prefix and uses the appropriate
/// verification method. Supports all hashing schemes plus legacy
/// cleartext detection.
///
/// # Errors
///
/// Returns `AuthError::InvalidCredentials` if the password does not match.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    if hash.starts_with("$argon2") {
        verify_argon2id(password, hash)
    } else if hash.starts_with("$2b$") || hash.starts_with("$2a$") || hash.starts_with("$2y$") {
        verify_bcrypt(password, hash)
    } else if hash.starts_with("$6$") {
        Ok(verify_sha512_crypt(password, hash))
    } else if hash.starts_with("$5$") {
        Ok(verify_sha256_crypt(password, hash))
    } else if hash.starts_with("$1$") {
        // MD5 crypt — legacy read-only, always reject for security
        tracing::warn!("MD5 crypt hash detected — legacy scheme, rejecting authentication");
        Ok(false)
    } else if is_des_crypt_hash(hash) {
        // DES crypt — legacy read-only, always reject for security
        tracing::warn!("DES crypt hash detected — legacy scheme, rejecting authentication");
        Ok(false)
    } else {
        // Cleartext comparison (dev mode only — caller should check config)
        // Uses constant-time comparison to prevent timing attacks
        let result: bool = password.as_bytes().ct_eq(hash.as_bytes()).into();
        Ok(result)
    }
}

/// Check if a hash uses a legacy scheme that should be rehashed.
#[must_use]
pub fn needs_rehash(hash: &str, target: PasswordScheme) -> bool {
    match target {
        PasswordScheme::Argon2id => !hash.starts_with("$argon2"),
        PasswordScheme::Bcrypt => {
            !hash.starts_with("$2b$") && !hash.starts_with("$2a$") && !hash.starts_with("$2y$")
        }
        PasswordScheme::Sha512Crypt => !hash.starts_with("$6$"),
        PasswordScheme::Sha256Crypt => !hash.starts_with("$5$"),
    }
}

/// Check if a hash looks like a DES crypt hash.
///
/// DES crypt hashes are exactly 13 characters from the set `[a-zA-Z0-9./]`.
fn is_des_crypt_hash(hash: &str) -> bool {
    hash.len() == 13
        && hash
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'/')
}

// --- Argon2id ---

fn hash_argon2id(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::HashingError(e.to_string()))?;
    Ok(hash.to_string())
}

fn verify_argon2id(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AuthError::HashingError(format!("invalid argon2 hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// --- Bcrypt ---

fn hash_bcrypt(password: &str) -> Result<String, AuthError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| AuthError::HashingError(e.to_string()))
}

fn verify_bcrypt(password: &str, hash: &str) -> Result<bool, AuthError> {
    bcrypt::verify(password, hash).map_err(|e| AuthError::HashingError(e.to_string()))
}

// --- SHA-512 crypt ---

fn hash_sha512_crypt(password: &str) -> Result<String, AuthError> {
    let params = sha_crypt::Sha512Params::new(5000)
        .map_err(|e| AuthError::HashingError(format!("{e:?}")))?;
    sha_crypt::sha512_simple(password, &params)
        .map_err(|e| AuthError::HashingError(format!("{e:?}")))
}

fn verify_sha512_crypt(password: &str, hash: &str) -> bool {
    sha_crypt::sha512_check(password, hash).is_ok()
}

// --- SHA-256 crypt ---

fn hash_sha256_crypt(password: &str) -> Result<String, AuthError> {
    let params = sha_crypt::Sha256Params::new(5000)
        .map_err(|e| AuthError::HashingError(format!("{e:?}")))?;
    sha_crypt::sha256_simple(password, &params)
        .map_err(|e| AuthError::HashingError(format!("{e:?}")))
}

fn verify_sha256_crypt(password: &str, hash: &str) -> bool {
    sha_crypt::sha256_check(password, hash).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_argon2id() {
        let hash = hash_password("test_password", PasswordScheme::Argon2id);
        assert!(hash.is_ok());
        let hash = hash.unwrap_or_else(|_| unreachable!());
        assert!(hash.starts_with("$argon2"));
        let verified = verify_password("test_password", &hash);
        assert!(verified.is_ok());
        assert!(verified.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn hash_and_verify_bcrypt() {
        let hash = hash_password("test_password", PasswordScheme::Bcrypt);
        assert!(hash.is_ok());
        let hash = hash.unwrap_or_else(|_| unreachable!());
        assert!(hash.starts_with("$2b$"));
        let verified = verify_password("test_password", &hash);
        assert!(verified.is_ok());
        assert!(verified.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn hash_and_verify_sha512_crypt() {
        let hash = hash_password("test_password", PasswordScheme::Sha512Crypt);
        assert!(hash.is_ok());
        let hash = hash.unwrap_or_else(|_| unreachable!());
        assert!(hash.starts_with("$6$"));
        let verified = verify_password("test_password", &hash);
        assert!(verified.is_ok());
        assert!(verified.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn hash_and_verify_sha256_crypt() {
        let hash = hash_password("test_password", PasswordScheme::Sha256Crypt);
        assert!(hash.is_ok());
        let hash = hash.unwrap_or_else(|_| unreachable!());
        assert!(hash.starts_with("$5$"));
        let verified = verify_password("test_password", &hash);
        assert!(verified.is_ok());
        assert!(verified.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn verify_wrong_password_returns_false() {
        let hash = hash_password("correct_password", PasswordScheme::Argon2id)
            .unwrap_or_else(|_| unreachable!());
        let verified = verify_password("wrong_password", &hash);
        assert!(verified.is_ok());
        assert!(!verified.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn verify_cleartext_fallback() {
        let result = verify_password("cleartext", "cleartext");
        assert!(result.is_ok());
        assert!(result.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn verify_cleartext_wrong_returns_false() {
        let result = verify_password("wrong", "cleartext");
        assert!(result.is_ok());
        assert!(!result.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn needs_rehash_detects_legacy_scheme() {
        let bcrypt_hash =
            hash_password("test", PasswordScheme::Bcrypt).unwrap_or_else(|_| unreachable!());
        assert!(needs_rehash(&bcrypt_hash, PasswordScheme::Argon2id));
        assert!(!needs_rehash(&bcrypt_hash, PasswordScheme::Bcrypt));
    }

    #[test]
    fn password_scheme_from_config_valid() {
        assert!(PasswordScheme::from_config("argon2id").is_ok());
        assert!(PasswordScheme::from_config("bcrypt").is_ok());
        assert!(PasswordScheme::from_config("sha512-crypt").is_ok());
        assert!(PasswordScheme::from_config("sha256-crypt").is_ok());
    }

    #[test]
    fn password_scheme_from_config_invalid() {
        assert!(PasswordScheme::from_config("md5").is_err());
    }

    #[test]
    fn verify_des_crypt_hash_returns_false() {
        // DES crypt hash: 13 chars of [a-zA-Z0-9./]
        let result = verify_password("password", "abJnggxhB/yWI");
        assert!(result.is_ok());
        assert!(!result.unwrap_or_else(|_| unreachable!()));
    }

    #[test]
    fn verify_md5_crypt_hash_returns_false() {
        let result = verify_password("password", "$1$salt$hash1234567890ab");
        assert!(result.is_ok());
        assert!(!result.unwrap_or_else(|_| unreachable!()));
    }
}

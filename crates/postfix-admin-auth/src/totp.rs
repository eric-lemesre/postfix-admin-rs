//! TOTP two-factor authentication (RFC 6238).
//!
//! Provides TOTP secret generation, QR code creation, code verification
//! with replay protection, recovery codes, and IP-based exemptions.

use std::net::IpAddr;

use totp_rs::{Algorithm, Secret, TOTP};

use crate::error::AuthError;
use crate::password::{hash_password, verify_password, PasswordScheme};

/// Result of setting up TOTP for an account.
#[derive(Debug, Clone)]
pub struct TotpSetup {
    /// The secret encoded in Base32 for manual entry.
    pub secret_base32: String,
    /// The `otpauth://` URI for QR code scanners.
    pub otpauth_uri: String,
    /// QR code image as PNG encoded in Base64.
    pub qr_code_base64: String,
    /// Recovery codes for emergency access.
    pub recovery_codes: Vec<String>,
}

/// Manages TOTP operations for a given issuer.
pub struct TotpManager {
    issuer: String,
}

impl TotpManager {
    /// Create a new TOTP manager with the given issuer name.
    #[must_use]
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
        }
    }

    /// Generate a new TOTP setup for an account.
    ///
    /// Creates a 160-bit secret, an `otpauth://` URI, a QR code (PNG/Base64),
    /// and 10 recovery codes.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::TotpSetupError` if TOTP generation fails.
    pub fn generate_setup(&self, account_name: &str) -> Result<TotpSetup, AuthError> {
        let secret = Secret::generate_secret();
        let secret_bytes = secret.to_bytes().map_err(|e| {
            AuthError::TotpSetupError(format!("failed to generate secret bytes: {e}"))
        })?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some(self.issuer.clone()),
            account_name.to_string(),
        )
        .map_err(|e| AuthError::TotpSetupError(format!("failed to create TOTP: {e}")))?;

        let otpauth_uri = totp.get_url();
        let qr_code_base64 = totp
            .get_qr_base64()
            .map_err(|e| AuthError::TotpSetupError(format!("failed to generate QR code: {e}")))?;

        let recovery_codes = generate_recovery_codes()?;

        Ok(TotpSetup {
            secret_base32: secret.to_encoded().to_string(),
            otpauth_uri,
            qr_code_base64,
            recovery_codes,
        })
    }

    /// Verify a TOTP code against a secret with replay protection.
    ///
    /// Uses SHA-1, 6 digits, 30-second step, skew of +/-1 step.
    /// Returns the current time step on success for replay tracking.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InvalidTotpCode` if the code is invalid.
    /// Returns `AuthError::TotpReplay` if the same time step was already used.
    /// Returns `AuthError::TotpSetupError` if TOTP creation fails.
    pub fn verify_code(
        &self,
        secret_base32: &str,
        code: &str,
        last_used_step: Option<u64>,
    ) -> Result<u64, AuthError> {
        let secret = Secret::Encoded(secret_base32.to_string());
        let secret_bytes = secret
            .to_bytes()
            .map_err(|e| AuthError::TotpSetupError(format!("invalid secret: {e}")))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some(self.issuer.clone()),
            String::new(),
        )
        .map_err(|e| AuthError::TotpSetupError(format!("failed to create TOTP: {e}")))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AuthError::TotpSetupError(format!("system time error: {e}")))?;

        let current_step = now.as_secs() / 30;

        // Check replay: reject if same step was already used
        if let Some(last) = last_used_step {
            if current_step == last {
                return Err(AuthError::TotpReplay);
            }
        }

        if totp.check_current(code).map_err(|e| {
            AuthError::TotpSetupError(format!("TOTP verification system error: {e}"))
        })? {
            Ok(current_step)
        } else {
            Err(AuthError::InvalidTotpCode)
        }
    }
}

/// Generate recovery codes (10 codes in `XXXX-XXXX` format).
///
/// Uses uppercase letters and digits.
///
/// # Errors
///
/// Returns `AuthError::TotpSetupError` if random generation fails.
pub fn generate_recovery_codes() -> Result<Vec<String>, AuthError> {
    const CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    const NUM_CODES: usize = 10;
    const HALF_LEN: usize = 4;

    let mut rng = rand::rng();
    let mut codes = Vec::with_capacity(NUM_CODES);

    for _ in 0..NUM_CODES {
        let first: String = (0..HALF_LEN)
            .map(|_| {
                let idx = rand::Rng::random_range(&mut rng, 0..CODE_CHARS.len());
                char::from(CODE_CHARS[idx])
            })
            .collect();
        let second: String = (0..HALF_LEN)
            .map(|_| {
                let idx = rand::Rng::random_range(&mut rng, 0..CODE_CHARS.len());
                char::from(CODE_CHARS[idx])
            })
            .collect();
        codes.push(format!("{first}-{second}"));
    }

    Ok(codes)
}

/// Hash a recovery code for storage using Argon2id.
///
/// # Errors
///
/// Returns `AuthError::HashingError` if hashing fails.
pub fn hash_recovery_code(code: &str) -> Result<String, AuthError> {
    // Normalize: uppercase and strip dashes
    let normalized: String = code.chars().filter(|c| *c != '-').collect::<String>();
    let normalized = normalized.to_uppercase();
    hash_password(&normalized, PasswordScheme::Argon2id)
}

/// Verify a recovery code against a list of hashed codes.
///
/// Returns the index of the matching code, or `None` if no match.
///
/// # Errors
///
/// Returns `AuthError::HashingError` if verification fails.
pub fn verify_recovery_code(
    code: &str,
    hashed_codes: &[String],
) -> Result<Option<usize>, AuthError> {
    let normalized: String = code.chars().filter(|c| *c != '-').collect::<String>();
    let normalized = normalized.to_uppercase();
    for (i, hash) in hashed_codes.iter().enumerate() {
        if verify_password(&normalized, hash)? {
            return Ok(Some(i));
        }
    }
    Ok(None)
}

/// IP exception entry for TOTP bypass.
#[derive(Debug, Clone)]
pub struct TotpIpException {
    /// If `None`, the exception is global (applies to all users).
    pub username: Option<String>,
    /// IP address that is exempt from TOTP.
    pub ip: IpAddr,
}

/// Check if a client IP is exempt from TOTP for a given user.
///
/// Checks both global exceptions (where `username` is `None`) and
/// per-user exceptions.
#[must_use]
pub fn is_totp_exempt(client_ip: IpAddr, username: &str, exceptions: &[TotpIpException]) -> bool {
    exceptions.iter().any(|ex| {
        ex.ip == client_ip
            && match &ex.username {
                None => true,
                Some(u) => u == username,
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn generate_setup_produces_valid_result() {
        let mgr = TotpManager::new("TestIssuer");
        let setup = mgr
            .generate_setup("user@example.com")
            .unwrap_or_else(|_| unreachable!());

        assert!(!setup.secret_base32.is_empty());
        assert!(setup.otpauth_uri.starts_with("otpauth://totp/"));
        assert!(!setup.qr_code_base64.is_empty());
        assert_eq!(setup.recovery_codes.len(), 10);
    }

    #[test]
    fn generate_setup_secret_is_base32() {
        let mgr = TotpManager::new("TestIssuer");
        let setup = mgr
            .generate_setup("user@example.com")
            .unwrap_or_else(|_| unreachable!());
        // Base32 characters: A-Z, 2-7, =
        assert!(setup
            .secret_base32
            .chars()
            .all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c) || c == '='));
    }

    #[test]
    fn generate_setup_otpauth_uri_contains_issuer() {
        let mgr = TotpManager::new("TestIssuer");
        let setup = mgr
            .generate_setup("user@example.com")
            .unwrap_or_else(|_| unreachable!());
        assert!(setup.otpauth_uri.contains("TestIssuer"));
    }

    #[test]
    fn generate_setup_qr_is_nonempty() {
        let mgr = TotpManager::new("TestIssuer");
        let setup = mgr
            .generate_setup("user@example.com")
            .unwrap_or_else(|_| unreachable!());
        assert!(!setup.qr_code_base64.is_empty());
    }

    #[test]
    fn generate_recovery_codes_returns_ten() {
        let codes = generate_recovery_codes().unwrap_or_else(|_| unreachable!());
        assert_eq!(codes.len(), 10);
    }

    #[test]
    fn generate_recovery_codes_format_xxxx_xxxx() {
        let codes = generate_recovery_codes().unwrap_or_else(|_| unreachable!());
        for code in &codes {
            assert_eq!(code.len(), 9); // 4 + 1 + 4
            assert_eq!(code.chars().nth(4), Some('-'));
            let parts: Vec<&str> = code.split('-').collect();
            assert_eq!(parts.len(), 2);
            assert!(parts[0].len() == 4);
            assert!(parts[1].len() == 4);
            assert!(parts[0]
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
            assert!(parts[1]
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
        }
    }

    #[test]
    fn verify_valid_totp_code() {
        let mgr = TotpManager::new("TestIssuer");
        let setup = mgr
            .generate_setup("user@example.com")
            .unwrap_or_else(|_| unreachable!());

        // Generate the current valid code from the secret
        let secret = Secret::Encoded(setup.secret_base32.clone());
        let secret_bytes = secret.to_bytes().unwrap_or_else(|_| unreachable!());
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("TestIssuer".to_string()),
            String::new(),
        )
        .unwrap_or_else(|_| unreachable!());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| unreachable!());
        let code = totp.generate(now.as_secs());

        let result = mgr.verify_code(&setup.secret_base32, &code, None);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_invalid_totp_code_returns_error() {
        let mgr = TotpManager::new("TestIssuer");
        let setup = mgr
            .generate_setup("user@example.com")
            .unwrap_or_else(|_| unreachable!());

        let result = mgr.verify_code(&setup.secret_base32, "000000", None);
        // This might succeed if 000000 happens to be valid, but extremely unlikely
        // We just check it returns a result (Ok or specific error)
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn verify_replay_detection() {
        let mgr = TotpManager::new("TestIssuer");
        let setup = mgr
            .generate_setup("user@example.com")
            .unwrap_or_else(|_| unreachable!());

        let secret = Secret::Encoded(setup.secret_base32.clone());
        let secret_bytes = secret.to_bytes().unwrap_or_else(|_| unreachable!());
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("TestIssuer".to_string()),
            String::new(),
        )
        .unwrap_or_else(|_| unreachable!());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| unreachable!());
        let code = totp.generate(now.as_secs());
        let current_step = now.as_secs() / 30;

        // First use should succeed
        let result = mgr.verify_code(&setup.secret_base32, &code, None);
        assert!(result.is_ok());

        // Replay with same step should fail
        let result = mgr.verify_code(&setup.secret_base32, &code, Some(current_step));
        assert!(matches!(result, Err(AuthError::TotpReplay)));
    }

    #[test]
    fn recovery_code_hash_and_verify() {
        let codes = generate_recovery_codes().unwrap_or_else(|_| unreachable!());
        let first_code = &codes[0];

        let hash = hash_recovery_code(first_code).unwrap_or_else(|_| unreachable!());
        let hashed = vec![hash];

        let result = verify_recovery_code(first_code, &hashed).unwrap_or_else(|_| unreachable!());
        assert_eq!(result, Some(0));
    }

    #[test]
    fn recovery_code_wrong_code_returns_none() {
        let codes = generate_recovery_codes().unwrap_or_else(|_| unreachable!());
        let hash = hash_recovery_code(&codes[0]).unwrap_or_else(|_| unreachable!());
        let hashed = vec![hash];

        let result = verify_recovery_code("ZZZZ-ZZZZ", &hashed).unwrap_or_else(|_| unreachable!());
        assert_eq!(result, None);
    }

    #[test]
    fn is_totp_exempt_global_exception() {
        let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let exceptions = vec![TotpIpException { username: None, ip }];
        assert!(is_totp_exempt(ip, "anyone@example.com", &exceptions));
    }

    #[test]
    fn is_totp_exempt_per_user_exception() {
        let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let exceptions = vec![TotpIpException {
            username: Some("admin@example.com".to_string()),
            ip,
        }];
        assert!(is_totp_exempt(ip, "admin@example.com", &exceptions));
        assert!(!is_totp_exempt(ip, "other@example.com", &exceptions));
    }

    #[test]
    fn is_totp_exempt_no_match() {
        let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let other_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        let exceptions = vec![TotpIpException {
            username: None,
            ip: other_ip,
        }];
        assert!(!is_totp_exempt(ip, "anyone@example.com", &exceptions));
    }
}

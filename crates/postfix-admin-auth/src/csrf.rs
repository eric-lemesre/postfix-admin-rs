//! CSRF (Cross-Site Request Forgery) token generation and verification.
//!
//! Uses cryptographically random tokens with constant-time comparison
//! to prevent CSRF attacks on form submissions.

use data_encoding::BASE64URL_NOPAD;
use subtle::ConstantTimeEq;

use crate::error::AuthError;

/// Generate a CSRF token (32 random bytes encoded as base64url).
///
/// # Errors
///
/// Returns `AuthError::HashingError` if random byte generation fails.
pub fn generate_csrf_token() -> Result<String, AuthError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|e| AuthError::HashingError(format!("CSRF token generation failed: {e}")))?;
    Ok(BASE64URL_NOPAD.encode(&bytes))
}

/// Verify a submitted CSRF token against the stored token.
///
/// Uses constant-time comparison to prevent timing attacks.
///
/// # Errors
///
/// Returns `AuthError::CsrfError` if the tokens do not match.
pub fn verify_csrf_token(submitted: &str, stored: &str) -> Result<(), AuthError> {
    if bool::from(submitted.as_bytes().ct_eq(stored.as_bytes())) {
        Ok(())
    } else {
        Err(AuthError::CsrfError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_csrf_token_is_nonempty() {
        let token = generate_csrf_token().unwrap_or_else(|_| unreachable!());
        assert!(!token.is_empty());
    }

    #[test]
    fn generate_csrf_token_is_unique() {
        let t1 = generate_csrf_token().unwrap_or_else(|_| unreachable!());
        let t2 = generate_csrf_token().unwrap_or_else(|_| unreachable!());
        assert_ne!(t1, t2);
    }

    #[test]
    fn verify_csrf_token_matching_succeeds() {
        let token = generate_csrf_token().unwrap_or_else(|_| unreachable!());
        let result = verify_csrf_token(&token, &token);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_csrf_token_mismatch_fails() {
        let t1 = generate_csrf_token().unwrap_or_else(|_| unreachable!());
        let t2 = generate_csrf_token().unwrap_or_else(|_| unreachable!());
        let result = verify_csrf_token(&t1, &t2);
        assert!(matches!(result, Err(AuthError::CsrfError)));
    }
}

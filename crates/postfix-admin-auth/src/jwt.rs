//! JWT (JSON Web Token) generation and verification.
//!
//! Issues access and refresh tokens for API authentication.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AuthError;

/// JWT claims embedded in access and refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the admin username (email).
    pub sub: String,
    /// Whether the subject is a superadmin.
    pub superadmin: bool,
    /// Token type: "access" or "refresh".
    pub token_type: String,
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Expiration (Unix timestamp).
    pub exp: i64,
}

/// JWT token pair returned after successful authentication.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// JWT token manager.
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_lifetime_secs: i64,
    refresh_lifetime_secs: i64,
}

impl JwtManager {
    /// Create a new JWT manager with the given secret key.
    #[must_use]
    pub fn new(secret: &[u8], access_lifetime_secs: i64, refresh_lifetime_secs: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            access_lifetime_secs,
            refresh_lifetime_secs,
        }
    }

    /// Issue a token pair for an authenticated admin.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InvalidToken` if token encoding fails.
    pub fn issue(&self, username: &str, superadmin: bool) -> Result<TokenPair, AuthError> {
        let now = Utc::now();

        let access_claims = Claims {
            sub: username.to_string(),
            superadmin,
            token_type: "access".to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.access_lifetime_secs)).timestamp(),
        };

        let refresh_claims = Claims {
            sub: username.to_string(),
            superadmin,
            token_type: "refresh".to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.refresh_lifetime_secs)).timestamp(),
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_lifetime_secs,
        })
    }

    /// Verify and decode an access token.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::TokenExpired` or `AuthError::InvalidToken`.
    pub fn verify_access(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self.decode(token)?;
        if claims.token_type != "access" {
            return Err(AuthError::InvalidToken("expected access token".to_string()));
        }
        Ok(claims)
    }

    /// Verify and decode a refresh token.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::TokenExpired` or `AuthError::InvalidToken`.
    pub fn verify_refresh(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self.decode(token)?;
        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken(
                "expected refresh token".to_string(),
            ));
        }
        Ok(claims)
    }

    fn decode(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(e.to_string()),
            })?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manager() -> JwtManager {
        JwtManager::new(b"test-secret-key-for-jwt-testing!", 900, 604_800)
    }

    #[test]
    fn issue_and_verify_access_token() {
        let mgr = test_manager();
        let pair = mgr.issue("admin@example.com", true);
        assert!(pair.is_ok());
        let pair = pair.unwrap_or_else(|_| unreachable!());
        assert_eq!(pair.token_type, "Bearer");
        assert_eq!(pair.expires_in, 900);

        let claims = mgr.verify_access(&pair.access_token);
        assert!(claims.is_ok());
        let claims = claims.unwrap_or_else(|_| unreachable!());
        assert_eq!(claims.sub, "admin@example.com");
        assert!(claims.superadmin);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn issue_and_verify_refresh_token() {
        let mgr = test_manager();
        let pair = mgr
            .issue("admin@example.com", false)
            .unwrap_or_else(|_| unreachable!());
        let claims = mgr.verify_refresh(&pair.refresh_token);
        assert!(claims.is_ok());
        let claims = claims.unwrap_or_else(|_| unreachable!());
        assert_eq!(claims.sub, "admin@example.com");
        assert!(!claims.superadmin);
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn verify_access_rejects_refresh_token() {
        let mgr = test_manager();
        let pair = mgr
            .issue("admin@example.com", false)
            .unwrap_or_else(|_| unreachable!());
        let result = mgr.verify_access(&pair.refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn verify_refresh_rejects_access_token() {
        let mgr = test_manager();
        let pair = mgr
            .issue("admin@example.com", false)
            .unwrap_or_else(|_| unreachable!());
        let result = mgr.verify_refresh(&pair.access_token);
        assert!(result.is_err());
    }

    #[test]
    fn verify_invalid_token_returns_error() {
        let mgr = test_manager();
        let result = mgr.verify_access("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn verify_wrong_secret_returns_error() {
        let mgr1 = test_manager();
        let mgr2 = JwtManager::new(b"different-secret-key-for-testing", 900, 604_800);
        let pair = mgr1
            .issue("admin@example.com", false)
            .unwrap_or_else(|_| unreachable!());
        let result = mgr2.verify_access(&pair.access_token);
        assert!(result.is_err());
    }
}

//! Authentication error types.

use thiserror::Error;

/// Errors that can occur during authentication operations.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Invalid credentials (wrong username or password).
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Account is locked due to too many failed attempts.
    #[error("account locked until {0}")]
    AccountLocked(String),

    /// Account is inactive.
    #[error("account is inactive")]
    AccountInactive,

    /// Token is expired.
    #[error("token expired")]
    TokenExpired,

    /// Token is invalid (malformed, wrong signature, etc.).
    #[error("invalid token: {0}")]
    InvalidToken(String),

    /// Missing authorization header or token.
    #[error("missing authorization")]
    MissingAuthorization,

    /// Insufficient permissions for the requested operation.
    #[error("insufficient permissions: {0}")]
    InsufficientPermissions(String),

    /// Password hashing failed.
    #[error("password hashing error: {0}")]
    HashingError(String),

    /// Unsupported password scheme.
    #[error("unsupported password scheme: {0}")]
    UnsupportedScheme(String),
}

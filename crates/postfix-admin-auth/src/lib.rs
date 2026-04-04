//! Authentication, sessions, JWT, TOTP, and RBAC for postfix-admin-rs.
//!
//! Handles multi-scheme password verification, session management,
//! JSON Web Tokens, TOTP two-factor authentication, and role-based access control.

pub mod app_password;
pub mod csrf;
pub mod error;
pub mod jwt;
pub mod mtls;
pub mod password;
pub mod rate_limit;
pub mod rbac;
pub mod totp;

pub use app_password::{generate_app_password, hash_app_password, verify_app_password};
pub use csrf::{generate_csrf_token, verify_csrf_token};
pub use error::AuthError;
pub use jwt::{Claims, JwtManager, TokenPair};
pub use mtls::{CertificateInfo, MtlsError, MtlsVerifier};
pub use password::{hash_password, needs_rehash, verify_password, PasswordScheme};
pub use rate_limit::LoginRateLimiter;
pub use rbac::{AuthIdentity, Role};
pub use totp::{TotpManager, TotpSetup};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_loads() {
        // Smoke test: the crate compiles and links correctly.
    }
}

//! Authentication, sessions, JWT, TOTP, and RBAC for postfix-admin-rs.
//!
//! Handles multi-scheme password verification, session management,
//! JSON Web Tokens, TOTP two-factor authentication, and role-based access control.

pub mod error;
pub mod jwt;
pub mod mtls;
pub mod password;

pub use error::AuthError;
pub use jwt::{Claims, JwtManager, TokenPair};
pub use mtls::{CertificateInfo, MtlsError, MtlsVerifier};
pub use password::{hash_password, needs_rehash, verify_password, PasswordScheme};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_loads() {
        // Smoke test: the crate compiles and links correctly.
    }
}

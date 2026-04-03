//! Authentication, sessions, JWT, TOTP, and RBAC for postfix-admin-rs.
//!
//! Handles multi-scheme password verification, session management,
//! JSON Web Tokens, TOTP two-factor authentication, and role-based access control.

pub mod mtls;

pub use mtls::{CertificateInfo, MtlsError, MtlsVerifier};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_loads() {
        // Smoke test: the crate compiles and links correctly.
    }
}

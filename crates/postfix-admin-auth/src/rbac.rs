//! Role-Based Access Control (RBAC) for postfix-admin-rs.
//!
//! Defines roles, authentication identity, and access control checks
//! for domain-level and assurance-level authorization.

use crate::error::AuthError;

/// User roles in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Full system administrator with access to all domains.
    SuperAdmin,
    /// Administrator of specific domains.
    DomainAdmin,
    /// Regular mailbox user.
    User,
}

/// Authenticated identity with role and verification status.
#[derive(Debug, Clone)]
pub struct AuthIdentity {
    /// The authenticated username (email address).
    pub username: String,
    /// The user's role.
    pub role: Role,
    /// Whether TOTP verification has been completed.
    pub totp_verified: bool,
    /// Whether mTLS client certificate has been verified.
    pub mtls_verified: bool,
}

/// Check if an admin has access to a specific domain.
///
/// - Superadmins always have access to all domains.
/// - Domain admins only have access to their assigned domains.
///
/// # Errors
///
/// Returns `AuthError::InsufficientPermissions` if access is denied.
pub fn check_domain_access(
    role: Role,
    is_superadmin: bool,
    target_domain: &str,
    admin_domains: &[String],
) -> Result<(), AuthError> {
    if is_superadmin || role == Role::SuperAdmin {
        return Ok(());
    }
    if admin_domains.iter().any(|d| d == target_domain) {
        return Ok(());
    }
    Err(AuthError::InsufficientPermissions(format!(
        "no access to domain '{target_domain}'"
    )))
}

/// Check if the authenticated identity meets the required assurance level.
///
/// Verifies that mTLS requirements are met based on role and configuration.
///
/// # Errors
///
/// Returns `AuthError::InsufficientPermissions` if assurance level is not met.
pub fn check_assurance_level(
    identity: &AuthIdentity,
    require_mtls_superadmin: bool,
    require_mtls_domain_admin: bool,
) -> Result<(), AuthError> {
    if require_mtls_superadmin && identity.role == Role::SuperAdmin && !identity.mtls_verified {
        return Err(AuthError::InsufficientPermissions(
            "mTLS client certificate required for superadmin access".to_string(),
        ));
    }
    if require_mtls_domain_admin && identity.role == Role::DomainAdmin && !identity.mtls_verified {
        return Err(AuthError::InsufficientPermissions(
            "mTLS client certificate required for domain admin access".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superadmin_always_has_domain_access() {
        let result = check_domain_access(Role::SuperAdmin, true, "example.com", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn domain_admin_access_to_own_domain() {
        let domains = vec!["example.com".to_string()];
        let result = check_domain_access(Role::DomainAdmin, false, "example.com", &domains);
        assert!(result.is_ok());
    }

    #[test]
    fn domain_admin_denied_other_domain() {
        let domains = vec!["example.com".to_string()];
        let result = check_domain_access(Role::DomainAdmin, false, "other.com", &domains);
        assert!(matches!(result, Err(AuthError::InsufficientPermissions(_))));
    }

    #[test]
    fn is_superadmin_flag_grants_access() {
        let result = check_domain_access(Role::DomainAdmin, true, "any-domain.com", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn assurance_level_superadmin_requires_mtls() {
        let identity = AuthIdentity {
            username: "admin@example.com".to_string(),
            role: Role::SuperAdmin,
            totp_verified: true,
            mtls_verified: false,
        };
        let result = check_assurance_level(&identity, true, false);
        assert!(matches!(result, Err(AuthError::InsufficientPermissions(_))));
    }

    #[test]
    fn assurance_level_superadmin_with_mtls_succeeds() {
        let identity = AuthIdentity {
            username: "admin@example.com".to_string(),
            role: Role::SuperAdmin,
            totp_verified: true,
            mtls_verified: true,
        };
        let result = check_assurance_level(&identity, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn assurance_level_domain_admin_requires_mtls() {
        let identity = AuthIdentity {
            username: "admin@example.com".to_string(),
            role: Role::DomainAdmin,
            totp_verified: true,
            mtls_verified: false,
        };
        let result = check_assurance_level(&identity, false, true);
        assert!(matches!(result, Err(AuthError::InsufficientPermissions(_))));
    }

    #[test]
    fn assurance_level_no_mtls_required_succeeds() {
        let identity = AuthIdentity {
            username: "admin@example.com".to_string(),
            role: Role::SuperAdmin,
            totp_verified: false,
            mtls_verified: false,
        };
        let result = check_assurance_level(&identity, false, false);
        assert!(result.is_ok());
    }
}

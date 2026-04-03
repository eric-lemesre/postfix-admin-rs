//! Mutual TLS (client certificate) verification for administrator authentication.
//!
//! Extracts and validates client certificate information from HTTP headers
//! set by a reverse proxy (Nginx, Apache). The reverse proxy handles the
//! actual TLS handshake and certificate verification; this module trusts
//! the proxy headers and extracts identity information from them.

use std::collections::HashMap;

use postfix_admin_core::config::MtlsConfig;
use thiserror::Error;

/// Information extracted from a verified client certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateInfo {
    /// The identity extracted from the configured DN field (e.g. email address).
    pub identity: String,
    /// The full subject DN as received from the proxy header.
    pub subject_dn: String,
    /// The certificate serial number, if provided.
    pub serial: Option<String>,
}

/// Errors that can occur during mTLS certificate extraction.
#[derive(Debug, Error)]
pub enum MtlsError {
    /// mTLS is disabled in the configuration.
    #[error("mTLS client certificate authentication is disabled")]
    Disabled,

    /// The verification header from the reverse proxy is missing.
    #[error("missing client certificate verification header '{0}'")]
    MissingVerificationHeader(String),

    /// The reverse proxy reports that client certificate verification failed.
    #[error("client certificate verification failed (proxy reported: '{0}')")]
    VerificationFailed(String),

    /// The subject DN header is missing.
    #[error("missing client certificate subject header '{0}'")]
    MissingSubjectHeader(String),

    /// The configured identity field was not found in the subject DN.
    #[error("field '{field}' not found in subject DN '{dn}'")]
    FieldNotFound { field: String, dn: String },
}

/// Verifies and extracts client certificate information from reverse proxy headers.
pub struct MtlsVerifier {
    config: MtlsConfig,
}

impl MtlsVerifier {
    /// Create a new verifier from the mTLS configuration.
    #[must_use]
    pub fn new(config: MtlsConfig) -> Self {
        Self { config }
    }

    /// Whether mTLS is enabled in the configuration.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Whether client certificates are required for superadmin accounts.
    #[must_use]
    pub fn required_for_superadmin(&self) -> bool {
        self.config.require_for_superadmin
    }

    /// Whether client certificates are required for domain admin accounts.
    #[must_use]
    pub fn required_for_domain_admin(&self) -> bool {
        self.config.require_for_domain_admin
    }

    /// Extract certificate information from HTTP headers.
    ///
    /// Headers are passed as a map of header name (case-insensitive key) to value.
    /// The reverse proxy must set at least the verification and subject headers.
    ///
    /// # Errors
    ///
    /// Returns `MtlsError` if mTLS is disabled, headers are missing, verification
    /// failed, or the identity field cannot be extracted from the DN.
    pub fn extract(&self, headers: &HashMap<String, String>) -> Result<CertificateInfo, MtlsError> {
        if !self.config.enabled {
            return Err(MtlsError::Disabled);
        }

        // Check the verification header (e.g. X-SSL-Client-Verify: SUCCESS)
        let verify_value = headers
            .get(&self.config.trusted_proxy_header)
            .ok_or_else(|| {
                MtlsError::MissingVerificationHeader(self.config.trusted_proxy_header.clone())
            })?;

        if !verify_value.eq_ignore_ascii_case("SUCCESS") {
            return Err(MtlsError::VerificationFailed(verify_value.clone()));
        }

        // Extract the subject DN
        let subject_dn = headers
            .get(&self.config.subject_header)
            .ok_or_else(|| MtlsError::MissingSubjectHeader(self.config.subject_header.clone()))?;

        // Extract the identity from the DN
        let identity = parse_dn_field(subject_dn, &self.config.cn_field).ok_or_else(|| {
            MtlsError::FieldNotFound {
                field: self.config.cn_field.clone(),
                dn: subject_dn.clone(),
            }
        })?;

        // Extract optional serial number
        let serial = headers.get(&self.config.serial_header).cloned();

        Ok(CertificateInfo {
            identity,
            subject_dn: subject_dn.clone(),
            serial,
        })
    }
}

/// Parse a field value from a Distinguished Name string.
///
/// Supports both RFC 2253 comma-separated format (`emailAddress=user@example.com, CN=User`)
/// and OpenSSL slash-separated format (`/emailAddress=user@example.com/CN=User`).
///
/// Returns `None` if the field is not found.
#[must_use]
pub fn parse_dn_field(dn: &str, field: &str) -> Option<String> {
    // Try comma-separated (RFC 2253): "emailAddress=user@example.com, CN=User Name, O=Org"
    for part in dn.split(',') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            if key.trim().eq_ignore_ascii_case(field) {
                return Some(value.trim().to_string());
            }
        }
    }

    // Try slash-separated (OpenSSL): "/emailAddress=user@example.com/CN=User Name/O=Org"
    for part in dn.split('/') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((key, value)) = part.split_once('=') {
            if key.trim().eq_ignore_ascii_case(field) {
                return Some(value.trim().to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_config() -> MtlsConfig {
        MtlsConfig {
            enabled: true,
            ..MtlsConfig::default()
        }
    }

    fn make_headers(verify: &str, dn: &str, serial: Option<&str>) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("X-SSL-Client-Verify".to_string(), verify.to_string());
        headers.insert("X-SSL-Client-S-DN".to_string(), dn.to_string());
        if let Some(s) = serial {
            headers.insert("X-SSL-Client-Serial".to_string(), s.to_string());
        }
        headers
    }

    // --- MtlsVerifier tests ---

    #[test]
    fn extract_disabled_returns_error() {
        let verifier = MtlsVerifier::new(MtlsConfig::default());
        let headers = HashMap::new();
        let result = verifier.extract(&headers);
        assert!(matches!(result, Err(MtlsError::Disabled)));
    }

    #[test]
    fn extract_missing_verify_header_returns_error() {
        let verifier = MtlsVerifier::new(enabled_config());
        let headers = HashMap::new();
        let result = verifier.extract(&headers);
        assert!(matches!(
            result,
            Err(MtlsError::MissingVerificationHeader(_))
        ));
    }

    #[test]
    fn extract_verification_failed_returns_error() {
        let verifier = MtlsVerifier::new(enabled_config());
        let headers = make_headers("FAILED", "CN=test", None);
        let result = verifier.extract(&headers);
        assert!(matches!(result, Err(MtlsError::VerificationFailed(_))));
    }

    #[test]
    fn extract_missing_subject_header_returns_error() {
        let verifier = MtlsVerifier::new(enabled_config());
        let mut headers = HashMap::new();
        headers.insert("X-SSL-Client-Verify".to_string(), "SUCCESS".to_string());
        let result = verifier.extract(&headers);
        assert!(matches!(result, Err(MtlsError::MissingSubjectHeader(_))));
    }

    #[test]
    fn extract_field_not_found_returns_error() {
        let verifier = MtlsVerifier::new(enabled_config());
        let headers = make_headers("SUCCESS", "CN=Admin User, O=Example", None);
        let result = verifier.extract(&headers);
        assert!(matches!(result, Err(MtlsError::FieldNotFound { .. })));
    }

    #[test]
    fn extract_success_rfc2253_format() {
        let verifier = MtlsVerifier::new(enabled_config());
        let headers = make_headers(
            "SUCCESS",
            "emailAddress=admin@example.com, CN=Admin User, O=Example",
            Some("1234ABCD"),
        );
        let result = verifier.extract(&headers);
        assert!(result.is_ok());
        let info = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(info.identity, "admin@example.com");
        assert_eq!(
            info.subject_dn,
            "emailAddress=admin@example.com, CN=Admin User, O=Example"
        );
        assert_eq!(info.serial.as_deref(), Some("1234ABCD"));
    }

    #[test]
    fn extract_success_openssl_format() {
        let verifier = MtlsVerifier::new(enabled_config());
        let headers = make_headers(
            "SUCCESS",
            "/emailAddress=admin@example.com/CN=Admin User/O=Example",
            None,
        );
        let result = verifier.extract(&headers);
        assert!(result.is_ok());
        let info = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(info.identity, "admin@example.com");
        assert!(info.serial.is_none());
    }

    #[test]
    fn extract_success_case_insensitive_verify() {
        let verifier = MtlsVerifier::new(enabled_config());
        let headers = make_headers("success", "emailAddress=admin@example.com, CN=Test", None);
        let result = verifier.extract(&headers);
        assert!(result.is_ok());
    }

    #[test]
    fn is_enabled_reflects_config() {
        let disabled = MtlsVerifier::new(MtlsConfig::default());
        assert!(!disabled.is_enabled());

        let enabled = MtlsVerifier::new(enabled_config());
        assert!(enabled.is_enabled());
    }

    #[test]
    fn required_for_superadmin_reflects_config() {
        let verifier = MtlsVerifier::new(MtlsConfig {
            enabled: true,
            require_for_superadmin: true,
            ..MtlsConfig::default()
        });
        assert!(verifier.required_for_superadmin());
    }

    #[test]
    fn required_for_domain_admin_reflects_config() {
        let verifier = MtlsVerifier::new(MtlsConfig {
            enabled: true,
            require_for_domain_admin: true,
            ..MtlsConfig::default()
        });
        assert!(verifier.required_for_domain_admin());
    }

    // --- parse_dn_field tests ---

    #[test]
    fn parse_dn_field_rfc2253_comma_separated() {
        let dn = "emailAddress=user@example.com, CN=User Name, O=Example Inc";
        assert_eq!(
            parse_dn_field(dn, "emailAddress"),
            Some("user@example.com".to_string())
        );
        assert_eq!(parse_dn_field(dn, "CN"), Some("User Name".to_string()));
        assert_eq!(parse_dn_field(dn, "O"), Some("Example Inc".to_string()));
    }

    #[test]
    fn parse_dn_field_openssl_slash_separated() {
        let dn = "/emailAddress=user@example.com/CN=User Name/O=Example Inc";
        assert_eq!(
            parse_dn_field(dn, "emailAddress"),
            Some("user@example.com".to_string())
        );
        assert_eq!(parse_dn_field(dn, "CN"), Some("User Name".to_string()));
    }

    #[test]
    fn parse_dn_field_case_insensitive() {
        let dn = "EMAILADDRESS=user@example.com, cn=User";
        assert_eq!(
            parse_dn_field(dn, "emailAddress"),
            Some("user@example.com".to_string())
        );
        assert_eq!(parse_dn_field(dn, "CN"), Some("User".to_string()));
    }

    #[test]
    fn parse_dn_field_not_found() {
        let dn = "CN=User Name, O=Example";
        assert_eq!(parse_dn_field(dn, "emailAddress"), None);
    }

    #[test]
    fn parse_dn_field_empty_dn() {
        assert_eq!(parse_dn_field("", "CN"), None);
    }

    #[test]
    fn parse_dn_field_with_whitespace() {
        let dn = "  emailAddress = user@example.com ,  CN = User  ";
        assert_eq!(
            parse_dn_field(dn, "emailAddress"),
            Some("user@example.com".to_string())
        );
    }
}

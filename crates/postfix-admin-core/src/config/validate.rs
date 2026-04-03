use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::config::error::{ConfigError, ConfigWarning};
use crate::config::profile::OperatingMode;
use crate::config::sections::SecretString;
use crate::config::{AppConfig, LogLevel};

const VALID_PASSWORD_SCHEMES: &[&str] = &["argon2id", "bcrypt", "sha512-crypt", "sha256-crypt"];
const SECRET_KEY_BYTES: usize = 32;
const MIN_PASSWORD_MIN_LENGTH: usize = 4;

/// Validate an `AppConfig` according to the operating mode.
///
/// Returns a list of warnings for non-fatal issues, or an error if
/// validation fails.
///
/// # Errors
///
/// Returns `ConfigError` if any required field is missing or invalid.
pub fn validate(
    config: &mut AppConfig,
    mode: OperatingMode,
) -> Result<Vec<ConfigWarning>, ConfigError> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    validate_universal(config, &mut errors);

    let log_level: Result<LogLevel, ConfigError> = config.logging.level.parse();

    if mode.is_production_like() {
        validate_production(config, &log_level, &mut errors, &mut warnings);
    } else {
        autogenerate_dev_secrets(config, &mut errors, &mut warnings);
    }

    if let Err(e) = log_level {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(warnings)
    } else if errors.len() == 1 {
        Err(errors.remove(0))
    } else {
        Err(ConfigError::Multiple(errors))
    }
}

/// Universal validations that apply to all operating modes.
fn validate_universal(config: &AppConfig, errors: &mut Vec<ConfigError>) {
    if config.database.url.is_empty() {
        errors.push(ConfigError::validation("database.url", "must not be empty"));
    }

    if config.password_policy.min_length < MIN_PASSWORD_MIN_LENGTH {
        errors.push(ConfigError::validation(
            "password_policy.min_length",
            format!("must be at least {MIN_PASSWORD_MIN_LENGTH}"),
        ));
    }

    if !VALID_PASSWORD_SCHEMES.contains(&config.auth.password_scheme.as_str()) {
        errors.push(ConfigError::validation(
            "auth.password_scheme",
            format!(
                "unsupported scheme '{}', expected one of: {}",
                config.auth.password_scheme,
                VALID_PASSWORD_SCHEMES.join(", ")
            ),
        ));
    }

    if config.auth.mtls.enabled {
        if config.auth.mtls.trusted_proxy_header.is_empty() {
            errors.push(ConfigError::validation(
                "auth.mtls.trusted_proxy_header",
                "must not be empty when mTLS is enabled",
            ));
        }
        if config.auth.mtls.subject_header.is_empty() {
            errors.push(ConfigError::validation(
                "auth.mtls.subject_header",
                "must not be empty when mTLS is enabled",
            ));
        }
        if config.auth.mtls.cn_field.is_empty() {
            errors.push(ConfigError::validation(
                "auth.mtls.cn_field",
                "must not be empty when mTLS is enabled",
            ));
        }
    }

    if config.grpc.require_client_cert {
        if !config.grpc.tls_enabled {
            errors.push(ConfigError::validation(
                "grpc.require_client_cert",
                "requires grpc.tls_enabled = true",
            ));
        }
        if config.grpc.tls_ca_cert_path.is_empty() {
            errors.push(ConfigError::validation(
                "grpc.tls_ca_cert_path",
                "must not be empty when grpc.require_client_cert is true",
            ));
        }
    }
}

/// Production-like validations (Prep, Prod, Deployed).
fn validate_production(
    config: &AppConfig,
    log_level: &Result<LogLevel, ConfigError>,
    errors: &mut Vec<ConfigError>,
    warnings: &mut Vec<ConfigWarning>,
) {
    if config.auth.allow_cleartext {
        errors.push(ConfigError::validation(
            "auth.allow_cleartext",
            "must be false in production-like environments",
        ));
    }

    if config.server.secret_key.is_empty() {
        errors.push(ConfigError::validation(
            "server.secret_key",
            "must be set in production-like environments",
        ));
    }

    if config.encryption.master_key.is_empty() {
        errors.push(ConfigError::validation(
            "encryption.master_key",
            "must be set in production-like environments",
        ));
    }

    if !config.server.tls.enabled {
        warnings.push(ConfigWarning::new(
            "server.tls.enabled",
            "TLS is disabled in a production-like environment",
        ));
    }

    if !config.auth.mtls.enabled {
        warnings.push(ConfigWarning::new(
            "auth.mtls.enabled",
            "mTLS client certificates are recommended for admin accounts \
             in production environments",
        ));
    } else if !config.auth.mtls.require_for_superadmin {
        warnings.push(ConfigWarning::new(
            "auth.mtls.require_for_superadmin",
            "mTLS is enabled but not required for superadmin accounts",
        ));
    }

    if let Ok(level) = log_level {
        if level.is_verbose() {
            warnings.push(ConfigWarning::new(
                "logging.level",
                format!(
                    "verbose logging level '{}' in production-like environment",
                    config.logging.level
                ),
            ));
        }
    }
}

/// Auto-generate missing secrets in dev/test mode.
fn autogenerate_dev_secrets(
    config: &mut AppConfig,
    errors: &mut Vec<ConfigError>,
    warnings: &mut Vec<ConfigWarning>,
) {
    if config.server.secret_key.is_empty() {
        match generate_secret_base64() {
            Ok(key) => {
                config.server.secret_key = SecretString::new(key);
                warnings.push(ConfigWarning::info(
                    "server.secret_key",
                    "auto-generated (dev/test mode)",
                ));
            }
            Err(e) => errors.push(e),
        }
    }

    if config.encryption.master_key.is_empty() {
        match generate_secret_base64() {
            Ok(key) => {
                config.encryption.master_key = SecretString::new(key);
                warnings.push(ConfigWarning::info(
                    "encryption.master_key",
                    "auto-generated (dev/test mode)",
                ));
            }
            Err(e) => errors.push(e),
        }
    }
}

/// Generate a cryptographically random secret encoded as base64.
fn generate_secret_base64() -> Result<String, ConfigError> {
    let mut buf = [0u8; SECRET_KEY_BYTES];
    getrandom::fill(&mut buf).map_err(|e| {
        ConfigError::SecretGeneration(format!("failed to generate random bytes: {e}"))
    })?;
    Ok(BASE64.encode(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::profile::Profile;

    fn dev_mode() -> OperatingMode {
        OperatingMode::Development(Profile::Dev)
    }

    fn prod_mode() -> OperatingMode {
        OperatingMode::Development(Profile::Prod)
    }

    #[test]
    fn validate_default_dev_config_succeeds() {
        let mut config = AppConfig::default();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_ok());
    }

    #[test]
    fn validate_empty_database_url_fails() {
        let mut config = AppConfig::default();
        config.database.url = SecretString::default();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_min_password_length_too_low_fails() {
        let mut config = AppConfig::default();
        config.password_policy.min_length = 2;
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_invalid_password_scheme_fails() {
        let mut config = AppConfig::default();
        config.auth.password_scheme = "md5".to_string();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_dev_autogenerates_secrets() {
        let mut config = AppConfig::default();
        assert!(config.server.secret_key.is_empty());
        assert!(config.encryption.master_key.is_empty());

        let warnings = validate(&mut config, dev_mode());
        assert!(warnings.is_ok());
        assert!(!config.server.secret_key.is_empty());
        assert!(!config.encryption.master_key.is_empty());
    }

    #[test]
    fn validate_prod_empty_secret_key_fails() {
        let mut config = AppConfig::default();
        config.encryption.master_key = SecretString::new("dGVzdC1rZXktMTIzNDU2Nzg5MDEyMzQ1Ng==");
        let result = validate(&mut config, prod_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_prod_empty_master_key_fails() {
        let mut config = AppConfig::default();
        config.server.secret_key = SecretString::new("dGVzdC1rZXktMTIzNDU2Nzg5MDEyMzQ1Ng==");
        let result = validate(&mut config, prod_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_prod_cleartext_allowed_fails() {
        let mut config = AppConfig::default();
        config.server.secret_key = SecretString::new("key1");
        config.encryption.master_key = SecretString::new("key2");
        config.auth.allow_cleartext = true;
        let result = validate(&mut config, prod_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_prod_tls_disabled_warns() {
        let mut config = AppConfig::default();
        config.server.secret_key = SecretString::new("key1");
        config.encryption.master_key = SecretString::new("key2");
        config.server.tls.enabled = false;

        let warnings = validate(&mut config, prod_mode());
        assert!(warnings.is_ok());
        let warnings = warnings.unwrap_or_default();
        assert!(warnings.iter().any(|w| w.field == "server.tls.enabled"));
    }

    #[test]
    fn validate_prod_debug_logging_warns() {
        let mut config = AppConfig::default();
        config.server.secret_key = SecretString::new("key1");
        config.encryption.master_key = SecretString::new("key2");
        config.logging.level = "debug".to_string();

        let warnings = validate(&mut config, prod_mode());
        assert!(warnings.is_ok());
        let warnings = warnings.unwrap_or_default();
        assert!(warnings.iter().any(|w| w.field == "logging.level"));
    }

    #[test]
    fn validate_invalid_log_level_fails() {
        let mut config = AppConfig::default();
        config.logging.level = "verbose".to_string();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    // --- mTLS validation tests ---

    #[test]
    fn validate_mtls_enabled_empty_headers_fails() {
        let mut config = AppConfig::default();
        config.auth.mtls.enabled = true;
        config.auth.mtls.trusted_proxy_header = String::new();
        config.auth.mtls.subject_header = String::new();
        config.auth.mtls.cn_field = String::new();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_mtls_enabled_empty_subject_header_fails() {
        let mut config = AppConfig::default();
        config.auth.mtls.enabled = true;
        config.auth.mtls.subject_header = String::new();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_mtls_enabled_empty_cn_field_fails() {
        let mut config = AppConfig::default();
        config.auth.mtls.enabled = true;
        config.auth.mtls.cn_field = String::new();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_mtls_enabled_with_defaults_succeeds() {
        let mut config = AppConfig::default();
        config.auth.mtls.enabled = true;
        let result = validate(&mut config, dev_mode());
        assert!(result.is_ok());
    }

    #[test]
    fn validate_prod_mtls_disabled_warns() {
        let mut config = AppConfig::default();
        config.server.secret_key = SecretString::new("key1");
        config.encryption.master_key = SecretString::new("key2");
        config.auth.mtls.enabled = false;

        let warnings = validate(&mut config, prod_mode()).unwrap_or_default();
        assert!(warnings.iter().any(|w| w.field == "auth.mtls.enabled"));
    }

    #[test]
    fn validate_prod_mtls_enabled_superadmin_not_required_warns() {
        let mut config = AppConfig::default();
        config.server.secret_key = SecretString::new("key1");
        config.encryption.master_key = SecretString::new("key2");
        config.auth.mtls.enabled = true;
        config.auth.mtls.require_for_superadmin = false;

        let warnings = validate(&mut config, prod_mode()).unwrap_or_default();
        assert!(warnings
            .iter()
            .any(|w| w.field == "auth.mtls.require_for_superadmin"));
    }

    #[test]
    fn validate_grpc_client_cert_without_tls_fails() {
        let mut config = AppConfig::default();
        config.grpc.require_client_cert = true;
        config.grpc.tls_enabled = false;
        config.grpc.tls_ca_cert_path = "/path/to/ca.pem".to_string();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn validate_grpc_client_cert_without_ca_path_fails() {
        let mut config = AppConfig::default();
        config.grpc.require_client_cert = true;
        config.grpc.tls_enabled = true;
        config.grpc.tls_ca_cert_path = String::new();
        let result = validate(&mut config, dev_mode());
        assert!(result.is_err());
    }

    #[test]
    fn generate_secret_base64_produces_valid_base64() {
        let secret = generate_secret_base64();
        assert!(secret.is_ok());
        let decoded = BASE64.decode(secret.unwrap_or_default());
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap_or_default().len(), SECRET_KEY_BYTES);
    }
}

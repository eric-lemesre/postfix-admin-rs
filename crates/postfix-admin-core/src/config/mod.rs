//! Configuration system for postfix-admin-rs.
//!
//! Loads configuration from layered TOML files, environment variables, and
//! CLI arguments. Validates settings contextually based on the operating
//! mode (development vs deployed) and profile (dev, test, prep, prod).

pub mod error;
pub mod loader;
pub mod profile;
pub mod sections;
pub mod validate;

use serde::Deserialize;

pub use error::{ConfigError, ConfigWarning};
pub use loader::{CliOverrides, ConfigLoader};
pub use profile::{OperatingMode, Profile};
pub use sections::{
    Argon2Config, AuthConfig, DatabaseConfig, DkimConfig, DomainDefaultsConfig, EncryptionConfig,
    FetchmailConfig, GrpcConfig, JwtConfig, LogFormat, LogLevel, LoggingConfig, MailConfig,
    PasswordPolicyConfig, SecretString, SecurityConfig, ServerConfig, TlsConfig, UiConfig,
    VacationConfig,
};

/// Root application configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub grpc: GrpcConfig,
    pub auth: AuthConfig,
    pub password_policy: PasswordPolicyConfig,
    pub mail: MailConfig,
    pub vacation: VacationConfig,
    pub fetchmail: FetchmailConfig,
    pub dkim: DkimConfig,
    pub logging: LoggingConfig,
    pub ui: UiConfig,
    pub domain_defaults: DomainDefaultsConfig,
    pub security: SecurityConfig,
    pub encryption: EncryptionConfig,
}

impl AppConfig {
    /// Load the application configuration.
    ///
    /// Detects the operating mode, loads files from the appropriate directories,
    /// applies environment variable and CLI overrides, then validates the result.
    ///
    /// Returns the validated config along with any non-fatal warnings.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if loading or validation fails.
    pub fn load(cli_overrides: &CliOverrides) -> Result<(Self, Vec<ConfigWarning>), ConfigError> {
        let profile = loader::resolve_profile(cli_overrides.profile)?;
        let mode = OperatingMode::detect(profile);

        tracing::info!(
            profile = %mode.profile(),
            mode = ?mode,
            "loading configuration"
        );

        let mut config = ConfigLoader::load(mode, cli_overrides)?;
        let warnings = validate::validate(&mut config, mode)?;

        for warning in &warnings {
            tracing::warn!("{warning}");
        }

        tracing::debug!(?config, "configuration loaded");

        Ok((config, warnings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_config_default_has_sane_values() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.bind_address, "127.0.0.1");
        assert!(!config.database.url.is_empty());
        assert_eq!(config.auth.password_scheme, "argon2id");
    }

    #[test]
    fn app_config_load_with_defaults_succeeds() {
        // Use a temp dir so no real config files interfere
        let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
        let cli = CliOverrides {
            config_path: Some(tmp.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let result = AppConfig::load(&cli);
        assert!(result.is_ok());
    }
}

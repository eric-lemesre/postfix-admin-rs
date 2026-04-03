use std::path::Path;

use config::{Config, Environment, File};

use crate::config::error::ConfigError;
use crate::config::profile::{OperatingMode, Profile, DEPLOYED_CONFIG_DIR, DEV_CONFIG_DIR};
use crate::config::AppConfig;

/// CLI argument overrides for configuration values.
#[derive(Debug, Default)]
pub struct CliOverrides {
    pub config_path: Option<String>,
    pub profile: Option<Profile>,
    pub database_url: Option<String>,
    pub bind_address: Option<String>,
    pub port: Option<u16>,
    pub log_level: Option<String>,
    pub workers: Option<usize>,
}

/// Loads configuration from layered sources according to the operating mode.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration with the given mode and CLI overrides.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if files cannot be parsed or deserialization fails.
    pub fn load(mode: OperatingMode, cli: &CliOverrides) -> Result<AppConfig, ConfigError> {
        let mut builder = Config::builder();

        // Layer 1: compiled defaults (via serde defaults on AppConfig)
        // Handled by deserializing into AppConfig which has Default on all fields.

        match mode {
            OperatingMode::Development(profile) => {
                builder = Self::add_dev_sources(builder, profile, cli);
            }
            OperatingMode::Deployed => {
                builder = Self::add_deployed_sources(builder, cli);
            }
        }

        // Environment variables: PAR__SECTION__KEY
        builder = builder.add_source(
            Environment::with_prefix("PAR")
                .separator("__")
                .try_parsing(true),
        );

        // CLI overrides (highest priority)
        builder = Self::apply_cli_overrides(builder, cli)?;

        let config = builder.build()?;
        let app_config: AppConfig = config.try_deserialize()?;

        Ok(app_config)
    }

    fn add_dev_sources(
        mut builder: config::ConfigBuilder<config::builder::DefaultState>,
        profile: Profile,
        cli: &CliOverrides,
    ) -> config::ConfigBuilder<config::builder::DefaultState> {
        let config_dir = cli.config_path.as_deref().unwrap_or(DEV_CONFIG_DIR);

        // config/default.toml
        let default_path = format!("{config_dir}/default.toml");
        if Path::new(&default_path).is_file() {
            builder = builder.add_source(File::with_name(&default_path));
        }

        // config/{profile}.toml
        let profile_path = format!("{config_dir}/{}.toml", profile.as_str());
        if Path::new(&profile_path).is_file() {
            builder = builder.add_source(File::with_name(&profile_path));
        }

        // config/local.toml (personal overrides)
        let local_path = format!("{config_dir}/local.toml");
        if Path::new(&local_path).is_file() {
            builder = builder.add_source(File::with_name(&local_path));
        }

        // config/secrets.toml
        let secrets_path = format!("{config_dir}/secrets.toml");
        if Path::new(&secrets_path).is_file() {
            builder = builder.add_source(File::with_name(&secrets_path));
        }

        // .env file (dev/test only)
        if matches!(profile, Profile::Dev | Profile::Test) {
            let _ = dotenvy::dotenv();
        }

        builder
    }

    fn add_deployed_sources(
        mut builder: config::ConfigBuilder<config::builder::DefaultState>,
        cli: &CliOverrides,
    ) -> config::ConfigBuilder<config::builder::DefaultState> {
        let config_dir = cli.config_path.as_deref().unwrap_or(DEPLOYED_CONFIG_DIR);

        // /etc/postfix-admin-rs/config.toml
        let config_path = format!("{config_dir}/config.toml");
        if Path::new(&config_path).is_file() {
            builder = builder.add_source(File::with_name(&config_path));
        }

        // /etc/postfix-admin-rs/config.local.toml
        let local_path = format!("{config_dir}/config.local.toml");
        if Path::new(&local_path).is_file() {
            builder = builder.add_source(File::with_name(&local_path));
        }

        // /etc/postfix-admin-rs/secrets.toml
        let secrets_path = format!("{config_dir}/secrets.toml");
        if Path::new(&secrets_path).is_file() {
            builder = builder.add_source(File::with_name(&secrets_path));
        }

        builder
    }

    fn apply_cli_overrides(
        mut builder: config::ConfigBuilder<config::builder::DefaultState>,
        cli: &CliOverrides,
    ) -> Result<config::ConfigBuilder<config::builder::DefaultState>, ConfigError> {
        if let Some(ref url) = cli.database_url {
            builder = builder.set_override("database.url", url.as_str())?;
        }
        if let Some(ref addr) = cli.bind_address {
            builder = builder.set_override("server.bind_address", addr.as_str())?;
        }
        if let Some(port) = cli.port {
            builder = builder.set_override("server.port", i64::from(port))?;
        }
        if let Some(ref level) = cli.log_level {
            builder = builder.set_override("logging.level", level.as_str())?;
        }
        if let Some(workers) = cli.workers {
            builder =
                builder.set_override("server.workers", i64::try_from(workers).unwrap_or(0))?;
        }
        Ok(builder)
    }
}

/// Resolve the profile from CLI args, environment, or default.
///
/// Priority: CLI `--profile` > `PAR_PROFILE` env var > default (`Dev`).
///
/// # Errors
///
/// Returns `ConfigError` if the profile string is not a valid profile name.
pub fn resolve_profile(cli_profile: Option<Profile>) -> Result<Profile, ConfigError> {
    if let Some(profile) = cli_profile {
        return Ok(profile);
    }

    if let Ok(env_profile) = std::env::var("PAR_PROFILE") {
        return env_profile.parse();
    }

    Ok(Profile::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_overrides_default_all_none() {
        let cli = CliOverrides::default();
        assert!(cli.config_path.is_none());
        assert!(cli.profile.is_none());
        assert!(cli.database_url.is_none());
        assert!(cli.bind_address.is_none());
        assert!(cli.port.is_none());
        assert!(cli.log_level.is_none());
        assert!(cli.workers.is_none());
    }

    #[test]
    fn resolve_profile_cli_takes_priority() {
        let result = resolve_profile(Some(Profile::Prod));
        assert_eq!(result.ok(), Some(Profile::Prod));
    }

    #[test]
    fn resolve_profile_default_is_dev() {
        // Clear env to ensure no PAR_PROFILE interference
        std::env::remove_var("PAR_PROFILE");
        let result = resolve_profile(None);
        assert_eq!(result.ok(), Some(Profile::Dev));
    }

    #[test]
    fn load_with_empty_dir_returns_defaults() {
        let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
        let cli = CliOverrides {
            config_path: Some(tmp.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let mode = OperatingMode::Development(Profile::Dev);
        let config = ConfigLoader::load(mode, &cli);
        assert!(config.is_ok());
        let config = config.unwrap_or_else(|_| unreachable!());
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.bind_address, "127.0.0.1");
    }

    #[test]
    fn load_cli_overrides_applied() {
        let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
        let cli = CliOverrides {
            config_path: Some(tmp.path().to_string_lossy().to_string()),
            port: Some(9090),
            bind_address: Some("0.0.0.0".to_string()),
            log_level: Some("debug".to_string()),
            ..Default::default()
        };
        let mode = OperatingMode::Development(Profile::Dev);
        let config = ConfigLoader::load(mode, &cli);
        assert!(config.is_ok());
        let config = config.unwrap_or_else(|_| unreachable!());
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.bind_address, "0.0.0.0");
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn load_from_toml_file() {
        let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
        let default_toml = tmp.path().join("default.toml");
        std::fs::write(
            &default_toml,
            r#"
[server]
port = 3000
bind_address = "10.0.0.1"

[logging]
level = "warn"
"#,
        )
        .unwrap_or_else(|_| unreachable!());

        let cli = CliOverrides {
            config_path: Some(tmp.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let mode = OperatingMode::Development(Profile::Dev);
        let config = ConfigLoader::load(mode, &cli);
        assert!(config.is_ok());
        let config = config.unwrap_or_else(|_| unreachable!());
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.bind_address, "10.0.0.1");
        assert_eq!(config.logging.level, "warn");
    }

    #[test]
    fn load_profile_overrides_default() {
        let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

        std::fs::write(
            tmp.path().join("default.toml"),
            r"
[server]
port = 3000
",
        )
        .unwrap_or_else(|_| unreachable!());

        std::fs::write(
            tmp.path().join("test.toml"),
            r"
[server]
port = 0
",
        )
        .unwrap_or_else(|_| unreachable!());

        let cli = CliOverrides {
            config_path: Some(tmp.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let mode = OperatingMode::Development(Profile::Test);
        let config = ConfigLoader::load(mode, &cli);
        assert!(config.is_ok());
        let config = config.unwrap_or_else(|_| unreachable!());
        assert_eq!(config.server.port, 0);
    }

    #[test]
    fn secret_string_deserialized_from_toml() {
        let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
        std::fs::write(
            tmp.path().join("default.toml"),
            r#"
[server]
secret_key = "my-secret-key"
"#,
        )
        .unwrap_or_else(|_| unreachable!());

        let cli = CliOverrides {
            config_path: Some(tmp.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let mode = OperatingMode::Development(Profile::Dev);
        let config = ConfigLoader::load(mode, &cli).unwrap_or_else(|_| unreachable!());
        assert_eq!(config.server.secret_key.expose(), "my-secret-key");
    }
}

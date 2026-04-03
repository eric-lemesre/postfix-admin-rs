//! Integration tests for the configuration loading system.

use postfix_admin_core::config::{
    AppConfig, CliOverrides, ConfigLoader, OperatingMode, Profile, SecretString,
};

/// Helper to create a temp dir with config files and return a `CliOverrides`
/// pointing to it.
fn cli_for_temp(dir: &tempfile::TempDir) -> CliOverrides {
    CliOverrides {
        config_path: Some(dir.path().to_string_lossy().to_string()),
        ..Default::default()
    }
}

#[test]
fn load_full_default_config_round_trip() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r#"
[server]
bind_address = "10.0.0.1"
port = 9000
workers = 4
base_url = "https://mail.test.com"

[server.tls]
enabled = true
cert_path = "/tmp/cert.pem"
key_path = "/tmp/key.pem"

[database]
url = "postgresql://test:test@localhost:5432/testdb"
max_connections = 20
min_connections = 5

[auth]
password_scheme = "bcrypt"
allow_cleartext = false

[auth.argon2]
memory_cost = 8192
time_cost = 3
parallelism = 2

[logging]
level = "warn"
format = "json"

[ui]
page_size = 50
site_name = "Test Admin"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    let cli = cli_for_temp(&tmp);
    let mode = OperatingMode::Development(Profile::Dev);
    let config = ConfigLoader::load(mode, &cli).unwrap_or_else(|_| unreachable!());

    assert_eq!(config.server.bind_address, "10.0.0.1");
    assert_eq!(config.server.port, 9000);
    assert_eq!(config.server.workers, 4);
    assert!(config.server.tls.enabled);
    assert_eq!(config.database.max_connections, 20);
    assert_eq!(config.auth.password_scheme, "bcrypt");
    assert_eq!(config.auth.argon2.memory_cost, 8192);
    assert_eq!(config.logging.level, "warn");
    assert_eq!(config.logging.format, "json");
    assert_eq!(config.ui.page_size, 50);
    assert_eq!(config.ui.site_name, "Test Admin");
}

#[test]
fn load_profile_layers_correctly() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r#"
[server]
port = 8080
bind_address = "127.0.0.1"

[logging]
level = "info"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("test.toml"),
        r#"
[server]
port = 0

[logging]
level = "debug"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    // With Dev profile, should get default.toml values
    let cli = cli_for_temp(&tmp);
    let config = ConfigLoader::load(OperatingMode::Development(Profile::Dev), &cli)
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.logging.level, "info");

    // With Test profile, should get test.toml overrides
    let config = ConfigLoader::load(OperatingMode::Development(Profile::Test), &cli)
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(config.server.port, 0);
    assert_eq!(config.logging.level, "debug");
    // bind_address should come from default.toml (not overridden in test.toml)
    assert_eq!(config.server.bind_address, "127.0.0.1");
}

#[test]
fn load_local_overrides_profile() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r"
[server]
port = 8080
",
    )
    .unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("dev.toml"),
        r"
[server]
port = 3000
",
    )
    .unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("local.toml"),
        r"
[server]
port = 4000
",
    )
    .unwrap_or_else(|_| unreachable!());

    let cli = cli_for_temp(&tmp);
    let config = ConfigLoader::load(OperatingMode::Development(Profile::Dev), &cli)
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(config.server.port, 4000);
}

#[test]
fn load_secrets_file_overrides_local() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r#"
[server]
secret_key = ""
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("secrets.toml"),
        r#"
[server]
secret_key = "from-secrets-file"

[encryption]
master_key = "encryption-key"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    let cli = cli_for_temp(&tmp);
    let config = ConfigLoader::load(OperatingMode::Development(Profile::Dev), &cli)
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(config.server.secret_key.expose(), "from-secrets-file");
    assert_eq!(config.encryption.master_key.expose(), "encryption-key");
}

#[test]
fn load_cli_overrides_take_highest_priority() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r#"
[server]
port = 8080
bind_address = "127.0.0.1"

[logging]
level = "info"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    let cli = CliOverrides {
        config_path: Some(tmp.path().to_string_lossy().to_string()),
        port: Some(9999),
        bind_address: Some("0.0.0.0".to_string()),
        log_level: Some("error".to_string()),
        ..Default::default()
    };

    let config = ConfigLoader::load(OperatingMode::Development(Profile::Dev), &cli)
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(config.server.port, 9999);
    assert_eq!(config.server.bind_address, "0.0.0.0");
    assert_eq!(config.logging.level, "error");
}

#[test]
fn app_config_load_validates_dev_mode() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    let cli = CliOverrides {
        config_path: Some(tmp.path().to_string_lossy().to_string()),
        ..Default::default()
    };

    let result = AppConfig::load(&cli);
    assert!(result.is_ok());

    let (config, warnings) = result.unwrap_or_else(|_| unreachable!());
    // Dev mode auto-generates secrets
    assert!(!config.server.secret_key.is_empty());
    assert!(!config.encryption.master_key.is_empty());
    // Should have info warnings about auto-generated keys
    assert!(
        warnings.len() >= 2,
        "expected at least 2 warnings, got {}",
        warnings.len()
    );
}

#[test]
fn app_config_load_prod_mode_rejects_empty_secrets() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r"
[auth]
allow_cleartext = false
",
    )
    .unwrap_or_else(|_| unreachable!());

    // Force prod profile via CLI
    let cli = CliOverrides {
        config_path: Some(tmp.path().to_string_lossy().to_string()),
        profile: Some(Profile::Prod),
        ..Default::default()
    };

    let result = AppConfig::load(&cli);
    // Should fail because secret_key and master_key are empty in prod mode
    assert!(result.is_err());
}

#[test]
fn app_config_load_prod_with_secrets_succeeds() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r#"
[server]
secret_key = "prod-secret-key-base64"

[auth]
allow_cleartext = false

[encryption]
master_key = "prod-master-key-base64"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    let cli = CliOverrides {
        config_path: Some(tmp.path().to_string_lossy().to_string()),
        profile: Some(Profile::Prod),
        ..Default::default()
    };

    let result = AppConfig::load(&cli);
    assert!(result.is_ok());
}

#[test]
fn deployed_mode_loads_from_config_toml() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    // In deployed mode, the file is config.toml (not default.toml)
    std::fs::write(
        tmp.path().join("config.toml"),
        r#"
[server]
port = 443
bind_address = "0.0.0.0"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    let cli = CliOverrides {
        config_path: Some(tmp.path().to_string_lossy().to_string()),
        ..Default::default()
    };

    let config =
        ConfigLoader::load(OperatingMode::Deployed, &cli).unwrap_or_else(|_| unreachable!());
    assert_eq!(config.server.port, 443);
    assert_eq!(config.server.bind_address, "0.0.0.0");
}

#[test]
fn empty_config_dir_produces_defaults() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
    let cli = cli_for_temp(&tmp);

    let config = ConfigLoader::load(OperatingMode::Development(Profile::Dev), &cli)
        .unwrap_or_else(|_| unreachable!());

    // Should have all defaults
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.bind_address, "127.0.0.1");
    assert_eq!(config.auth.password_scheme, "argon2id");
    assert_eq!(config.logging.level, "info");
    assert!(config.server.secret_key.is_empty());
}

#[test]
fn validation_rejects_invalid_password_scheme_through_load() {
    let tmp = tempfile::tempdir().unwrap_or_else(|_| unreachable!());

    std::fs::write(
        tmp.path().join("default.toml"),
        r#"
[auth]
password_scheme = "md5"
"#,
    )
    .unwrap_or_else(|_| unreachable!());

    let cli = CliOverrides {
        config_path: Some(tmp.path().to_string_lossy().to_string()),
        ..Default::default()
    };

    let result = AppConfig::load(&cli);
    assert!(result.is_err());
}

#[test]
fn secret_string_masked_in_debug_output() {
    let secret = SecretString::new("super-secret-value");
    let debug = format!("{secret:?}");
    assert!(!debug.contains("super-secret"));
    assert!(debug.contains("***"));
}

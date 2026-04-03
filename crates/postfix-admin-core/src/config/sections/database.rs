use serde::Deserialize;

use super::server::SecretString;

/// Database connection configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: SecretString,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub table_prefix: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: SecretString::new(
                "postgresql://postfix_admin:password@localhost:5432/postfix_admin",
            ),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_seconds: 5,
            idle_timeout_seconds: 300,
            table_prefix: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_config_default_has_valid_url() {
        let cfg = DatabaseConfig::default();
        assert!(!cfg.url.is_empty());
        assert!(cfg.url.expose().contains("postfix_admin"));
    }

    #[test]
    fn database_config_default_pool_settings() {
        let cfg = DatabaseConfig::default();
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.min_connections, 2);
    }
}

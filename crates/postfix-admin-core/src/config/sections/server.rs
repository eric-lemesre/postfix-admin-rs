use std::fmt;

use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub workers: usize,
    pub base_url: String,
    pub secret_key: SecretString,
    pub tls: TlsConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            workers: 0,
            base_url: String::new(),
            secret_key: SecretString::default(),
            tls: TlsConfig::default(),
        }
    }
}

/// TLS configuration for the HTTP server.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

/// A secret string that is zeroized on drop and masked in debug output.
///
/// Does not implement `Serialize` to prevent accidental leakage.
#[derive(Clone, Default, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(from = "String")]
pub struct SecretString(String);

impl SecretString {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            f.write_str("SecretString(<empty>)")
        } else {
            f.write_str("SecretString(***)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_default_binds_to_localhost() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.bind_address, "127.0.0.1");
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn tls_config_default_disabled() {
        let cfg = TlsConfig::default();
        assert!(!cfg.enabled);
    }

    #[test]
    fn secret_string_debug_masks_value() {
        let secret = SecretString::new("super-secret");
        assert_eq!(format!("{secret:?}"), "SecretString(***)");
    }

    #[test]
    fn secret_string_debug_shows_empty() {
        let secret = SecretString::default();
        assert_eq!(format!("{secret:?}"), "SecretString(<empty>)");
    }

    #[test]
    fn secret_string_expose_returns_inner_value() {
        let secret = SecretString::new("my-value");
        assert_eq!(secret.expose(), "my-value");
    }

    #[test]
    fn secret_string_is_empty_works() {
        assert!(SecretString::default().is_empty());
        assert!(!SecretString::new("x").is_empty());
    }
}

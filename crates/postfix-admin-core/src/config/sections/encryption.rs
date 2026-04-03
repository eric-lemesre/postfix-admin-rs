use serde::Deserialize;

use super::server::SecretString;

/// Encryption configuration for data at rest (DKIM keys, fetchmail passwords).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct EncryptionConfig {
    pub master_key: SecretString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_config_default_empty_key() {
        let cfg = EncryptionConfig::default();
        assert!(cfg.master_key.is_empty());
    }
}

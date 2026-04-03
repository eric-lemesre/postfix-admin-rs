use serde::Deserialize;

/// DKIM signing configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DkimConfig {
    pub enabled: bool,
    pub default_key_size: u32,
}

impl Default for DkimConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_key_size: 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dkim_config_default_enabled_2048() {
        let cfg = DkimConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.default_key_size, 2048);
    }
}

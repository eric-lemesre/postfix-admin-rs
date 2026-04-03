use serde::Deserialize;

/// Fetchmail integration configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FetchmailConfig {
    pub enabled: bool,
    pub min_poll_interval: u32,
}

impl Default for FetchmailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_poll_interval: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetchmail_config_default_enabled() {
        let cfg = FetchmailConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.min_poll_interval, 5);
    }
}

use serde::Deserialize;

/// Vacation auto-responder configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct VacationConfig {
    pub enabled: bool,
    pub domain: String,
}

impl Default for VacationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            domain: "autoreply.example.com".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vacation_config_default_enabled() {
        let cfg = VacationConfig::default();
        assert!(cfg.enabled);
    }
}

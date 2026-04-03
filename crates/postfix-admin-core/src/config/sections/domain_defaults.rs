use serde::Deserialize;

/// Default values applied when creating new domains.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DomainDefaultsConfig {
    pub aliases: u64,
    pub mailboxes: u64,
    pub maxquota: u64,
    pub quota: u64,
    pub transport: String,
    pub backupmx: bool,
}

impl Default for DomainDefaultsConfig {
    fn default() -> Self {
        Self {
            aliases: 0,
            mailboxes: 0,
            maxquota: 0,
            quota: 0,
            transport: "virtual:".to_string(),
            backupmx: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_defaults_config_unlimited_by_default() {
        let cfg = DomainDefaultsConfig::default();
        assert_eq!(cfg.aliases, 0);
        assert_eq!(cfg.mailboxes, 0);
        assert_eq!(cfg.transport, "virtual:");
    }
}

use serde::Deserialize;

/// Security headers and policy configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct SecurityConfig {
    pub dns_check_enabled: bool,
    pub local_alias_only: bool,
    pub csp_enabled: bool,
    pub hsts_enabled: bool,
    pub hsts_max_age: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            dns_check_enabled: true,
            local_alias_only: false,
            csp_enabled: true,
            hsts_enabled: true,
            hsts_max_age: 31_536_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_config_default_enables_protections() {
        let cfg = SecurityConfig::default();
        assert!(cfg.csp_enabled);
        assert!(cfg.hsts_enabled);
        assert!(cfg.dns_check_enabled);
    }
}

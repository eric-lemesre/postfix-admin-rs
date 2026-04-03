use serde::Deserialize;

/// Authentication configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    pub session_lifetime: u64,
    pub max_login_attempts: u32,
    pub lockout_duration: u64,
    pub password_scheme: String,
    pub allow_cleartext: bool,
    pub argon2: Argon2Config,
    pub jwt: JwtConfig,
    pub mtls: MtlsConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_lifetime: 3600,
            max_login_attempts: 5,
            lockout_duration: 900,
            password_scheme: "argon2id".to_string(),
            allow_cleartext: false,
            argon2: Argon2Config::default(),
            jwt: JwtConfig::default(),
            mtls: MtlsConfig::default(),
        }
    }
}

/// Argon2id hashing parameters (OWASP 2024 recommendations).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Argon2Config {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_cost: 19456,
            time_cost: 2,
            parallelism: 1,
        }
    }
}

/// JWT token configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct JwtConfig {
    pub access_token_lifetime: u64,
    pub refresh_token_lifetime: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            access_token_lifetime: 900,
            refresh_token_lifetime: 604_800,
        }
    }
}

/// Mutual TLS (client certificate) configuration for administrator authentication.
///
/// When enabled, the reverse proxy (Nginx/Apache) verifies client certificates
/// and forwards identity information via HTTP headers. This provides a strong
/// additional MFA factor for privileged accounts.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MtlsConfig {
    pub enabled: bool,
    pub trusted_proxy_header: String,
    pub subject_header: String,
    pub serial_header: String,
    pub require_for_superadmin: bool,
    pub require_for_domain_admin: bool,
    pub cn_field: String,
}

impl Default for MtlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trusted_proxy_header: "X-SSL-Client-Verify".to_string(),
            subject_header: "X-SSL-Client-S-DN".to_string(),
            serial_header: "X-SSL-Client-Serial".to_string(),
            require_for_superadmin: false,
            require_for_domain_admin: false,
            cn_field: "emailAddress".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_config_default_uses_argon2id() {
        let cfg = AuthConfig::default();
        assert_eq!(cfg.password_scheme, "argon2id");
        assert!(!cfg.allow_cleartext);
    }

    #[test]
    fn argon2_config_default_owasp_2024() {
        let cfg = Argon2Config::default();
        assert_eq!(cfg.memory_cost, 19456);
        assert_eq!(cfg.time_cost, 2);
        assert_eq!(cfg.parallelism, 1);
    }

    #[test]
    fn jwt_config_default_lifetimes() {
        let cfg = JwtConfig::default();
        assert_eq!(cfg.access_token_lifetime, 900);
        assert_eq!(cfg.refresh_token_lifetime, 604_800);
    }

    #[test]
    fn mtls_config_default_disabled() {
        let cfg = MtlsConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.trusted_proxy_header, "X-SSL-Client-Verify");
        assert_eq!(cfg.subject_header, "X-SSL-Client-S-DN");
        assert_eq!(cfg.serial_header, "X-SSL-Client-Serial");
        assert!(!cfg.require_for_superadmin);
        assert!(!cfg.require_for_domain_admin);
        assert_eq!(cfg.cn_field, "emailAddress");
    }

    #[test]
    fn auth_config_default_includes_mtls() {
        let cfg = AuthConfig::default();
        assert!(!cfg.mtls.enabled);
    }
}

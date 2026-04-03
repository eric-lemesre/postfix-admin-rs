use serde::Deserialize;

use super::server::SecretString;

/// SMTP mail notification configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_tls: bool,
    pub smtp_username: String,
    pub smtp_password: SecretString,
    pub from_address: String,
    pub from_name: String,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 25,
            smtp_tls: false,
            smtp_username: String::new(),
            smtp_password: SecretString::default(),
            from_address: "postmaster@example.com".to_string(),
            from_name: "PostfixAdmin".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mail_config_default_local_smtp() {
        let cfg = MailConfig::default();
        assert_eq!(cfg.smtp_host, "localhost");
        assert_eq!(cfg.smtp_port, 25);
        assert!(!cfg.smtp_tls);
    }
}

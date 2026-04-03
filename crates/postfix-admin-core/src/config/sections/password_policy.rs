use serde::Deserialize;

/// Password policy configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PasswordPolicyConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
}

impl Default for PasswordPolicyConfig {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 256,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_policy_default_has_sane_values() {
        let cfg = PasswordPolicyConfig::default();
        assert_eq!(cfg.min_length, 8);
        assert_eq!(cfg.max_length, 256);
        assert!(cfg.require_uppercase);
        assert!(!cfg.require_special);
    }
}

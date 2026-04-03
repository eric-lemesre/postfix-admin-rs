use std::fmt;

use thiserror::Error;

/// Severity level for configuration warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningSeverity {
    /// Informational message, not an issue.
    Info,
    /// Potential issue that should be reviewed.
    Warning,
}

/// A non-fatal configuration warning.
#[derive(Debug, Clone)]
pub struct ConfigWarning {
    pub field: &'static str,
    pub message: String,
    pub severity: WarningSeverity,
}

impl fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.severity {
            WarningSeverity::Info => "INFO",
            WarningSeverity::Warning => "WARN",
        };
        write!(f, "[{level}] {}: {}", self.field, self.message)
    }
}

/// Errors that can occur during configuration loading and validation.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A required configuration file was not found.
    #[error("configuration file not found: {path}")]
    FileNotFound { path: String },

    /// A configuration file could not be parsed.
    #[error("configuration parse error: {0}")]
    Parse(String),

    /// A single field failed validation.
    #[error("invalid configuration for '{field}': {reason}")]
    Validation { field: &'static str, reason: String },

    /// Multiple validation errors occurred.
    #[error("multiple configuration errors: {}", format_multiple(.0))]
    Multiple(Vec<ConfigError>),

    /// Failed to generate a secret value.
    #[error("secret generation error: {0}")]
    SecretGeneration(String),

    /// Any other configuration error.
    #[error("configuration error: {0}")]
    Other(String),
}

fn format_multiple(errors: &[ConfigError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

impl From<config::ConfigError> for ConfigError {
    fn from(err: config::ConfigError) -> Self {
        Self::Parse(err.to_string())
    }
}

impl ConfigError {
    #[must_use]
    pub fn validation(field: &'static str, reason: impl Into<String>) -> Self {
        Self::Validation {
            field,
            reason: reason.into(),
        }
    }
}

impl ConfigWarning {
    #[must_use]
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
            severity: WarningSeverity::Warning,
        }
    }

    #[must_use]
    pub fn info(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
            severity: WarningSeverity::Info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_display_shows_field_and_reason() {
        let err = ConfigError::validation("database.url", "must not be empty");
        assert_eq!(
            err.to_string(),
            "invalid configuration for 'database.url': must not be empty"
        );
    }

    #[test]
    fn file_not_found_display_shows_path() {
        let err = ConfigError::FileNotFound {
            path: "/etc/config.toml".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "configuration file not found: /etc/config.toml"
        );
    }

    #[test]
    fn multiple_errors_display_joins_messages() {
        let err = ConfigError::Multiple(vec![
            ConfigError::validation("a", "bad"),
            ConfigError::validation("b", "worse"),
        ]);
        let msg = err.to_string();
        assert!(msg.contains("bad"));
        assert!(msg.contains("worse"));
    }

    #[test]
    fn warning_display_shows_severity_and_field() {
        let warn = ConfigWarning::new("server.tls.enabled", "TLS is disabled");
        assert_eq!(
            warn.to_string(),
            "[WARN] server.tls.enabled: TLS is disabled"
        );
    }

    #[test]
    fn info_warning_display_shows_info_level() {
        let info = ConfigWarning::info("server.secret_key", "auto-generated");
        assert_eq!(info.to_string(), "[INFO] server.secret_key: auto-generated");
    }
}

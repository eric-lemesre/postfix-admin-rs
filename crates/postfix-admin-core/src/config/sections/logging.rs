use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

use crate::config::ConfigError;

/// Logging configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub audit_retention_days: u32,
    pub syslog_enabled: bool,
    pub syslog_facility: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            audit_retention_days: 365,
            syslog_enabled: false,
            syslog_facility: "mail".to_string(),
        }
    }
}

/// Supported log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Returns `true` for verbose levels (trace, debug).
    #[must_use]
    pub fn is_verbose(self) -> bool {
        matches!(self, Self::Trace | Self::Debug)
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        };
        f.write_str(s)
    }
}

impl FromStr for LogLevel {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(ConfigError::validation(
                "logging.level",
                format!("unknown log level '{s}', expected: trace, debug, info, warn, error"),
            )),
        }
    }
}

/// Supported log output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl FromStr for LogFormat {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "pretty" => Ok(Self::Pretty),
            "compact" => Ok(Self::Compact),
            _ => Err(ConfigError::validation(
                "logging.format",
                format!("unknown log format '{s}', expected: json, pretty, compact"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_config_default_info_pretty() {
        let cfg = LoggingConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.format, "pretty");
    }

    #[test]
    fn log_level_from_str_valid() {
        assert_eq!(LogLevel::from_str("trace").ok(), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("DEBUG").ok(), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info").ok(), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("warn").ok(), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("warning").ok(), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("error").ok(), Some(LogLevel::Error));
    }

    #[test]
    fn log_level_from_str_invalid() {
        assert!(LogLevel::from_str("verbose").is_err());
    }

    #[test]
    fn log_level_is_verbose_for_trace_and_debug() {
        assert!(LogLevel::Trace.is_verbose());
        assert!(LogLevel::Debug.is_verbose());
        assert!(!LogLevel::Info.is_verbose());
        assert!(!LogLevel::Warn.is_verbose());
        assert!(!LogLevel::Error.is_verbose());
    }

    #[test]
    fn log_format_from_str_valid() {
        assert_eq!(LogFormat::from_str("json").ok(), Some(LogFormat::Json));
        assert_eq!(LogFormat::from_str("pretty").ok(), Some(LogFormat::Pretty));
        assert_eq!(
            LogFormat::from_str("compact").ok(),
            Some(LogFormat::Compact)
        );
    }

    #[test]
    fn log_format_from_str_invalid() {
        assert!(LogFormat::from_str("xml").is_err());
    }
}

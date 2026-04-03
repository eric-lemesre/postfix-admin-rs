use std::fmt;
use std::path::Path;
use std::str::FromStr;

use crate::config::ConfigError;

/// Application profile controlling default behaviors and validation rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    /// Development profile with relaxed defaults.
    #[default]
    Dev,
    /// Test profile for automated testing.
    Test,
    /// Pre-production profile with production-like validation.
    Prep,
    /// Production profile with strict validation.
    Prod,
}

impl Profile {
    /// Returns `true` for profiles that require production-grade security.
    #[must_use]
    pub fn is_production_like(self) -> bool {
        matches!(self, Self::Prep | Self::Prod)
    }

    /// Returns the filename for this profile (e.g., `"dev"`, `"test"`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Test => "test",
            Self::Prep => "prep",
            Self::Prod => "prod",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Profile {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Dev),
            "test" | "testing" => Ok(Self::Test),
            "prep" | "preprod" | "staging" => Ok(Self::Prep),
            "prod" | "production" => Ok(Self::Prod),
            _ => Err(ConfigError::validation(
                "profile",
                format!("unknown profile '{s}', expected: dev, test, prep, prod"),
            )),
        }
    }
}

/// Operating mode determines where configuration files are loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingMode {
    /// Development mode: loads from `./config/`.
    Development(Profile),
    /// Deployed mode: loads from `/etc/postfix-admin-rs/`.
    Deployed,
}

/// Path for deployed configuration.
pub const DEPLOYED_CONFIG_DIR: &str = "/etc/postfix-admin-rs";
/// Path for development configuration.
pub const DEV_CONFIG_DIR: &str = "./config";

impl OperatingMode {
    /// Detect the operating mode by checking filesystem paths.
    ///
    /// If `./config/` exists, uses `Development` mode with the given profile.
    /// If `/etc/postfix-admin-rs/` exists, uses `Deployed` mode.
    /// Falls back to `Development(Dev)`.
    #[must_use]
    pub fn detect(profile: Profile) -> Self {
        if Path::new(DEV_CONFIG_DIR).is_dir() {
            return Self::Development(profile);
        }
        if Path::new(DEPLOYED_CONFIG_DIR).is_dir() {
            return Self::Deployed;
        }
        Self::Development(profile)
    }

    /// Returns the profile for this mode.
    ///
    /// Deployed mode always uses `Prod`.
    #[must_use]
    pub fn profile(self) -> Profile {
        match self {
            Self::Development(profile) => profile,
            Self::Deployed => Profile::Prod,
        }
    }

    /// Returns `true` if the current mode requires production-grade validation.
    #[must_use]
    pub fn is_production_like(self) -> bool {
        match self {
            Self::Development(profile) => profile.is_production_like(),
            Self::Deployed => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_from_str_dev_variants_succeed() {
        assert_eq!(Profile::from_str("dev").ok(), Some(Profile::Dev));
        assert_eq!(Profile::from_str("development").ok(), Some(Profile::Dev));
        assert_eq!(Profile::from_str("DEV").ok(), Some(Profile::Dev));
    }

    #[test]
    fn profile_from_str_test_variants_succeed() {
        assert_eq!(Profile::from_str("test").ok(), Some(Profile::Test));
        assert_eq!(Profile::from_str("testing").ok(), Some(Profile::Test));
    }

    #[test]
    fn profile_from_str_prep_variants_succeed() {
        assert_eq!(Profile::from_str("prep").ok(), Some(Profile::Prep));
        assert_eq!(Profile::from_str("preprod").ok(), Some(Profile::Prep));
        assert_eq!(Profile::from_str("staging").ok(), Some(Profile::Prep));
    }

    #[test]
    fn profile_from_str_prod_variants_succeed() {
        assert_eq!(Profile::from_str("prod").ok(), Some(Profile::Prod));
        assert_eq!(Profile::from_str("production").ok(), Some(Profile::Prod));
    }

    #[test]
    fn profile_from_str_unknown_returns_error() {
        assert!(Profile::from_str("invalid").is_err());
    }

    #[test]
    fn profile_is_production_like_for_prep_and_prod() {
        assert!(!Profile::Dev.is_production_like());
        assert!(!Profile::Test.is_production_like());
        assert!(Profile::Prep.is_production_like());
        assert!(Profile::Prod.is_production_like());
    }

    #[test]
    fn profile_display_matches_as_str() {
        assert_eq!(Profile::Dev.to_string(), "dev");
        assert_eq!(Profile::Test.to_string(), "test");
        assert_eq!(Profile::Prep.to_string(), "prep");
        assert_eq!(Profile::Prod.to_string(), "prod");
    }

    #[test]
    fn profile_default_is_dev() {
        assert_eq!(Profile::default(), Profile::Dev);
    }

    #[test]
    fn operating_mode_deployed_always_prod_like() {
        let mode = OperatingMode::Deployed;
        assert!(mode.is_production_like());
        assert_eq!(mode.profile(), Profile::Prod);
    }

    #[test]
    fn operating_mode_development_reflects_profile() {
        let mode = OperatingMode::Development(Profile::Dev);
        assert!(!mode.is_production_like());
        assert_eq!(mode.profile(), Profile::Dev);

        let mode = OperatingMode::Development(Profile::Prod);
        assert!(mode.is_production_like());
        assert_eq!(mode.profile(), Profile::Prod);
    }
}

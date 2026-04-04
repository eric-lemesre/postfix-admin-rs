use serde::Deserialize;

/// API-specific configuration (CORS, rate limiting).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    pub cors: CorsConfig,
    pub rate_limit: ApiRateLimitConfig,
}

/// CORS configuration for the REST API.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_secs: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            max_age_secs: 3600,
        }
    }
}

/// API-level rate limiting configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ApiRateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for ApiRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 100,
            burst_size: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_config_default_has_cors_and_rate_limit() {
        let cfg = ApiConfig::default();
        assert_eq!(cfg.cors.allowed_origins, vec!["*"]);
        assert_eq!(cfg.cors.max_age_secs, 3600);
        assert!(cfg.rate_limit.enabled);
        assert_eq!(cfg.rate_limit.requests_per_minute, 100);
        assert_eq!(cfg.rate_limit.burst_size, 20);
    }

    #[test]
    fn cors_config_default_allows_standard_methods() {
        let cfg = CorsConfig::default();
        assert!(cfg.allowed_methods.contains(&"GET".to_string()));
        assert!(cfg.allowed_methods.contains(&"POST".to_string()));
        assert!(cfg.allowed_methods.contains(&"DELETE".to_string()));
    }
}

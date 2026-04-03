use serde::Deserialize;

/// gRPC server configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GrpcConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub tls_cert_path: String,
    pub tls_key_path: String,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_address: "0.0.0.0".to_string(),
            port: 50051,
            tls_enabled: false,
            tls_cert_path: String::new(),
            tls_key_path: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_config_default_disabled() {
        let cfg = GrpcConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.port, 50051);
    }
}

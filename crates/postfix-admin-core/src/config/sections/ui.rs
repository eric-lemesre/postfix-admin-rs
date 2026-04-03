use serde::Deserialize;

/// Web UI configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub page_size: u32,
    pub default_language: String,
    pub available_languages: Vec<String>,
    pub default_theme: String,
    pub site_name: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            page_size: 20,
            default_language: "en".to_string(),
            available_languages: vec!["en".to_string(), "fr".to_string()],
            default_theme: "auto".to_string(),
            site_name: "PostfixAdmin".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_config_default_has_en_fr_languages() {
        let cfg = UiConfig::default();
        assert_eq!(cfg.available_languages, vec!["en", "fr"]);
        assert_eq!(cfg.page_size, 20);
    }
}

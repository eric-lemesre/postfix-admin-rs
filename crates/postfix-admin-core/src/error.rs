use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("not found: {entity} '{id}'")]
    NotFound { entity: &'static str, id: String },

    #[error("already exists: {entity} '{id}'")]
    AlreadyExists { entity: &'static str, id: String },

    #[error("repository error: {0}")]
    Repository(String),
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("invalid {field}: {reason}")]
    InvalidField { field: &'static str, reason: String },

    #[error("multiple validation errors: {0:?}")]
    Multiple(Vec<ValidationError>),
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("quota exceeded for domain '{domain}': {reason}")]
    QuotaExceeded { domain: String, reason: String },

    #[error("limit reached for domain '{domain}': {reason}")]
    LimitReached { domain: String, reason: String },

    #[error("domain '{domain}' is not active")]
    Inactive { domain: String },

    #[error("alias loop detected: {path}")]
    AliasLoop { path: String },
}

impl CoreError {
    #[must_use]
    pub fn not_found(entity: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            id: id.into(),
        }
    }

    #[must_use]
    pub fn already_exists(entity: &'static str, id: impl Into<String>) -> Self {
        Self::AlreadyExists {
            entity,
            id: id.into(),
        }
    }

    #[must_use]
    pub fn repository(msg: impl Into<String>) -> Self {
        Self::Repository(msg.into())
    }
}

impl ValidationError {
    #[must_use]
    pub fn invalid_field(field: &'static str, reason: impl Into<String>) -> Self {
        Self::InvalidField {
            field,
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display_shows_entity_and_id() {
        let err = CoreError::not_found("domain", "example.com");
        assert_eq!(err.to_string(), "not found: domain 'example.com'");
    }

    #[test]
    fn already_exists_display_shows_entity_and_id() {
        let err = CoreError::already_exists("mailbox", "user@example.com");
        assert_eq!(
            err.to_string(),
            "already exists: mailbox 'user@example.com'"
        );
    }

    #[test]
    fn validation_error_converts_to_core_error() {
        let val_err = ValidationError::invalid_field("email", "invalid format");
        let core_err: CoreError = val_err.into();
        assert!(matches!(core_err, CoreError::Validation(_)));
    }

    #[test]
    fn domain_error_converts_to_core_error() {
        let dom_err = DomainError::Inactive {
            domain: "test.com".to_string(),
        };
        let core_err: CoreError = dom_err.into();
        assert!(matches!(core_err, CoreError::Domain(_)));
    }

    #[test]
    fn repository_error_display_shows_message() {
        let err = CoreError::repository("connection failed");
        assert_eq!(err.to_string(), "repository error: connection failed");
    }
}

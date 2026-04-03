use postfix_admin_core::CoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("not found: {entity} '{id}'")]
    NotFound { entity: &'static str, id: String },

    #[error("already exists: {entity} '{id}'")]
    AlreadyExists { entity: &'static str, id: String },
}

impl DbError {
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
}

impl From<DbError> for CoreError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound { entity, id } => CoreError::NotFound { entity, id },
            DbError::AlreadyExists { entity, id } => CoreError::AlreadyExists { entity, id },
            DbError::Sqlx(e) => CoreError::repository(e.to_string()),
            DbError::Migration(e) => CoreError::repository(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_converts_to_core_error() {
        let db_err = DbError::not_found("domain", "example.com");
        let core_err: CoreError = db_err.into();
        assert!(matches!(
            core_err,
            CoreError::NotFound {
                entity: "domain",
                ..
            }
        ));
    }

    #[test]
    fn already_exists_converts_to_core_error() {
        let db_err = DbError::already_exists("mailbox", "user@example.com");
        let core_err: CoreError = db_err.into();
        assert!(matches!(
            core_err,
            CoreError::AlreadyExists {
                entity: "mailbox",
                ..
            }
        ));
    }

    #[test]
    fn sqlx_error_converts_to_repository_error() {
        let sqlx_err = sqlx::Error::PoolTimedOut;
        let db_err = DbError::Sqlx(sqlx_err);
        let core_err: CoreError = db_err.into();
        assert!(matches!(core_err, CoreError::Repository(_)));
    }

    #[test]
    fn display_not_found_shows_entity_and_id() {
        let err = DbError::not_found("domain", "example.com");
        assert_eq!(err.to_string(), "not found: domain 'example.com'");
    }
}

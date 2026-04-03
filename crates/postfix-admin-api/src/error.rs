//! RFC 7807 Problem Details error handling for the REST API.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use postfix_admin_auth::AuthError;
use postfix_admin_core::error::{CoreError, DomainError, ValidationError};

/// RFC 7807 Problem Details response body.
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

/// Unified API error type that converts to RFC 7807 responses.
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    AlreadyExists(String),
    Validation(String),
    DomainRule(String),
    Auth(AuthError),
    Internal(String),
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::NotFound { entity, id } => {
                Self::NotFound(format!("{entity} '{id}' not found"))
            }
            CoreError::AlreadyExists { entity, id } => {
                Self::AlreadyExists(format!("{entity} '{id}' already exists"))
            }
            CoreError::Validation(ValidationError::InvalidField { field, reason }) => {
                Self::Validation(format!("{field}: {reason}"))
            }
            CoreError::Validation(ValidationError::Multiple(errors)) => {
                let msgs: Vec<String> = errors.iter().map(ToString::to_string).collect();
                Self::Validation(msgs.join("; "))
            }
            CoreError::Domain(
                DomainError::QuotaExceeded { reason, .. }
                | DomainError::LimitReached { reason, .. },
            ) => Self::DomainRule(reason),
            CoreError::Domain(DomainError::Inactive { domain }) => {
                Self::DomainRule(format!("domain '{domain}' is inactive"))
            }
            CoreError::Domain(DomainError::AliasLoop { path }) => {
                Self::DomainRule(format!("alias loop: {path}"))
            }
            CoreError::Repository(msg) => Self::Internal(msg),
            CoreError::Config(err) => Self::Internal(err.to_string()),
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        Self::Auth(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, problem) = match self {
            Self::NotFound(detail) => (
                StatusCode::NOT_FOUND,
                ProblemDetails {
                    problem_type: "about:blank",
                    title: "Not Found",
                    status: 404,
                    detail,
                    field: None,
                },
            ),
            Self::AlreadyExists(detail) => (
                StatusCode::CONFLICT,
                ProblemDetails {
                    problem_type: "about:blank",
                    title: "Conflict",
                    status: 409,
                    detail,
                    field: None,
                },
            ),
            Self::Validation(detail) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ProblemDetails {
                    problem_type: "about:blank",
                    title: "Validation Error",
                    status: 422,
                    detail,
                    field: None,
                },
            ),
            Self::DomainRule(detail) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ProblemDetails {
                    problem_type: "about:blank",
                    title: "Business Rule Violation",
                    status: 422,
                    detail,
                    field: None,
                },
            ),
            Self::Auth(ref auth_err) => {
                let (code, title) = match auth_err {
                    AuthError::InvalidCredentials => {
                        (StatusCode::UNAUTHORIZED, "Invalid Credentials")
                    }
                    AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token Expired"),
                    AuthError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "Invalid Token"),
                    AuthError::MissingAuthorization => {
                        (StatusCode::UNAUTHORIZED, "Missing Authorization")
                    }
                    AuthError::AccountInactive => (StatusCode::FORBIDDEN, "Account Inactive"),
                    AuthError::AccountLocked(_) => (StatusCode::FORBIDDEN, "Account Locked"),
                    AuthError::InsufficientPermissions(_) => {
                        (StatusCode::FORBIDDEN, "Insufficient Permissions")
                    }
                    AuthError::InvalidTotpCode | AuthError::TotpReplay => {
                        (StatusCode::UNAUTHORIZED, "Invalid TOTP Code")
                    }
                    AuthError::CsrfError => (StatusCode::FORBIDDEN, "CSRF Validation Failed"),
                    AuthError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "Rate Limited"),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication Error"),
                };
                (
                    code,
                    ProblemDetails {
                        problem_type: "about:blank",
                        title,
                        status: code.as_u16(),
                        detail: auth_err.to_string(),
                        field: None,
                    },
                )
            }
            Self::Internal(detail) => {
                tracing::error!(error = %detail, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ProblemDetails {
                        problem_type: "about:blank",
                        title: "Internal Server Error",
                        status: 500,
                        detail: "an internal error occurred".to_string(),
                        field: None,
                    },
                )
            }
        };

        (status, Json(problem)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_not_found_maps_to_404() {
        let err: ApiError = CoreError::not_found("domain", "example.com").into();
        assert!(matches!(err, ApiError::NotFound(_)));
    }

    #[test]
    fn core_already_exists_maps_to_conflict() {
        let err: ApiError = CoreError::already_exists("domain", "example.com").into();
        assert!(matches!(err, ApiError::AlreadyExists(_)));
    }

    #[test]
    fn auth_error_maps_to_api_error() {
        let err: ApiError = AuthError::InvalidCredentials.into();
        assert!(matches!(err, ApiError::Auth(AuthError::InvalidCredentials)));
    }
}

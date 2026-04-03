//! Authentication extractors for API handlers.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::error::ProblemDetails;
use crate::state::AppState;
use postfix_admin_auth::Claims;

/// Authenticated admin identity extracted from JWT Bearer token.
#[derive(Debug, Clone)]
pub struct AuthAdmin {
    pub username: String,
    pub superadmin: bool,
}

/// Rejection when authentication fails.
pub struct AuthRejection(ProblemDetails);

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.0.status).unwrap_or(StatusCode::UNAUTHORIZED),
            Json(self.0),
        )
            .into_response()
    }
}

impl FromRequestParts<AppState> for AuthAdmin {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                AuthRejection(ProblemDetails {
                    problem_type: "about:blank",
                    title: "Missing Authorization",
                    status: 401,
                    detail: "authorization header required".to_string(),
                    field: None,
                })
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            AuthRejection(ProblemDetails {
                problem_type: "about:blank",
                title: "Invalid Authorization",
                status: 401,
                detail: "expected Bearer token".to_string(),
                field: None,
            })
        })?;

        let claims: Claims = state.jwt.verify_access(token).map_err(|e| {
            AuthRejection(ProblemDetails {
                problem_type: "about:blank",
                title: "Invalid Token",
                status: 401,
                detail: e.to_string(),
                field: None,
            })
        })?;

        Ok(Self {
            username: claims.sub,
            superadmin: claims.superadmin,
        })
    }
}

/// Extractor that requires superadmin privileges.
#[derive(Debug, Clone)]
pub struct RequireSuperAdmin(pub AuthAdmin);

impl FromRequestParts<AppState> for RequireSuperAdmin {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let admin = AuthAdmin::from_request_parts(parts, state).await?;
        if !admin.superadmin {
            return Err(AuthRejection(ProblemDetails {
                problem_type: "about:blank",
                title: "Insufficient Permissions",
                status: 403,
                detail: "superadmin access required".to_string(),
                field: None,
            }));
        }
        Ok(Self(admin))
    }
}

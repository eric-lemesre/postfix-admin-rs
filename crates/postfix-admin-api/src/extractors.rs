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
///
/// When mTLS is configured as required for superadmins, also verifies the
/// client certificate via reverse proxy headers.
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

        // Check mTLS requirement for superadmin
        if state.mtls_verifier.is_enabled() && state.mtls_verifier.required_for_superadmin() {
            let headers = extract_header_map(parts);
            let cert_info = state.mtls_verifier.extract(&headers).map_err(|e| {
                AuthRejection(ProblemDetails {
                    problem_type: "about:blank",
                    title: "mTLS Required",
                    status: 403,
                    detail: e.to_string(),
                    field: None,
                })
            })?;

            // Verify the certificate identity matches the admin username
            if cert_info.identity != admin.username {
                return Err(AuthRejection(ProblemDetails {
                    problem_type: "about:blank",
                    title: "Certificate Mismatch",
                    status: 403,
                    detail: "client certificate identity does not match admin".to_string(),
                    field: None,
                }));
            }
        }

        Ok(Self(admin))
    }
}

/// Extractor that requires domain admin access to a specific domain.
///
/// Extracts the domain from the path parameter and verifies the admin
/// has access to it (either as superadmin or via `domain_admins`).
#[derive(Debug, Clone)]
pub struct RequireDomainAdmin {
    pub admin: AuthAdmin,
    pub domain: String,
}

impl FromRequestParts<AppState> for RequireDomainAdmin {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let admin = AuthAdmin::from_request_parts(parts, state).await?;

        // Extract domain from path parameters
        let domain = parts
            .uri
            .path()
            .split('/')
            .find(|s| s.contains('.'))
            .unwrap_or_default()
            .to_string();

        if domain.is_empty() {
            return Err(AuthRejection(ProblemDetails {
                problem_type: "about:blank",
                title: "Missing Domain",
                status: 400,
                detail: "domain parameter required".to_string(),
                field: None,
            }));
        }

        // Superadmins have access to all domains
        if !admin.superadmin {
            let email = postfix_admin_core::EmailAddress::try_from(admin.username.clone())
                .map_err(|_| {
                    AuthRejection(ProblemDetails {
                        problem_type: "about:blank",
                        title: "Internal Error",
                        status: 500,
                        detail: "invalid admin username".to_string(),
                        field: None,
                    })
                })?;

            let admin_domains = state.admins.find_admin_domains(&email).await.map_err(|e| {
                AuthRejection(ProblemDetails {
                    problem_type: "about:blank",
                    title: "Internal Error",
                    status: 500,
                    detail: e.to_string(),
                    field: None,
                })
            })?;

            let has_access = admin_domains.iter().any(|d| d.as_str() == domain);
            if !has_access {
                return Err(AuthRejection(ProblemDetails {
                    problem_type: "about:blank",
                    title: "Insufficient Permissions",
                    status: 403,
                    detail: format!("no access to domain '{domain}'"),
                    field: None,
                }));
            }
        }

        Ok(Self { admin, domain })
    }
}

/// Extract HTTP headers into a `HashMap` for mTLS verification.
fn extract_header_map(parts: &Parts) -> std::collections::HashMap<String, String> {
    parts
        .headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
        .collect()
}

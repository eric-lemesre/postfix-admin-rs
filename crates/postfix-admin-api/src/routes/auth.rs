//! Authentication endpoints: login, refresh, logout.

use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractors::AuthAdmin;
use crate::state::AppState;
use postfix_admin_auth::{verify_password, TokenPair};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenPair>, ApiError> {
    let username = postfix_admin_core::EmailAddress::try_from(body.username)
        .map_err(|e| ApiError::Validation(format!("invalid username: {e}")))?;

    let admin = state
        .admins
        .find_by_username(&username)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::Auth(
            postfix_admin_auth::AuthError::InvalidCredentials,
        ))?;

    if !admin.active {
        return Err(ApiError::Auth(
            postfix_admin_auth::AuthError::AccountInactive,
        ));
    }

    // In a real implementation, we would fetch the stored password hash
    // from the database. For now, we verify against a placeholder.
    // The admin repository returns AdminResponse which doesn't include
    // the password hash (by design). We need a dedicated auth query.
    // TODO: Add find_password_hash to AdminRepository
    let _verified = verify_password(&body.password, "placeholder").map_err(ApiError::Auth)?;

    let pair = state
        .jwt
        .issue(username.to_string().as_str(), admin.superadmin)?;

    Ok(Json(pair))
}

/// POST /api/v1/auth/refresh
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<TokenPair>, ApiError> {
    let claims = state.jwt.verify_refresh(&body.refresh_token)?;
    let pair = state.jwt.issue(&claims.sub, claims.superadmin)?;
    Ok(Json(pair))
}

/// POST /api/v1/auth/logout
pub async fn logout(_admin: AuthAdmin) -> Result<Json<serde_json::Value>, ApiError> {
    // Server-side token invalidation would require a blocklist.
    // For now, clients simply discard their tokens.
    Ok(Json(serde_json::json!({ "message": "logged out" })))
}

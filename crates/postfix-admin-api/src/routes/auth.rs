//! Authentication endpoints: login, refresh, logout.

use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractors::AuthAdmin;
use crate::state::AppState;
use postfix_admin_auth::{hash_password, needs_rehash, verify_password, PasswordScheme, TokenPair};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
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
    // Extract client IP for rate limiting (from X-Forwarded-For or peer address)
    let client_ip = "unknown".to_string(); // TODO: extract from request extensions

    // Check rate limiting
    state
        .rate_limiter
        .check_allowed(&client_ip)
        .map_err(ApiError::Auth)?;

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

    // Fetch the actual password hash from the database
    let password_hash = state
        .admins
        .find_password_hash(&username)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::Auth(
            postfix_admin_auth::AuthError::InvalidCredentials,
        ))?;

    // Verify the password
    let verified = verify_password(&body.password, &password_hash).map_err(ApiError::Auth)?;
    if !verified {
        state.rate_limiter.record_failure(&client_ip);
        return Err(ApiError::Auth(
            postfix_admin_auth::AuthError::InvalidCredentials,
        ));
    }

    // Rehash if needed (upgrade to current scheme)
    if let Ok(scheme) = PasswordScheme::from_config(&state.password_scheme) {
        if needs_rehash(&password_hash, scheme) {
            if let Ok(new_hash) = hash_password(&body.password, scheme) {
                let update = postfix_admin_core::dto::UpdateAdmin {
                    password: postfix_admin_core::Password::try_from(new_hash).ok(),
                    superadmin: None,
                    totp_enabled: None,
                    active: None,
                };
                // Best-effort rehash — don't fail login if this fails
                let _ = state.admins.update(&username, &update).await;
            }
        }
    }

    // TODO: TOTP verification if admin.totp_enabled
    // This would check body.totp_code against the stored TOTP secret

    // Record successful login
    state.rate_limiter.record_success(&client_ip);

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

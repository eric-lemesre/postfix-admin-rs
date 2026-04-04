//! Vacation auto-responder endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiError;
use crate::extractors::AuthAdmin;
use crate::response::ApiResponse;
use crate::state::AppState;
use postfix_admin_core::dto::{UpdateVacation, VacationResponse};
use postfix_admin_core::types::EmailAddress;

/// GET /api/v1/mailboxes/:username/vacation
pub async fn get(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ApiResponse<VacationResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let vacation = state
        .vacations
        .find_by_email(&email)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("vacation for '{email}'")))?;
    Ok(Json(ApiResponse::new(vacation)))
}

/// PUT /api/v1/mailboxes/:username/vacation
pub async fn upsert(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(body): Json<UpdateVacation>,
) -> Result<Json<ApiResponse<VacationResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let vacation = state.vacations.upsert(&email, &body).await?;
    Ok(Json(ApiResponse::new(vacation)))
}

/// DELETE /api/v1/mailboxes/:username/vacation
pub async fn delete(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    state.vacations.delete(&email).await?;
    Ok(StatusCode::NO_CONTENT)
}

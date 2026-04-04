//! Admin CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiError;
use crate::extractors::RequireSuperAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{AdminResponse, CreateAdmin, UpdateAdmin};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::EmailAddress;

/// GET /api/v1/admins
pub async fn list(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Query(page): Query<PageRequest>,
) -> Result<Json<ApiListResponse<AdminResponse>>, ApiError> {
    let result = state.admins.find_all(&page).await?;
    Ok(Json(result.into()))
}

/// GET /api/v1/admins/:username
pub async fn get(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ApiResponse<AdminResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let admin = state
        .admins
        .find_by_username(&email)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("admin '{email}'")))?;
    Ok(Json(ApiResponse::new(admin)))
}

/// POST /api/v1/admins
pub async fn create(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateAdmin>,
) -> Result<(StatusCode, Json<ApiResponse<AdminResponse>>), ApiError> {
    let admin = state.admins.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(admin))))
}

/// PUT /api/v1/admins/:username
pub async fn update(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(body): Json<UpdateAdmin>,
) -> Result<Json<ApiResponse<AdminResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let admin = state.admins.update(&email, &body).await?;
    Ok(Json(ApiResponse::new(admin)))
}

/// DELETE /api/v1/admins/:username
pub async fn delete(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    state.admins.delete(&email).await?;
    Ok(StatusCode::NO_CONTENT)
}

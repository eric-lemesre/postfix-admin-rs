//! Fetchmail CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::AuthAdmin;
use crate::state::AppState;
use postfix_admin_core::dto::{CreateFetchmail, FetchmailResponse, UpdateFetchmail};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::EmailAddress;
use postfix_admin_core::PageResponse;

/// GET /api/v1/mailboxes/:username/fetchmail
pub async fn list(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(page): Query<PageRequest>,
) -> Result<Json<PageResponse<FetchmailResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let result = state.fetchmail.find_by_mailbox(&email, &page).await?;
    Ok(Json(result))
}

/// GET /api/v1/fetchmail/:id
pub async fn get(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FetchmailResponse>, ApiError> {
    let entry = state
        .fetchmail
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("fetchmail entry '{id}'")))?;
    Ok(Json(entry))
}

/// POST /api/v1/fetchmail
pub async fn create(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateFetchmail>,
) -> Result<(StatusCode, Json<FetchmailResponse>), ApiError> {
    let entry = state.fetchmail.create(&body).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

/// PUT /api/v1/fetchmail/:id
pub async fn update(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFetchmail>,
) -> Result<Json<FetchmailResponse>, ApiError> {
    let entry = state.fetchmail.update(id, &body).await?;
    Ok(Json(entry))
}

/// DELETE /api/v1/fetchmail/:id
pub async fn delete(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.fetchmail.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

//! Mailbox CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiError;
use crate::extractors::AuthAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{CreateMailbox, MailboxResponse, UpdateMailbox};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::{DomainName, EmailAddress};

/// GET /api/v1/domains/:domain/mailboxes
pub async fn list(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(domain): Path<String>,
    Query(page): Query<PageRequest>,
) -> Result<Json<ApiListResponse<MailboxResponse>>, ApiError> {
    let domain_name = DomainName::try_from(domain)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let result = state.mailboxes.find_by_domain(&domain_name, &page).await?;
    Ok(Json(result.into()))
}

/// GET /api/v1/mailboxes/:username
pub async fn get(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ApiResponse<MailboxResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let mailbox = state
        .mailboxes
        .find_by_username(&email)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("mailbox '{email}'")))?;
    Ok(Json(ApiResponse::new(mailbox)))
}

/// POST /api/v1/mailboxes
pub async fn create(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateMailbox>,
) -> Result<(StatusCode, Json<ApiResponse<MailboxResponse>>), ApiError> {
    let mailbox = state.mailboxes.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(mailbox))))
}

/// PUT /api/v1/mailboxes/:username
pub async fn update(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(body): Json<UpdateMailbox>,
) -> Result<Json<ApiResponse<MailboxResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let mailbox = state.mailboxes.update(&email, &body).await?;
    Ok(Json(ApiResponse::new(mailbox)))
}

/// DELETE /api/v1/mailboxes/:username
pub async fn delete(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    state.mailboxes.delete(&email).await?;
    Ok(StatusCode::NO_CONTENT)
}

//! Alias CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiError;
use crate::extractors::AuthAdmin;
use crate::state::AppState;
use postfix_admin_core::dto::{AliasResponse, CreateAlias, UpdateAlias};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::{DomainName, EmailAddress};
use postfix_admin_core::PageResponse;

/// GET /api/v1/domains/:domain/aliases
pub async fn list(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(domain): Path<String>,
    Query(page): Query<PageRequest>,
) -> Result<Json<PageResponse<AliasResponse>>, ApiError> {
    let domain_name = DomainName::try_from(domain)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let result = state.aliases.find_by_domain(&domain_name, &page).await?;
    Ok(Json(result))
}

/// GET /api/v1/aliases/:address
pub async fn get(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<AliasResponse>, ApiError> {
    let email = EmailAddress::try_from(address)
        .map_err(|e| ApiError::Validation(format!("invalid address: {e}")))?;
    let alias = state
        .aliases
        .find_by_address(&email)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("alias '{email}'")))?;
    Ok(Json(alias))
}

/// POST /api/v1/aliases
pub async fn create(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateAlias>,
) -> Result<(StatusCode, Json<AliasResponse>), ApiError> {
    let alias = state.aliases.create(&body).await?;
    Ok((StatusCode::CREATED, Json(alias)))
}

/// PUT /api/v1/aliases/:address
pub async fn update(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(body): Json<UpdateAlias>,
) -> Result<Json<AliasResponse>, ApiError> {
    let email = EmailAddress::try_from(address)
        .map_err(|e| ApiError::Validation(format!("invalid address: {e}")))?;
    let alias = state.aliases.update(&email, &body).await?;
    Ok(Json(alias))
}

/// DELETE /api/v1/aliases/:address
pub async fn delete(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<StatusCode, ApiError> {
    let email = EmailAddress::try_from(address)
        .map_err(|e| ApiError::Validation(format!("invalid address: {e}")))?;
    state.aliases.delete(&email).await?;
    Ok(StatusCode::NO_CONTENT)
}

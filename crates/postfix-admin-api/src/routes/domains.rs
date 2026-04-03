//! Domain CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiError;
use crate::extractors::RequireSuperAdmin;
use crate::state::AppState;
use postfix_admin_core::dto::{CreateDomain, DomainResponse, UpdateDomain};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::DomainName;
use postfix_admin_core::PageResponse;

/// GET /api/v1/domains
pub async fn list(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Query(page): Query<PageRequest>,
) -> Result<Json<PageResponse<DomainResponse>>, ApiError> {
    let result = state.domains.find_all(&page).await?;
    Ok(Json(result))
}

/// GET /api/v1/domains/:name
pub async fn get(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DomainResponse>, ApiError> {
    let domain_name = DomainName::try_from(name)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let domain = state
        .domains
        .find_by_name(&domain_name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("domain '{domain_name}'")))?;
    Ok(Json(domain))
}

/// POST /api/v1/domains
pub async fn create(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateDomain>,
) -> Result<(StatusCode, Json<DomainResponse>), ApiError> {
    let domain = state.domains.create(&body).await?;
    Ok((StatusCode::CREATED, Json(domain)))
}

/// PUT /api/v1/domains/:name
pub async fn update(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateDomain>,
) -> Result<Json<DomainResponse>, ApiError> {
    let domain_name = DomainName::try_from(name)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let domain = state.domains.update(&domain_name, &body).await?;
    Ok(Json(domain))
}

/// DELETE /api/v1/domains/:name
pub async fn delete(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    let domain_name = DomainName::try_from(name)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    state.domains.delete(&domain_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

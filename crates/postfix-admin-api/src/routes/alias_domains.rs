//! Alias domain endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiError;
use crate::extractors::RequireSuperAdmin;
use crate::state::AppState;
use postfix_admin_core::dto::{AliasDomainResponse, CreateAliasDomain};
use postfix_admin_core::types::DomainName;

/// GET /api/v1/domains/:domain/alias-domains
pub async fn list_by_target(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<Json<Vec<AliasDomainResponse>>, ApiError> {
    let domain_name = DomainName::try_from(domain)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let result = state.alias_domains.find_by_target(&domain_name).await?;
    Ok(Json(result))
}

/// GET /api/v1/alias-domains/:alias
pub async fn get(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(alias): Path<String>,
) -> Result<Json<AliasDomainResponse>, ApiError> {
    let domain_name = DomainName::try_from(alias)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let alias_domain = state
        .alias_domains
        .find_by_alias(&domain_name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("alias domain '{domain_name}'")))?;
    Ok(Json(alias_domain))
}

/// POST /api/v1/alias-domains
pub async fn create(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateAliasDomain>,
) -> Result<(StatusCode, Json<AliasDomainResponse>), ApiError> {
    let alias_domain = state.alias_domains.create(&body).await?;
    Ok((StatusCode::CREATED, Json(alias_domain)))
}

/// DELETE /api/v1/alias-domains/:alias
pub async fn delete(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(alias): Path<String>,
) -> Result<StatusCode, ApiError> {
    let domain_name = DomainName::try_from(alias)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    state.alias_domains.delete(&domain_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

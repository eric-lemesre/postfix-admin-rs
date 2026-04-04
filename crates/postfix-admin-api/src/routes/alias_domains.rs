//! Alias domain endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::RequireSuperAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{AliasDomainResponse, CreateAliasDomain};
use postfix_admin_core::types::DomainName;

/// GET /api/v1/domains/:domain/alias-domains
#[utoipa::path(
    get,
    path = "/api/v1/domains/{domain}/alias-domains",
    tag = "alias-domains",
    operation_id = "list_alias_domains",
    params(("domain" = String, Path, description = "Target domain name")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of alias domains", body = ApiListResponse<AliasDomainResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
        (status = 403, description = "Not a superadmin", body = ProblemDetails),
    )
)]
pub async fn list_by_target(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<Json<ApiListResponse<AliasDomainResponse>>, ApiError> {
    let domain_name = DomainName::try_from(domain)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let result = state.alias_domains.find_by_target(&domain_name).await?;
    Ok(Json(ApiListResponse::from_vec(result)))
}

/// GET /api/v1/alias-domains/:alias
#[utoipa::path(
    get,
    path = "/api/v1/alias-domains/{alias}",
    tag = "alias-domains",
    operation_id = "get_alias_domain",
    params(("alias" = String, Path, description = "Alias domain name")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Alias domain found", body = ApiResponse<AliasDomainResponse>),
        (status = 404, description = "Alias domain not found", body = ProblemDetails),
    )
)]
pub async fn get(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<AliasDomainResponse>>, ApiError> {
    let domain_name = DomainName::try_from(alias)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let alias_domain = state
        .alias_domains
        .find_by_alias(&domain_name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("alias domain '{domain_name}'")))?;
    Ok(Json(ApiResponse::new(alias_domain)))
}

/// POST /api/v1/alias-domains
#[utoipa::path(
    post,
    path = "/api/v1/alias-domains",
    tag = "alias-domains",
    operation_id = "create_alias_domain",
    security(("bearer_auth" = [])),
    request_body = CreateAliasDomain,
    responses(
        (status = 201, description = "Alias domain created", body = ApiResponse<AliasDomainResponse>),
        (status = 409, description = "Alias domain already exists", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateAliasDomain>,
) -> Result<(StatusCode, Json<ApiResponse<AliasDomainResponse>>), ApiError> {
    let alias_domain = state.alias_domains.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(alias_domain))))
}

/// DELETE /api/v1/alias-domains/:alias
#[utoipa::path(
    delete,
    path = "/api/v1/alias-domains/{alias}",
    tag = "alias-domains",
    operation_id = "delete_alias_domain",
    params(("alias" = String, Path, description = "Alias domain name")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Alias domain deleted"),
        (status = 404, description = "Alias domain not found", body = ProblemDetails),
    )
)]
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

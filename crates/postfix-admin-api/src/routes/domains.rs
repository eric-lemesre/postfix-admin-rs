//! Domain CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::RequireSuperAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{CreateDomain, DomainResponse, UpdateDomain};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::DomainName;

/// GET /api/v1/domains
#[utoipa::path(
    get,
    path = "/api/v1/domains",
    tag = "domains",
    operation_id = "list_domains",
    params(PageRequest),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of domains", body = ApiListResponse<DomainResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
        (status = 403, description = "Not a superadmin", body = ProblemDetails),
    )
)]
pub async fn list(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Query(page): Query<PageRequest>,
) -> Result<Json<ApiListResponse<DomainResponse>>, ApiError> {
    let result = state.domains.find_all(&page).await?;
    Ok(Json(result.into()))
}

/// GET /api/v1/domains/:name
#[utoipa::path(
    get,
    path = "/api/v1/domains/{name}",
    tag = "domains",
    operation_id = "get_domain",
    params(("name" = String, Path, description = "Domain name")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Domain found", body = ApiResponse<DomainResponse>),
        (status = 404, description = "Domain not found", body = ProblemDetails),
    )
)]
pub async fn get(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<DomainResponse>>, ApiError> {
    let domain_name = DomainName::try_from(name)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let domain = state
        .domains
        .find_by_name(&domain_name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("domain '{domain_name}'")))?;
    Ok(Json(ApiResponse::new(domain)))
}

/// POST /api/v1/domains
#[utoipa::path(
    post,
    path = "/api/v1/domains",
    tag = "domains",
    operation_id = "create_domain",
    security(("bearer_auth" = [])),
    request_body = CreateDomain,
    responses(
        (status = 201, description = "Domain created", body = ApiResponse<DomainResponse>),
        (status = 409, description = "Domain already exists", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateDomain>,
) -> Result<(StatusCode, Json<ApiResponse<DomainResponse>>), ApiError> {
    let domain = state.domains.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(domain))))
}

/// PUT /api/v1/domains/:name
#[utoipa::path(
    put,
    path = "/api/v1/domains/{name}",
    tag = "domains",
    operation_id = "update_domain",
    params(("name" = String, Path, description = "Domain name")),
    security(("bearer_auth" = [])),
    request_body = UpdateDomain,
    responses(
        (status = 200, description = "Domain updated", body = ApiResponse<DomainResponse>),
        (status = 404, description = "Domain not found", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn update(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateDomain>,
) -> Result<Json<ApiResponse<DomainResponse>>, ApiError> {
    let domain_name = DomainName::try_from(name)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let domain = state.domains.update(&domain_name, &body).await?;
    Ok(Json(ApiResponse::new(domain)))
}

/// DELETE /api/v1/domains/:name
#[utoipa::path(
    delete,
    path = "/api/v1/domains/{name}",
    tag = "domains",
    operation_id = "delete_domain",
    params(("name" = String, Path, description = "Domain name")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Domain deleted"),
        (status = 404, description = "Domain not found", body = ProblemDetails),
    )
)]
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

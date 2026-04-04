//! Alias CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::AuthAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{AliasResponse, CreateAlias, UpdateAlias};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::{DomainName, EmailAddress};

/// GET /api/v1/domains/:domain/aliases
#[utoipa::path(
    get,
    path = "/api/v1/domains/{domain}/aliases",
    tag = "aliases",
    operation_id = "list_aliases",
    params(
        ("domain" = String, Path, description = "Domain name"),
        PageRequest,
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of aliases", body = ApiListResponse<AliasResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
    )
)]
pub async fn list(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(domain): Path<String>,
    Query(page): Query<PageRequest>,
) -> Result<Json<ApiListResponse<AliasResponse>>, ApiError> {
    let domain_name = DomainName::try_from(domain)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let result = state.aliases.find_by_domain(&domain_name, &page).await?;
    Ok(Json(result.into()))
}

/// GET /api/v1/aliases/:address
#[utoipa::path(
    get,
    path = "/api/v1/aliases/{address}",
    tag = "aliases",
    operation_id = "get_alias",
    params(("address" = String, Path, description = "Alias email address")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Alias found", body = ApiResponse<AliasResponse>),
        (status = 404, description = "Alias not found", body = ProblemDetails),
    )
)]
pub async fn get(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<ApiResponse<AliasResponse>>, ApiError> {
    let email = EmailAddress::try_from(address)
        .map_err(|e| ApiError::Validation(format!("invalid address: {e}")))?;
    let alias = state
        .aliases
        .find_by_address(&email)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("alias '{email}'")))?;
    Ok(Json(ApiResponse::new(alias)))
}

/// POST /api/v1/aliases
#[utoipa::path(
    post,
    path = "/api/v1/aliases",
    tag = "aliases",
    operation_id = "create_alias",
    security(("bearer_auth" = [])),
    request_body = CreateAlias,
    responses(
        (status = 201, description = "Alias created", body = ApiResponse<AliasResponse>),
        (status = 409, description = "Alias already exists", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateAlias>,
) -> Result<(StatusCode, Json<ApiResponse<AliasResponse>>), ApiError> {
    let alias = state.aliases.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(alias))))
}

/// PUT /api/v1/aliases/:address
#[utoipa::path(
    put,
    path = "/api/v1/aliases/{address}",
    tag = "aliases",
    operation_id = "update_alias",
    params(("address" = String, Path, description = "Alias email address")),
    security(("bearer_auth" = [])),
    request_body = UpdateAlias,
    responses(
        (status = 200, description = "Alias updated", body = ApiResponse<AliasResponse>),
        (status = 404, description = "Alias not found", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn update(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(body): Json<UpdateAlias>,
) -> Result<Json<ApiResponse<AliasResponse>>, ApiError> {
    let email = EmailAddress::try_from(address)
        .map_err(|e| ApiError::Validation(format!("invalid address: {e}")))?;
    let alias = state.aliases.update(&email, &body).await?;
    Ok(Json(ApiResponse::new(alias)))
}

/// DELETE /api/v1/aliases/:address
#[utoipa::path(
    delete,
    path = "/api/v1/aliases/{address}",
    tag = "aliases",
    operation_id = "delete_alias",
    params(("address" = String, Path, description = "Alias email address")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Alias deleted"),
        (status = 404, description = "Alias not found", body = ProblemDetails),
    )
)]
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

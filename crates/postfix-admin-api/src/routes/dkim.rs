//! DKIM key and signing endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::RequireSuperAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{
    CreateDkimKey, CreateDkimSigning, DkimKeyResponse, DkimSigningResponse,
};
use postfix_admin_core::types::DomainName;

/// GET /api/v1/domains/:domain/dkim/keys
#[utoipa::path(
    get,
    path = "/api/v1/domains/{domain}/dkim/keys",
    tag = "dkim",
    operation_id = "list_dkim_keys",
    params(("domain" = String, Path, description = "Domain name")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of DKIM keys", body = ApiListResponse<DkimKeyResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
        (status = 403, description = "Not a superadmin", body = ProblemDetails),
    )
)]
pub async fn list_keys(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<Json<ApiListResponse<DkimKeyResponse>>, ApiError> {
    let domain_name = DomainName::try_from(domain)
        .map_err(|e| ApiError::Validation(format!("invalid domain: {e}")))?;
    let keys = state.dkim.find_keys_by_domain(&domain_name).await?;
    Ok(Json(ApiListResponse::from_vec(keys)))
}

/// POST /api/v1/dkim/keys
#[utoipa::path(
    post,
    path = "/api/v1/dkim/keys",
    tag = "dkim",
    operation_id = "create_dkim_key",
    security(("bearer_auth" = [])),
    request_body = CreateDkimKey,
    responses(
        (status = 201, description = "DKIM key created", body = ApiResponse<DkimKeyResponse>),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create_key(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateDkimKey>,
) -> Result<(StatusCode, Json<ApiResponse<DkimKeyResponse>>), ApiError> {
    let key = state.dkim.create_key(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(key))))
}

/// DELETE /api/v1/dkim/keys/:id
#[utoipa::path(
    delete,
    path = "/api/v1/dkim/keys/{id}",
    tag = "dkim",
    operation_id = "delete_dkim_key",
    params(("id" = Uuid, Path, description = "DKIM key ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "DKIM key deleted"),
        (status = 404, description = "Key not found", body = ProblemDetails),
    )
)]
pub async fn delete_key(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.dkim.delete_key(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/dkim/keys/:id/signings
#[utoipa::path(
    get,
    path = "/api/v1/dkim/keys/{id}/signings",
    tag = "dkim",
    operation_id = "list_dkim_signings",
    params(("id" = Uuid, Path, description = "DKIM key ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of DKIM signings", body = ApiListResponse<DkimSigningResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
    )
)]
pub async fn list_signings(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiListResponse<DkimSigningResponse>>, ApiError> {
    let signings = state.dkim.find_signings_by_key_id(id).await?;
    Ok(Json(ApiListResponse::from_vec(signings)))
}

/// POST /api/v1/dkim/signings
#[utoipa::path(
    post,
    path = "/api/v1/dkim/signings",
    tag = "dkim",
    operation_id = "create_dkim_signing",
    security(("bearer_auth" = [])),
    request_body = CreateDkimSigning,
    responses(
        (status = 201, description = "DKIM signing created", body = ApiResponse<DkimSigningResponse>),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create_signing(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateDkimSigning>,
) -> Result<(StatusCode, Json<ApiResponse<DkimSigningResponse>>), ApiError> {
    let signing = state.dkim.create_signing(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(signing))))
}

/// DELETE /api/v1/dkim/signings/:id
#[utoipa::path(
    delete,
    path = "/api/v1/dkim/signings/{id}",
    tag = "dkim",
    operation_id = "delete_dkim_signing",
    params(("id" = Uuid, Path, description = "DKIM signing ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "DKIM signing deleted"),
        (status = 404, description = "Signing not found", body = ProblemDetails),
    )
)]
pub async fn delete_signing(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.dkim.delete_signing(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

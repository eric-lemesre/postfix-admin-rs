//! Admin CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::RequireSuperAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{AdminResponse, CreateAdmin, UpdateAdmin};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::EmailAddress;

/// GET /api/v1/admins
#[utoipa::path(
    get,
    path = "/api/v1/admins",
    tag = "admins",
    operation_id = "list_admins",
    params(PageRequest),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of admins", body = ApiListResponse<AdminResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
        (status = 403, description = "Not a superadmin", body = ProblemDetails),
    )
)]
pub async fn list(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Query(page): Query<PageRequest>,
) -> Result<Json<ApiListResponse<AdminResponse>>, ApiError> {
    let result = state.admins.find_all(&page).await?;
    Ok(Json(result.into()))
}

/// GET /api/v1/admins/:username
#[utoipa::path(
    get,
    path = "/api/v1/admins/{username}",
    tag = "admins",
    operation_id = "get_admin",
    params(("username" = String, Path, description = "Admin email address")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Admin found", body = ApiResponse<AdminResponse>),
        (status = 404, description = "Admin not found", body = ProblemDetails),
    )
)]
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
#[utoipa::path(
    post,
    path = "/api/v1/admins",
    tag = "admins",
    operation_id = "create_admin",
    security(("bearer_auth" = [])),
    request_body = CreateAdmin,
    responses(
        (status = 201, description = "Admin created", body = ApiResponse<AdminResponse>),
        (status = 409, description = "Admin already exists", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateAdmin>,
) -> Result<(StatusCode, Json<ApiResponse<AdminResponse>>), ApiError> {
    let admin = state.admins.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(admin))))
}

/// PUT /api/v1/admins/:username
#[utoipa::path(
    put,
    path = "/api/v1/admins/{username}",
    tag = "admins",
    operation_id = "update_admin",
    params(("username" = String, Path, description = "Admin email address")),
    security(("bearer_auth" = [])),
    request_body = UpdateAdmin,
    responses(
        (status = 200, description = "Admin updated", body = ApiResponse<AdminResponse>),
        (status = 404, description = "Admin not found", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
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
#[utoipa::path(
    delete,
    path = "/api/v1/admins/{username}",
    tag = "admins",
    operation_id = "delete_admin",
    params(("username" = String, Path, description = "Admin email address")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Admin deleted"),
        (status = 404, description = "Admin not found", body = ProblemDetails),
    )
)]
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

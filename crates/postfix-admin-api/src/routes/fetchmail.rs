//! Fetchmail CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::AuthAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{CreateFetchmail, FetchmailResponse, UpdateFetchmail};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::EmailAddress;

/// GET /api/v1/mailboxes/:username/fetchmail
#[utoipa::path(
    get,
    path = "/api/v1/mailboxes/{username}/fetchmail",
    tag = "fetchmail",
    operation_id = "list_fetchmail",
    params(
        ("username" = String, Path, description = "Mailbox email address"),
        PageRequest,
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of fetchmail entries", body = ApiListResponse<FetchmailResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
    )
)]
pub async fn list(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(page): Query<PageRequest>,
) -> Result<Json<ApiListResponse<FetchmailResponse>>, ApiError> {
    let email = EmailAddress::try_from(username)
        .map_err(|e| ApiError::Validation(format!("invalid email: {e}")))?;
    let result = state.fetchmail.find_by_mailbox(&email, &page).await?;
    Ok(Json(result.into()))
}

/// GET /api/v1/fetchmail/:id
#[utoipa::path(
    get,
    path = "/api/v1/fetchmail/{id}",
    tag = "fetchmail",
    operation_id = "get_fetchmail",
    params(("id" = Uuid, Path, description = "Fetchmail entry ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Fetchmail entry found", body = ApiResponse<FetchmailResponse>),
        (status = 404, description = "Entry not found", body = ProblemDetails),
    )
)]
pub async fn get(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<FetchmailResponse>>, ApiError> {
    let entry = state
        .fetchmail
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("fetchmail entry '{id}'")))?;
    Ok(Json(ApiResponse::new(entry)))
}

/// POST /api/v1/fetchmail
#[utoipa::path(
    post,
    path = "/api/v1/fetchmail",
    tag = "fetchmail",
    operation_id = "create_fetchmail",
    security(("bearer_auth" = [])),
    request_body = CreateFetchmail,
    responses(
        (status = 201, description = "Fetchmail entry created", body = ApiResponse<FetchmailResponse>),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateFetchmail>,
) -> Result<(StatusCode, Json<ApiResponse<FetchmailResponse>>), ApiError> {
    let entry = state.fetchmail.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(entry))))
}

/// PUT /api/v1/fetchmail/:id
#[utoipa::path(
    put,
    path = "/api/v1/fetchmail/{id}",
    tag = "fetchmail",
    operation_id = "update_fetchmail",
    params(("id" = Uuid, Path, description = "Fetchmail entry ID")),
    security(("bearer_auth" = [])),
    request_body = UpdateFetchmail,
    responses(
        (status = 200, description = "Fetchmail entry updated", body = ApiResponse<FetchmailResponse>),
        (status = 404, description = "Entry not found", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn update(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFetchmail>,
) -> Result<Json<ApiResponse<FetchmailResponse>>, ApiError> {
    let entry = state.fetchmail.update(id, &body).await?;
    Ok(Json(ApiResponse::new(entry)))
}

/// DELETE /api/v1/fetchmail/:id
#[utoipa::path(
    delete,
    path = "/api/v1/fetchmail/{id}",
    tag = "fetchmail",
    operation_id = "delete_fetchmail",
    params(("id" = Uuid, Path, description = "Fetchmail entry ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Fetchmail entry deleted"),
        (status = 404, description = "Entry not found", body = ProblemDetails),
    )
)]
pub async fn delete(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.fetchmail.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

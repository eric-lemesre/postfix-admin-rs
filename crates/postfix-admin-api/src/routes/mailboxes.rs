//! Mailbox CRUD endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::AuthAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{CreateMailbox, MailboxResponse, UpdateMailbox};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::{DomainName, EmailAddress};

/// GET /api/v1/domains/:domain/mailboxes
#[utoipa::path(
    get,
    path = "/api/v1/domains/{domain}/mailboxes",
    tag = "mailboxes",
    operation_id = "list_mailboxes",
    params(
        ("domain" = String, Path, description = "Domain name"),
        PageRequest,
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of mailboxes", body = ApiListResponse<MailboxResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
    )
)]
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
#[utoipa::path(
    get,
    path = "/api/v1/mailboxes/{username}",
    tag = "mailboxes",
    operation_id = "get_mailbox",
    params(("username" = String, Path, description = "Mailbox email address")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Mailbox found", body = ApiResponse<MailboxResponse>),
        (status = 404, description = "Mailbox not found", body = ProblemDetails),
    )
)]
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
#[utoipa::path(
    post,
    path = "/api/v1/mailboxes",
    tag = "mailboxes",
    operation_id = "create_mailbox",
    security(("bearer_auth" = [])),
    request_body = CreateMailbox,
    responses(
        (status = 201, description = "Mailbox created", body = ApiResponse<MailboxResponse>),
        (status = 409, description = "Mailbox already exists", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
pub async fn create(
    _admin: AuthAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateMailbox>,
) -> Result<(StatusCode, Json<ApiResponse<MailboxResponse>>), ApiError> {
    let mailbox = state.mailboxes.create(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(mailbox))))
}

/// PUT /api/v1/mailboxes/:username
#[utoipa::path(
    put,
    path = "/api/v1/mailboxes/{username}",
    tag = "mailboxes",
    operation_id = "update_mailbox",
    params(("username" = String, Path, description = "Mailbox email address")),
    security(("bearer_auth" = [])),
    request_body = UpdateMailbox,
    responses(
        (status = 200, description = "Mailbox updated", body = ApiResponse<MailboxResponse>),
        (status = 404, description = "Mailbox not found", body = ProblemDetails),
        (status = 422, description = "Validation error", body = ProblemDetails),
    )
)]
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
#[utoipa::path(
    delete,
    path = "/api/v1/mailboxes/{username}",
    tag = "mailboxes",
    operation_id = "delete_mailbox",
    params(("username" = String, Path, description = "Mailbox email address")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Mailbox deleted"),
        (status = 404, description = "Mailbox not found", body = ProblemDetails),
    )
)]
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

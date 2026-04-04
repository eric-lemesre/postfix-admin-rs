//! Audit log endpoints.

use axum::extract::{Query, State};
use axum::Json;
use utoipa::IntoParams;

use crate::error::{ApiError, ProblemDetails};
use crate::extractors::RequireSuperAdmin;
use crate::response::ApiListResponse;
use crate::state::AppState;
use postfix_admin_core::dto::{LogFilter, LogResponse};
use postfix_admin_core::pagination::PageRequest;

/// Query parameters combining log filter and pagination.
#[derive(Debug, serde::Deserialize, IntoParams)]
pub struct LogQuery {
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    25
}

/// GET /api/v1/logs
#[utoipa::path(
    get,
    path = "/api/v1/logs",
    tag = "logs",
    operation_id = "list_logs",
    params(LogQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of audit logs", body = ApiListResponse<LogResponse>),
        (status = 401, description = "Not authenticated", body = ProblemDetails),
        (status = 403, description = "Not a superadmin", body = ProblemDetails),
    )
)]
pub async fn list(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<Json<ApiListResponse<LogResponse>>, ApiError> {
    let filter = LogFilter {
        domain: query.domain,
        username: query.username,
        action: query.action,
        from: None,
        until: None,
    };
    let page = PageRequest::new(query.page, query.per_page);
    let result = state.logs.find_all(&filter, &page).await?;
    Ok(Json(result.into()))
}

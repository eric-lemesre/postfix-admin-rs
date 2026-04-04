//! DKIM key and signing endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::RequireSuperAdmin;
use crate::response::{ApiListResponse, ApiResponse};
use crate::state::AppState;
use postfix_admin_core::dto::{
    CreateDkimKey, CreateDkimSigning, DkimKeyResponse, DkimSigningResponse,
};
use postfix_admin_core::types::DomainName;

/// GET /api/v1/domains/:domain/dkim/keys
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
pub async fn create_key(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateDkimKey>,
) -> Result<(StatusCode, Json<ApiResponse<DkimKeyResponse>>), ApiError> {
    let key = state.dkim.create_key(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(key))))
}

/// DELETE /api/v1/dkim/keys/:id
pub async fn delete_key(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.dkim.delete_key(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/dkim/keys/:id/signings
pub async fn list_signings(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiListResponse<DkimSigningResponse>>, ApiError> {
    let signings = state.dkim.find_signings_by_key_id(id).await?;
    Ok(Json(ApiListResponse::from_vec(signings)))
}

/// POST /api/v1/dkim/signings
pub async fn create_signing(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Json(body): Json<CreateDkimSigning>,
) -> Result<(StatusCode, Json<ApiResponse<DkimSigningResponse>>), ApiError> {
    let signing = state.dkim.create_signing(&body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(signing))))
}

/// DELETE /api/v1/dkim/signings/:id
pub async fn delete_signing(
    _admin: RequireSuperAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.dkim.delete_signing(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

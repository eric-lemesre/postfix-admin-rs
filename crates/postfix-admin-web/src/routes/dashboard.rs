//! Dashboard page.

use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use tower_sessions::Session;

use crate::session;
use crate::state::WebState;
use crate::templates::{self, DashboardTemplate};

/// GET /dashboard
pub async fn index(State(state): State<WebState>, session: Session) -> Response {
    let Some(admin_username) = session::get_admin_username(&session).await else {
        return Redirect::to("/login").into_response();
    };

    let domain_count = state.domains.count().await.unwrap_or(0);
    let flash_message = session::take_flash(&session).await;
    let flash_error = session::take_flash_error(&session).await;

    templates::render(&DashboardTemplate {
        admin_username,
        flash_message,
        flash_error,
        domain_count,
    })
}

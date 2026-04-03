//! Web route modules and router construction.

pub mod auth;
pub mod dashboard;
pub mod domains;

use axum::response::Redirect;
use axum::routing::{get, post};
use axum::Router;

use crate::state::WebState;

/// Build the web UI router.
pub fn web_router() -> Router<WebState> {
    Router::new()
        // Root redirect
        .route("/", get(|| async { Redirect::to("/dashboard") }))
        // Auth
        .route("/login", get(auth::login_form).post(auth::login_post))
        .route("/logout", post(auth::logout))
        // Dashboard
        .route("/dashboard", get(dashboard::index))
        // Domains
        .route("/domains", get(domains::list))
        .route("/domains/new", get(domains::new_form).post(domains::create))
        .route(
            "/domains/{name}/edit",
            get(domains::edit_form).post(domains::update),
        )
        .route("/domains/{name}/delete", post(domains::delete))
}

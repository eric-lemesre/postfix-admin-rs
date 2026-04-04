//! API route modules and router construction.

pub mod admins;
pub mod alias_domains;
pub mod aliases;
pub mod auth;
pub mod dkim;
pub mod domains;
pub mod fetchmail;
pub mod logs;
pub mod mailboxes;
pub mod vacations;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::middleware::rate_limit::rate_limit_middleware;
use crate::state::AppState;

/// Build the `/api/v1` router with all REST endpoints.
///
/// The `state` parameter is needed to wire the rate limiting middleware
/// via `from_fn_with_state`.
pub fn api_router(state: AppState) -> Router {
    Router::new()
        // Auth
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        // Domains
        .route("/domains", get(domains::list).post(domains::create))
        .route(
            "/domains/{name}",
            get(domains::get)
                .put(domains::update)
                .delete(domains::delete),
        )
        // Mailboxes (nested under domains for listing)
        .route("/domains/{domain}/mailboxes", get(mailboxes::list))
        .route("/mailboxes", post(mailboxes::create))
        .route(
            "/mailboxes/{username}",
            get(mailboxes::get)
                .put(mailboxes::update)
                .delete(mailboxes::delete),
        )
        // Aliases (nested under domains for listing)
        .route("/domains/{domain}/aliases", get(aliases::list))
        .route("/aliases", post(aliases::create))
        .route(
            "/aliases/{address}",
            get(aliases::get)
                .put(aliases::update)
                .delete(aliases::delete),
        )
        // Admins
        .route("/admins", get(admins::list).post(admins::create))
        .route(
            "/admins/{username}",
            get(admins::get).put(admins::update).delete(admins::delete),
        )
        // Vacations
        .route(
            "/mailboxes/{username}/vacation",
            get(vacations::get)
                .put(vacations::upsert)
                .delete(vacations::delete),
        )
        // Fetchmail
        .route("/mailboxes/{username}/fetchmail", get(fetchmail::list))
        .route("/fetchmail", post(fetchmail::create))
        .route(
            "/fetchmail/{id}",
            get(fetchmail::get)
                .put(fetchmail::update)
                .delete(fetchmail::delete),
        )
        // DKIM
        .route("/domains/{domain}/dkim/keys", get(dkim::list_keys))
        .route("/dkim/keys", post(dkim::create_key))
        .route("/dkim/keys/{id}", delete(dkim::delete_key))
        .route("/dkim/keys/{id}/signings", get(dkim::list_signings))
        .route("/dkim/signings", post(dkim::create_signing))
        .route("/dkim/signings/{id}", delete(dkim::delete_signing))
        // Alias domains
        .route(
            "/domains/{domain}/alias-domains",
            get(alias_domains::list_by_target),
        )
        .route("/alias-domains", post(alias_domains::create))
        .route(
            "/alias-domains/{alias}",
            get(alias_domains::get).delete(alias_domains::delete),
        )
        // Logs
        .route("/logs", get(logs::list))
        // API rate limiting (pass-through if not configured)
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .with_state(state)
}

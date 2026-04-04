//! REST and gRPC API for postfix-admin-rs.
//!
//! Provides the HTTP REST API (axum) with JWT authentication,
//! RFC 7807 error handling, and CRUD endpoints for all entities.

// Route handlers all return `Result` — doc `# Errors` sections are not
// useful for axum handlers whose error semantics are defined by `IntoResponse`.
#![allow(clippy::missing_errors_doc)]

pub mod error;
pub mod extractors;
#[allow(clippy::needless_for_each)]
pub mod openapi;
pub mod response;
pub mod routes;
pub mod state;

pub use openapi::ApiDoc;
pub use routes::api_router;
pub use state::AppState;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_loads() {
        // Smoke test: the crate compiles and links correctly.
    }
}

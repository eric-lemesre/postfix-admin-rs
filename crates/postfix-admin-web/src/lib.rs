//! Web interface for postfix-admin-rs.
//!
//! Server-rendered HTML using Askama templates with HTMX
//! for dynamic interactions and Tailwind CSS for styling.

pub mod routes;
pub mod session;
pub mod state;
pub mod templates;

pub use routes::web_router;
pub use state::WebState;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_loads() {
        // Smoke test: the crate compiles and links correctly.
    }
}

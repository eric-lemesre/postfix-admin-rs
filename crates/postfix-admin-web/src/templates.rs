//! Askama template structs for server-rendered HTML pages.

use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use postfix_admin_core::dto::DomainResponse;

/// Render an Askama template into an axum `Response`.
pub fn render(template: &impl Template) -> Response {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "template rendering failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "template error").into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub admin_username: String,
    pub flash_message: Option<String>,
    pub flash_error: Option<String>,
    pub domain_count: i64,
}

#[derive(Template)]
#[template(path = "domains/list.html")]
pub struct DomainListTemplate {
    pub admin_username: String,
    pub flash_message: Option<String>,
    pub flash_error: Option<String>,
    pub domains: Vec<DomainResponse>,
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Template)]
#[template(path = "domains/form.html")]
pub struct DomainFormTemplate {
    pub admin_username: String,
    pub flash_message: Option<String>,
    pub flash_error: Option<String>,
    pub is_edit: bool,
    pub domain_name: String,
    pub description: String,
    pub aliases: i32,
    pub mailboxes: i32,
    pub active: bool,
    pub error: Option<String>,
}

//! Domain management web pages.

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use tower_sessions::Session;

use crate::session;
use crate::state::WebState;
use crate::templates::{self, DomainFormTemplate, DomainListTemplate};
use postfix_admin_core::dto::{CreateDomain, UpdateDomain};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::types::DomainName;

#[derive(serde::Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u32,
}

fn default_page() -> u32 {
    1
}

/// GET /domains
pub async fn list(
    State(state): State<WebState>,
    session: Session,
    Query(query): Query<PageQuery>,
) -> Response {
    let Some(admin_username) = session::get_admin_username(&session).await else {
        return Redirect::to("/login").into_response();
    };

    let page = PageRequest::new(query.page, 25);
    let Ok(result) = state.domains.find_all(&page).await else {
        return Redirect::to("/dashboard").into_response();
    };

    let flash_message = session::take_flash(&session).await;
    let flash_error = session::take_flash_error(&session).await;
    let total_pages = result.total_pages();

    templates::render(&DomainListTemplate {
        admin_username,
        flash_message,
        flash_error,
        domains: result.items,
        page: result.page,
        total_pages,
    })
}

/// GET /domains/new
pub async fn new_form(session: Session) -> Response {
    let Some(admin_username) = session::get_admin_username(&session).await else {
        return Redirect::to("/login").into_response();
    };

    templates::render(&DomainFormTemplate {
        admin_username,
        flash_message: None,
        flash_error: None,
        is_edit: false,
        domain_name: String::new(),
        description: String::new(),
        aliases: 0,
        mailboxes: 0,
        active: true,
        error: None,
    })
}

#[derive(serde::Deserialize)]
pub struct DomainForm {
    pub domain: Option<String>,
    pub description: Option<String>,
    pub aliases: Option<i32>,
    pub mailboxes: Option<i32>,
    pub active: Option<String>,
}

/// POST /domains/new
pub async fn create(
    State(state): State<WebState>,
    session: Session,
    Form(form): Form<DomainForm>,
) -> Response {
    if session::get_admin_username(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }

    let domain_str = form.domain.unwrap_or_default();
    let Ok(domain_name) = DomainName::try_from(domain_str) else {
        session::set_flash_error(&session, "Invalid domain name").await;
        return Redirect::to("/domains/new").into_response();
    };

    let dto = CreateDomain {
        domain: domain_name,
        description: form.description,
        aliases: form.aliases,
        mailboxes: form.mailboxes,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: Some(form.active.is_some()),
    };

    match state.domains.create(&dto).await {
        Ok(_) => {
            session::set_flash(&session, "Domain created successfully").await;
            Redirect::to("/domains").into_response()
        }
        Err(e) => {
            session::set_flash_error(&session, &e.to_string()).await;
            Redirect::to("/domains/new").into_response()
        }
    }
}

/// GET /domains/:name/edit
pub async fn edit_form(
    State(state): State<WebState>,
    session: Session,
    Path(name): Path<String>,
) -> Response {
    let Some(admin_username) = session::get_admin_username(&session).await else {
        return Redirect::to("/login").into_response();
    };

    let Ok(domain_name) = DomainName::try_from(name) else {
        return Redirect::to("/domains").into_response();
    };

    let Ok(Some(domain)) = state.domains.find_by_name(&domain_name).await else {
        return Redirect::to("/domains").into_response();
    };

    let flash_error = session::take_flash_error(&session).await;

    templates::render(&DomainFormTemplate {
        admin_username,
        flash_message: None,
        flash_error,
        is_edit: true,
        domain_name: domain.domain.to_string(),
        description: domain.description,
        aliases: domain.aliases,
        mailboxes: domain.mailboxes,
        active: domain.active,
        error: None,
    })
}

/// POST /domains/:name/edit
pub async fn update(
    State(state): State<WebState>,
    session: Session,
    Path(name): Path<String>,
    Form(form): Form<DomainForm>,
) -> Response {
    if session::get_admin_username(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }

    let Ok(domain_name) = DomainName::try_from(name) else {
        return Redirect::to("/domains").into_response();
    };

    let dto = UpdateDomain {
        description: form.description,
        aliases: form.aliases,
        mailboxes: form.mailboxes,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: Some(form.active.is_some()),
    };

    match state.domains.update(&domain_name, &dto).await {
        Ok(_) => {
            session::set_flash(&session, "Domain updated successfully").await;
            Redirect::to("/domains").into_response()
        }
        Err(e) => {
            session::set_flash_error(&session, &e.to_string()).await;
            Redirect::to(&format!("/domains/{domain_name}/edit")).into_response()
        }
    }
}

/// POST /domains/:name/delete
pub async fn delete(
    State(state): State<WebState>,
    session: Session,
    Path(name): Path<String>,
) -> Response {
    if session::get_admin_username(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }

    let Ok(domain_name) = DomainName::try_from(name) else {
        return Redirect::to("/domains").into_response();
    };

    match state.domains.delete(&domain_name).await {
        Ok(()) => {
            session::set_flash(&session, "Domain deleted").await;
        }
        Err(e) => {
            session::set_flash_error(&session, &e.to_string()).await;
        }
    }

    Redirect::to("/domains").into_response()
}

//! Web authentication routes: login form, login POST, logout.

use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use tower_sessions::Session;

use crate::session;
use crate::state::WebState;
use crate::templates::{self, LoginTemplate};

/// GET /login — show login form.
pub async fn login_form(session: Session) -> Response {
    // If already logged in, redirect to dashboard.
    if session::get_admin_username(&session).await.is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    templates::render(&LoginTemplate { error: None })
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// POST /login — process login.
pub async fn login_post(
    State(state): State<WebState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Response {
    let Ok(username) = postfix_admin_core::EmailAddress::try_from(form.username) else {
        return templates::render(&LoginTemplate {
            error: Some("Invalid email address".to_string()),
        });
    };

    let Ok(Some(admin)) = state.admins.find_by_username(&username).await else {
        return templates::render(&LoginTemplate {
            error: Some("Invalid credentials".to_string()),
        });
    };

    if !admin.active {
        return templates::render(&LoginTemplate {
            error: Some("Account is inactive".to_string()),
        });
    }

    // TODO: Verify password hash from database (requires AdminRepository extension)
    // For now, we accept the login if the admin exists and is active.
    let _ = form.password;

    session::set_admin(&session, username.as_ref(), admin.superadmin).await;
    Redirect::to("/dashboard").into_response()
}

/// POST /logout
pub async fn logout(session: Session) -> Redirect {
    session::clear_session(&session).await;
    Redirect::to("/login")
}

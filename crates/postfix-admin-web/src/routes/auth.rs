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
    let csrf_token = session::get_csrf_token(&session).await;
    templates::render(&LoginTemplate {
        error: None,
        csrf_token,
    })
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub csrf_token: String,
}

/// POST /login — process login.
pub async fn login_post(
    State(state): State<WebState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Response {
    // Verify CSRF token
    if !session::verify_csrf(&session, &form.csrf_token).await {
        let csrf_token = session::get_csrf_token(&session).await;
        return templates::render(&LoginTemplate {
            error: Some("Invalid CSRF token. Please try again.".to_string()),
            csrf_token,
        });
    }

    let Ok(username) = postfix_admin_core::EmailAddress::try_from(form.username) else {
        let csrf_token = session::get_csrf_token(&session).await;
        return templates::render(&LoginTemplate {
            error: Some("Invalid email address".to_string()),
            csrf_token,
        });
    };

    let Ok(Some(admin)) = state.admins.find_by_username(&username).await else {
        let csrf_token = session::get_csrf_token(&session).await;
        return templates::render(&LoginTemplate {
            error: Some("Invalid credentials".to_string()),
            csrf_token,
        });
    };

    if !admin.active {
        let csrf_token = session::get_csrf_token(&session).await;
        return templates::render(&LoginTemplate {
            error: Some("Account is inactive".to_string()),
            csrf_token,
        });
    }

    // Check rate limiting
    let client_ip = "unknown".to_string(); // TODO: extract from request extensions
    if let Err(e) = state.rate_limiter.check_allowed(&client_ip) {
        let csrf_token = session::get_csrf_token(&session).await;
        return templates::render(&LoginTemplate {
            error: Some(e.to_string()),
            csrf_token,
        });
    }

    // Verify password hash from database
    let Ok(Some(password_hash)) = state.admins.find_password_hash(&username).await else {
        let csrf_token = session::get_csrf_token(&session).await;
        return templates::render(&LoginTemplate {
            error: Some("Invalid credentials".to_string()),
            csrf_token,
        });
    };

    let verified = postfix_admin_auth::verify_password(&form.password, &password_hash);
    if !matches!(verified, Ok(true)) {
        state.rate_limiter.record_failure(&client_ip);
        let csrf_token = session::get_csrf_token(&session).await;
        return templates::render(&LoginTemplate {
            error: Some("Invalid credentials".to_string()),
            csrf_token,
        });
    }

    // TODO: TOTP verification if admin.totp_enabled

    // Record successful login and clear rate limit
    state.rate_limiter.record_success(&client_ip);

    // Regenerate session ID to prevent session fixation attacks
    session::regenerate_session(&session).await;
    session::set_admin(&session, username.as_ref(), admin.superadmin).await;
    Redirect::to("/dashboard").into_response()
}

/// POST /logout
pub async fn logout(session: Session) -> Redirect {
    session::clear_session(&session).await;
    Redirect::to("/login")
}

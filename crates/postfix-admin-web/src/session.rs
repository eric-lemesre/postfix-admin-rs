//! Session helpers for the web interface.

use tower_sessions::Session;

const SESSION_KEY_USERNAME: &str = "username";
const SESSION_KEY_SUPERADMIN: &str = "superadmin";
const SESSION_KEY_FLASH: &str = "flash_message";
const SESSION_KEY_FLASH_ERROR: &str = "flash_error";

/// Get the authenticated admin username from the session.
pub async fn get_admin_username(session: &Session) -> Option<String> {
    session
        .get::<String>(SESSION_KEY_USERNAME)
        .await
        .ok()
        .flatten()
}

/// Set the authenticated admin in the session.
pub async fn set_admin(session: &Session, username: &str, superadmin: bool) {
    let _ = session
        .insert(SESSION_KEY_USERNAME, username.to_string())
        .await;
    let _ = session.insert(SESSION_KEY_SUPERADMIN, superadmin).await;
}

/// Clear the session (logout).
pub async fn clear_session(session: &Session) {
    session.flush().await.ok();
}

/// Set a flash message for the next page load.
pub async fn set_flash(session: &Session, message: &str) {
    let _ = session.insert(SESSION_KEY_FLASH, message.to_string()).await;
}

/// Set a flash error for the next page load.
pub async fn set_flash_error(session: &Session, error: &str) {
    let _ = session
        .insert(SESSION_KEY_FLASH_ERROR, error.to_string())
        .await;
}

/// Get and consume the flash message.
pub async fn take_flash(session: &Session) -> Option<String> {
    let msg = session
        .get::<String>(SESSION_KEY_FLASH)
        .await
        .ok()
        .flatten();
    if msg.is_some() {
        let _ = session.remove::<String>(SESSION_KEY_FLASH).await;
    }
    msg
}

/// Get and consume the flash error.
pub async fn take_flash_error(session: &Session) -> Option<String> {
    let msg = session
        .get::<String>(SESSION_KEY_FLASH_ERROR)
        .await
        .ok()
        .flatten();
    if msg.is_some() {
        let _ = session.remove::<String>(SESSION_KEY_FLASH_ERROR).await;
    }
    msg
}

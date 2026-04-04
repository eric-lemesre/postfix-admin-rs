//! Web application state.

use std::sync::Arc;

use postfix_admin_auth::LoginRateLimiter;
use postfix_admin_core::repository::{
    AdminRepository, AliasDomainRepository, AliasRepository, DkimRepository, DomainRepository,
    FetchmailRepository, LogRepository, MailboxRepository, VacationRepository,
};

/// Shared state for web handlers. Uses the same repositories as the API.
#[derive(Clone)]
pub struct WebState {
    pub domains: Arc<dyn DomainRepository>,
    pub mailboxes: Arc<dyn MailboxRepository>,
    pub aliases: Arc<dyn AliasRepository>,
    pub admins: Arc<dyn AdminRepository>,
    pub vacations: Arc<dyn VacationRepository>,
    pub alias_domains: Arc<dyn AliasDomainRepository>,
    pub dkim: Arc<dyn DkimRepository>,
    pub fetchmail: Arc<dyn FetchmailRepository>,
    pub logs: Arc<dyn LogRepository>,
    pub password_scheme: String,
    pub rate_limiter: Arc<LoginRateLimiter>,
}

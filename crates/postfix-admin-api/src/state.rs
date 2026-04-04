//! Application state shared across all API handlers.

use std::sync::Arc;

use postfix_admin_auth::{JwtManager, LoginRateLimiter, MtlsVerifier};
use postfix_admin_core::repository::{
    AdminRepository, AliasDomainRepository, AliasRepository, AppPasswordRepository, DkimRepository,
    DomainRepository, FetchmailRepository, LogRepository, MailboxRepository, VacationRepository,
};

use crate::middleware::ApiRateLimiter;

/// Shared application state injected into handlers via axum's `State` extractor.
#[derive(Clone)]
pub struct AppState {
    pub domains: Arc<dyn DomainRepository>,
    pub mailboxes: Arc<dyn MailboxRepository>,
    pub aliases: Arc<dyn AliasRepository>,
    pub admins: Arc<dyn AdminRepository>,
    pub vacations: Arc<dyn VacationRepository>,
    pub alias_domains: Arc<dyn AliasDomainRepository>,
    pub dkim: Arc<dyn DkimRepository>,
    pub fetchmail: Arc<dyn FetchmailRepository>,
    pub logs: Arc<dyn LogRepository>,
    pub app_passwords: Arc<dyn AppPasswordRepository>,
    pub jwt: Arc<JwtManager>,
    pub password_scheme: String,
    pub rate_limiter: Arc<LoginRateLimiter>,
    pub mtls_verifier: Arc<MtlsVerifier>,
    pub api_rate_limiter: Option<Arc<ApiRateLimiter>>,
}

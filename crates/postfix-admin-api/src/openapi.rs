//! `OpenAPI` 3.1 specification generation.

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::OpenApi;
use utoipa::{Modify, OpenApi as OpenApiDerive};

use crate::error::ProblemDetails;
use crate::response::PaginationMeta;
use crate::routes::{auth, logs};
use postfix_admin_auth::TokenPair;
use postfix_admin_core::dto::{
    AdminResponse, AliasDomainResponse, AliasResponse, CreateAdmin, CreateAlias, CreateAliasDomain,
    CreateDkimKey, CreateDkimSigning, CreateDomain, CreateFetchmail, CreateMailbox,
    DkimKeyResponse, DkimSigningResponse, DomainResponse, FetchmailResponse, LogResponse,
    MailboxResponse, UpdateAdmin, UpdateAlias, UpdateDomain, UpdateFetchmail, UpdateMailbox,
    UpdateVacation, VacationResponse,
};

/// Generate the complete `OpenAPI` specification.
#[allow(clippy::needless_for_each)]
#[derive(OpenApiDerive)]
#[openapi(
    info(
        title = "postfix-admin-rs API",
        version = "0.6.0",
        description = "REST API for managing Postfix virtual domains, mailboxes, and aliases.",
        license(name = "MIT"),
    ),
    paths(
        // Auth
        crate::routes::auth::login,
        crate::routes::auth::refresh,
        crate::routes::auth::logout,
        // Domains
        crate::routes::domains::list,
        crate::routes::domains::get,
        crate::routes::domains::create,
        crate::routes::domains::update,
        crate::routes::domains::delete,
        // Mailboxes
        crate::routes::mailboxes::list,
        crate::routes::mailboxes::get,
        crate::routes::mailboxes::create,
        crate::routes::mailboxes::update,
        crate::routes::mailboxes::delete,
        // Aliases
        crate::routes::aliases::list,
        crate::routes::aliases::get,
        crate::routes::aliases::create,
        crate::routes::aliases::update,
        crate::routes::aliases::delete,
        // Admins
        crate::routes::admins::list,
        crate::routes::admins::get,
        crate::routes::admins::create,
        crate::routes::admins::update,
        crate::routes::admins::delete,
        // Vacations
        crate::routes::vacations::get,
        crate::routes::vacations::upsert,
        crate::routes::vacations::delete,
        // Fetchmail
        crate::routes::fetchmail::list,
        crate::routes::fetchmail::get,
        crate::routes::fetchmail::create,
        crate::routes::fetchmail::update,
        crate::routes::fetchmail::delete,
        // DKIM
        crate::routes::dkim::list_keys,
        crate::routes::dkim::create_key,
        crate::routes::dkim::delete_key,
        crate::routes::dkim::list_signings,
        crate::routes::dkim::create_signing,
        crate::routes::dkim::delete_signing,
        // Alias domains
        crate::routes::alias_domains::list_by_target,
        crate::routes::alias_domains::get,
        crate::routes::alias_domains::create,
        crate::routes::alias_domains::delete,
        // Logs
        crate::routes::logs::list,
    ),
    components(schemas(
        // Auth
        auth::LoginRequest,
        auth::RefreshRequest,
        TokenPair,
        // Domains
        CreateDomain, UpdateDomain, DomainResponse,
        // Mailboxes
        CreateMailbox, UpdateMailbox, MailboxResponse,
        // Aliases
        CreateAlias, UpdateAlias, AliasResponse,
        // Admins
        CreateAdmin, UpdateAdmin, AdminResponse,
        // Vacations
        UpdateVacation, VacationResponse,
        // Fetchmail
        CreateFetchmail, UpdateFetchmail, FetchmailResponse,
        // DKIM
        CreateDkimKey, DkimKeyResponse,
        CreateDkimSigning, DkimSigningResponse,
        // Alias domains
        CreateAliasDomain, AliasDomainResponse,
        // Logs
        LogResponse, logs::LogQuery,
        // Response wrappers
        PaginationMeta,
        ProblemDetails,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "domains", description = "Domain management"),
        (name = "mailboxes", description = "Mailbox management"),
        (name = "aliases", description = "Alias management"),
        (name = "admins", description = "Admin management"),
        (name = "vacations", description = "Vacation auto-responder management"),
        (name = "fetchmail", description = "Fetchmail configuration"),
        (name = "dkim", description = "DKIM key and signing management"),
        (name = "alias-domains", description = "Alias domain management"),
        (name = "logs", description = "Audit log viewing"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        let components = openapi.components.get_or_insert_default();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn openapi_spec_generates_valid_json() {
        let spec = ApiDoc::openapi();
        let json = spec.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn openapi_spec_has_all_tags() {
        let spec = ApiDoc::openapi();
        let tags = spec.tags.unwrap_or_default();
        let tag_names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        assert!(tag_names.contains(&"auth"));
        assert!(tag_names.contains(&"domains"));
        assert!(tag_names.contains(&"mailboxes"));
        assert!(tag_names.contains(&"aliases"));
        assert!(tag_names.contains(&"admins"));
        assert!(tag_names.contains(&"vacations"));
        assert!(tag_names.contains(&"fetchmail"));
        assert!(tag_names.contains(&"dkim"));
        assert!(tag_names.contains(&"alias-domains"));
        assert!(tag_names.contains(&"logs"));
    }

    #[test]
    fn openapi_spec_has_bearer_security_scheme() {
        let spec = ApiDoc::openapi();
        let json = spec.to_json().unwrap_or_default();
        assert!(json.contains("bearer_auth"));
        assert!(json.contains("bearer"));
    }
}

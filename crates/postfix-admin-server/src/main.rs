//! Main server binary for postfix-admin-rs.

use std::sync::Arc;

use postfix_admin_api::{api_router, ApiDoc, AppState};
use postfix_admin_auth::{JwtManager, LoginRateLimiter, MtlsVerifier};
use postfix_admin_core::config::CliOverrides;
use postfix_admin_core::AppConfig;
use postfix_admin_web::{web_router, WebState};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = CliOverrides::default();
    let (config, warnings) = AppConfig::load(&cli)?;

    for w in &warnings {
        tracing::warn!("{w}");
    }

    let db_url = config.database.url.expose();

    if db_url.starts_with("postgresql://") || db_url.starts_with("postgres://") {
        run_with_postgres(&config).await
    } else if db_url.starts_with("mysql://") {
        run_with_mysql(&config).await
    } else {
        anyhow::bail!("Unsupported database URL scheme. Expected postgresql:// or mysql://");
    }
}

fn create_shared_services(config: &AppConfig) -> anyhow::Result<SharedServices> {
    let jwt = create_jwt_manager(config)?;
    let rate_limiter = Arc::new(LoginRateLimiter::new(
        config.auth.max_login_attempts,
        config.auth.lockout_duration,
        config.auth.lockout_duration,
    ));
    let mtls_verifier = Arc::new(MtlsVerifier::new(config.auth.mtls.clone()));

    Ok(SharedServices {
        jwt,
        rate_limiter,
        mtls_verifier,
        password_scheme: config.auth.password_scheme.clone(),
    })
}

struct SharedServices {
    jwt: Arc<JwtManager>,
    rate_limiter: Arc<LoginRateLimiter>,
    mtls_verifier: Arc<MtlsVerifier>,
    password_scheme: String,
}

async fn run_with_postgres(config: &AppConfig) -> anyhow::Result<()> {
    let pool = postfix_admin_db::create_pg_pool(
        config.database.url.expose(),
        config.database.max_connections,
    )
    .await?;

    postfix_admin_db::run_pg_migrations(&pool).await?;

    let services = create_shared_services(config)?;

    let api_state = AppState {
        domains: Arc::new(postfix_admin_db::PgDomainRepository::new(pool.clone())),
        mailboxes: Arc::new(postfix_admin_db::PgMailboxRepository::new(pool.clone())),
        aliases: Arc::new(postfix_admin_db::PgAliasRepository::new(pool.clone())),
        admins: Arc::new(postfix_admin_db::PgAdminRepository::new(pool.clone())),
        vacations: Arc::new(postfix_admin_db::PgVacationRepository::new(pool.clone())),
        alias_domains: Arc::new(postfix_admin_db::PgAliasDomainRepository::new(pool.clone())),
        dkim: Arc::new(postfix_admin_db::PgDkimRepository::new(pool.clone())),
        fetchmail: Arc::new(postfix_admin_db::PgFetchmailRepository::new(pool.clone())),
        logs: Arc::new(postfix_admin_db::PgLogRepository::new(pool.clone())),
        app_passwords: Arc::new(postfix_admin_db::PgAppPasswordRepository::new(pool)),
        jwt: Arc::clone(&services.jwt),
        password_scheme: services.password_scheme.clone(),
        rate_limiter: Arc::clone(&services.rate_limiter),
        mtls_verifier: Arc::clone(&services.mtls_verifier),
    };

    let web_state = WebState {
        domains: Arc::clone(&api_state.domains),
        mailboxes: Arc::clone(&api_state.mailboxes),
        aliases: Arc::clone(&api_state.aliases),
        admins: Arc::clone(&api_state.admins),
        vacations: Arc::clone(&api_state.vacations),
        alias_domains: Arc::clone(&api_state.alias_domains),
        dkim: Arc::clone(&api_state.dkim),
        fetchmail: Arc::clone(&api_state.fetchmail),
        logs: Arc::clone(&api_state.logs),
        password_scheme: services.password_scheme.clone(),
        rate_limiter: Arc::clone(&services.rate_limiter),
    };

    serve(config, api_state, web_state).await
}

async fn run_with_mysql(config: &AppConfig) -> anyhow::Result<()> {
    let pool = postfix_admin_db::create_mysql_pool(
        config.database.url.expose(),
        config.database.max_connections,
    )
    .await?;

    postfix_admin_db::run_mysql_migrations(&pool).await?;

    let services = create_shared_services(config)?;

    let api_state = AppState {
        domains: Arc::new(postfix_admin_db::MysqlDomainRepository::new(pool.clone())),
        mailboxes: Arc::new(postfix_admin_db::MysqlMailboxRepository::new(pool.clone())),
        aliases: Arc::new(postfix_admin_db::MysqlAliasRepository::new(pool.clone())),
        admins: Arc::new(postfix_admin_db::MysqlAdminRepository::new(pool.clone())),
        vacations: Arc::new(postfix_admin_db::MysqlVacationRepository::new(pool.clone())),
        alias_domains: Arc::new(postfix_admin_db::MysqlAliasDomainRepository::new(
            pool.clone(),
        )),
        dkim: Arc::new(postfix_admin_db::MysqlDkimRepository::new(pool.clone())),
        fetchmail: Arc::new(postfix_admin_db::MysqlFetchmailRepository::new(
            pool.clone(),
        )),
        logs: Arc::new(postfix_admin_db::MysqlLogRepository::new(pool.clone())),
        app_passwords: Arc::new(postfix_admin_db::MysqlAppPasswordRepository::new(pool)),
        jwt: Arc::clone(&services.jwt),
        password_scheme: services.password_scheme.clone(),
        rate_limiter: Arc::clone(&services.rate_limiter),
        mtls_verifier: Arc::clone(&services.mtls_verifier),
    };

    let web_state = WebState {
        domains: Arc::clone(&api_state.domains),
        mailboxes: Arc::clone(&api_state.mailboxes),
        aliases: Arc::clone(&api_state.aliases),
        admins: Arc::clone(&api_state.admins),
        vacations: Arc::clone(&api_state.vacations),
        alias_domains: Arc::clone(&api_state.alias_domains),
        dkim: Arc::clone(&api_state.dkim),
        fetchmail: Arc::clone(&api_state.fetchmail),
        logs: Arc::clone(&api_state.logs),
        password_scheme: services.password_scheme,
        rate_limiter: services.rate_limiter,
    };

    serve(config, api_state, web_state).await
}

fn create_jwt_manager(config: &AppConfig) -> anyhow::Result<Arc<JwtManager>> {
    let secret = config.server.secret_key.expose();
    if secret.is_empty() {
        anyhow::bail!("server.secret_key must be set for JWT signing");
    }
    let access_lifetime = i64::try_from(config.auth.jwt.access_token_lifetime).unwrap_or(i64::MAX);
    let refresh_lifetime =
        i64::try_from(config.auth.jwt.refresh_token_lifetime).unwrap_or(i64::MAX);
    Ok(Arc::new(JwtManager::new(
        secret.as_bytes(),
        access_lifetime,
        refresh_lifetime,
    )))
}

async fn serve(config: &AppConfig, api_state: AppState, web_state: WebState) -> anyhow::Result<()> {
    let session_store = MemoryStore::default();
    let session_lifetime = i64::try_from(config.auth.session_lifetime).unwrap_or(3600);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Strict)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(
            session_lifetime,
        )));

    let app = axum::Router::new()
        .nest("/api/v1", api_router().with_state(api_state))
        .merge(SwaggerUi::new("/api/docs").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .merge(web_router().with_state(web_state))
        .layer(session_layer)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let bind = format!("{}:{}", config.server.bind_address, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    tracing::info!(bind = %bind, "postfix-admin-rs server started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("server shut down");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .ok()
            .map(|mut s| async move { s.recv().await });
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

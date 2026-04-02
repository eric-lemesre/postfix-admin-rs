> **Language:** English | [Francais](../fr/guidelines/GUIDELINES-Rust.md)

---
# Rust guidelines — postfix-admin-rs

---
## Table of contents

1. [Code organization](#1-code-organization)
2. [Naming conventions](#2-naming-conventions)
3. [Error handling](#3-error-handling)
4. [Types and patterns](#4-types-and-patterns)
5. [Async and concurrency](#5-async-and-concurrency)
6. [Database and sqlx](#6-database-and-sqlx)
7. [API and handlers](#7-api-and-handlers)
8. [Testing](#8-testing)
9. [Performance](#9-performance)
10. [Security](#10-security)
11. [Documentation](#11-documentation)
12. [Tooling and CI](#12-tooling-and-ci)

---
---
## 1. Code organization

### Crate structure

Each crate in the workspace has a unique responsibility:

```
crates/
├── postfix-admin-core/      # Domain models, traits, shared types
├── postfix-admin-db/        # Data access layer
├── postfix-admin-auth/      # Authentication and authorization
├── postfix-admin-api/       # REST and gRPC handlers
├── postfix-admin-web/       # Web interface (templates, routes)
├── postfix-admin-cli/       # CLI (binary)
└── postfix-admin-server/    # Main server (binary)
```

### Crate dependency rules

```
postfix-admin-server → postfix-admin-web, postfix-admin-api, postfix-admin-cli, postfix-admin-db, postfix-admin-auth, postfix-admin-core
postfix-admin-web    → postfix-admin-core, postfix-admin-db, postfix-admin-auth
postfix-admin-api    → postfix-admin-core, postfix-admin-db, postfix-admin-auth
postfix-admin-cli    → postfix-admin-core, postfix-admin-db, postfix-admin-auth
postfix-admin-auth   → postfix-admin-core, par-db
par-db     → par-core
par-core   → (no internal dependencies)
```

Circular dependencies are forbidden. If two crates need common functionality,
it should be extracted into `par-core`.

### Module organization

```rust
// Order in a file:
// 1. Imports (grouped and ordered)
// 2. Constants
// 3. Types (structs, enums)
// 4. Trait implementations
// 5. impl methods
// 6. Free functions
// 7. Tests (module #[cfg(test)])
```

### Imports

```rust
// Grouped by blocks separated by an empty line:
// 1. std
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// 3. Workspace crates
use par_core::models::Domain;
use par_db::repositories::DomainRepository;

// 4. Local modules
use crate::error::AppError;
use super::middleware::RequireAuth;
```

Prefer specific imports over wildcard imports (`use module::*`).
Exception: well-known crate preludes (`use sqlx::prelude::*`).

---

---
## 2. Naming Conventions

### General

| Element             | Convention                                | Example                     |
|---------------------|-------------------------------------------|-----------------------------|
| Crate               | kebab-case                                | `par-core`                  |
| Module              | snake_case                                | `domain_repository`         |
| Type (struct, enum) | PascalCase                                | `DomainHandler`             |
| Trait               | PascalCase (adjective/verb)               | `Validatable`, `Repository` |
| Function / method   | snake_case                                | `find_by_domain`            |
| Constant            | SCREAMING_SNAKE_CASE                      | `MAX_LOGIN_ATTEMPTS`        |
| Variable            | snake_case                                | `domain_count`              |
| Lifetime            | `'a`, `'de`, `'ctx` (short and descriptive) | `'a`                        |
| Type parameter      | `T`, `E`, or descriptive name             | `T: Repository`             |
| Feature flag        | kebab-case                                | `grpc-support`              |

### Project-specific Naming

| Concept              | Pattern                | Example                             |
|----------------------|------------------------|-------------------------------------|
| Domain model         | Simple name            | `Domain`, `Mailbox`, `Alias`        |
| Creation DTO         | `Create{Entity}`       | `CreateDomain`, `CreateMailbox`     |
| Update DTO           | `Update{Entity}`       | `UpdateDomain`, `UpdateMailbox`     |
| Response DTO         | `{Entity}Response`     | `DomainResponse`, `MailboxResponse` |
| Repository trait     | `{Entity}Repository`   | `DomainRepository`                  |
| Repository impl      | `Pg{Entity}Repository` | `PgDomainRepository`                |
| Handler / Controller | `{entity}_{action}`    | `domain_create`, `mailbox_list`     |
| Error                | `{Module}Error`        | `AuthError`, `DbError`              |
| Middleware           | `Require{Role}`        | `RequireSuperAdmin`                 |

### Constructors

```rust
// Prefer new() for simple cases
impl Domain {
    pub fn new(name: String, description: String) -> Self { ... }
}

// Use the builder pattern for complex types
impl DomainBuilder {
    pub fn new(name: String) -> Self { ... }
    pub fn description(mut self, desc: String) -> Self { ... }
    pub fn build(self) -> Result<Domain, ValidationError> { ... }
}

// Use from/into for conversions
impl From<DomainRow> for Domain { ... }
impl From<CreateDomainRequest> for CreateDomain { ... }
---
## 3. Error Handling

### Error Architecture

```rust
// par-core/src/error.rs — Domain base error

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("domain not found: {0}")]
    NotFound(String),

    #[error("domain already exists: {0}")]
    AlreadyExists(String),

    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("operation not allowed")]
    Unauthorized,

    #[error("limit reached: {message}")]
    LimitReached { message: String },
}
```

```rust
// par-db/src/error.rs — DB layer error

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("record not found")]
    NotFound,

    #[error("unique constraint violation: {0}")]
    UniqueViolation(String),
}
```

```rust
// par-api/src/error.rs — API error converted to HTTP response

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Db(#[from] DbError),

    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Conversion to RFC 7807 Problem Details
        match self {
            ApiError::Domain(DomainError::NotFound(_)) => StatusCode::NOT_FOUND,
            ApiError::Domain(DomainError::Validation(_)) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Auth(AuthError::Unauthorized) => StatusCode::UNAUTHORIZED,
            // ...
        }
    }
}
```

### Rules

- Use `thiserror` for library error types
- Use `anyhow` only in binaries (main, CLI) and tests
- Never use `.unwrap()` or `.expect()` in production code
  - Exceptions: after a check that guarantees success, with a comment
- Propagate errors with `?` rather than `match` when possible
- Errors must contain enough context for diagnosis

```rust
// Good: error with context
Err(DomainError::NotFound(domain_name.to_string()))

// Bad: error without context
Err(DomainError::NotFound("".to_string()))
```

- Do not log AND propagate the same error (choose one or the other)
- Log at the highest level (handler), not in lower layers

---

---
---
## 4. Types and Models

### Domain Models vs DTO

```rust
// Domain model (par-core) — represents business logic
pub struct Domain {
    pub name: DomainName,       // Validated newtype
    pub description: String,
    pub limits: DomainLimits,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Creation DTO (par-core) — incoming data
#[derive(Deserialize, Validate)]
pub struct CreateDomain {
    pub domain: String,
    pub description: Option<String>,
    pub aliases: Option<i32>,
    pub mailboxes: Option<i32>,
    // ...
}

// Response DTO (par-api) — outgoing data
#[derive(Serialize)]
pub struct DomainResponse {
    pub domain: String,
    pub description: String,
    pub aliases_count: i64,
    pub aliases_limit: i32,
    pub active: bool,
}
```

### Newtypes for Type Safety

```rust
// Use newtypes for business identifiers and values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainName(String);

impl DomainName {
    pub fn new(name: &str) -> Result<Self, ValidationError> {
        // RFC 1035 validation
        validate_domain_name(name)?;
        Ok(Self(name.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

// Prevents mixing up DomainName with EmailAddress at compile-time
```

### Validation

```rust
// Use the `validator` crate for declarative validation
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateMailbox {
    #[validate(email)]
    pub username: String,

    #[validate(length(min = 8, max = 256))]
    pub password: String,

    #[validate(length(max = 255))]
    pub name: Option<String>,

    #[validate(range(min = 0))]
    pub quota: Option<i64>,
}

// For complex business validations, implement a custom trait
pub trait BusinessValidation {
    fn validate_business_rules(&self, ctx: &ValidationContext) -> Result<(), ValidationError>;
}
```

### Serialization

```rust
// Derive Serialize/Deserialize only on types that need it
// Domain models should NOT derive Serialize/Deserialize
// Only DTOs do

// Sensitive fields are never serialized
#[derive(Serialize)]
pub struct MailboxResponse {
    pub username: String,
    pub name: String,
    #[serde(skip_serializing)]  // Never in response
    pub password: String,
}

// Use rename_all for JSON consistency
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiResponse<T> {
    pub data: T,
    pub meta: Option<PaginationMeta>,
}
```

---
---
## 5. Async and Concurrency

### Runtime

- Use `tokio` as the async runtime (feature `full`)
- All I/O operations must be async
- Avoid `block_in_place` and `spawn_blocking` except for password hashing

### Rules

```rust
// Async functions should be clearly marked
pub async fn find_domain(pool: &PgPool, name: &str) -> Result<Domain, DbError> {
    // ...
}

// Prefer async traits (async-trait) when necessary
#[async_trait::async_trait]
pub trait DomainRepository: Send + Sync {
    async fn find_by_name(&self, name: &str) -> Result<Option<Domain>, DbError>;
    async fn create(&self, domain: &CreateDomain) -> Result<Domain, DbError>;
    // ...
}

// Password hashing is CPU-intensive → spawn_blocking
pub async fn hash_password(password: &str) -> Result<String, AuthError> {
    let password = password.to_string();
    tokio::task::spawn_blocking(move || {
        argon2::hash_encoded(password.as_bytes(), &salt, &config)
    })
    .await
    .map_err(|_| AuthError::Internal)?
}
```

### Shared State

```rust
// Use Arc for shared state between handlers
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<AppConfig>,
    pub domain_repo: Arc<dyn DomainRepository>,
    pub mailbox_repo: Arc<dyn MailboxRepository>,
    // ...
}

// State is passed via axum::extract::State
async fn list_domains(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<ApiResponse<Vec<DomainResponse>>>, ApiError> {
    // ...
}
```

### Cancellation

- Futures must be cancellation-safe
- Use `tokio::select!` with caution
- SQL transactions must be committed or rolled back even in case of cancellation

---

---
---
## 6. Database and sqlx

### Repository Pattern

```rust
// Trait in par-core (no sqlx dependency)
#[async_trait::async_trait]
pub trait DomainRepository: Send + Sync {
    async fn find_all(&self, params: &ListParams) -> Result<Vec<Domain>, DbError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Domain>, DbError>;
    async fn create(&self, input: &CreateDomain) -> Result<Domain, DbError>;
    async fn update(&self, name: &str, input: &UpdateDomain) -> Result<Domain, DbError>;
    async fn delete(&self, name: &str) -> Result<(), DbError>;
    async fn count_by_domain(&self, name: &str) -> Result<DomainStats, DbError>;
}

// PostgreSQL implementation in par-db
pub struct PgDomainRepository {
    pool: PgPool,
}

#[async_trait::async_trait]
impl DomainRepository for PgDomainRepository {
    async fn find_by_name(&self, name: &str) -> Result<Option<Domain>, DbError> {
        let row = sqlx::query_as!(
            DomainRow,
            "SELECT * FROM domain WHERE domain = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Domain::from))
    }
    // ...
}
```

### SQL Queries

```rust
// ALWAYS use parameterized queries (never format!/SQL concatenation)
// Good
sqlx::query!("SELECT * FROM domain WHERE domain = $1", domain_name)

// Forbidden
sqlx::query(&format!("SELECT * FROM domain WHERE domain = '{}'", domain_name))
```

### Transactions

```rust
// Use transactions for multi-table operations
pub async fn create_mailbox(&self, input: &CreateMailbox) -> Result<Mailbox, DbError> {
    let mut tx = self.pool.begin().await?;

    // Create the mailbox
    let mailbox = sqlx::query_as!(...)
        .fetch_one(&mut *tx)
        .await?;

    // Create the automatic alias
    sqlx::query!(...)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(mailbox)
}
```

### Migrations

- Use `sqlx-cli` for migrations
- Migration file: `migrations/YYYYMMDDHHMMSS_description.sql`
- Each migration must be reversible (provide a `down.sql`)
- Test migrations on the three backends (PG, MySQL, SQLite)

---

---
---
## 7. API and handlers

### Axum handler structure

```rust
/// Lists domains with pagination and filtering.
pub async fn list_domains(
    State(state): State<Arc<AppState>>,
    auth: RequireSuperAdminOrDomainAdmin,
    Query(params): Query<ListDomainsParams>,
) -> Result<Json<ApiResponse<Vec<DomainResponse>>>, ApiError> {
    let domains = state.domain_repo.find_all(&params.into()).await?;
    let total = state.domain_repo.count(&params.into()).await?;

    let response = domains
        .into_iter()
        .map(DomainResponse::from)
        .collect();

    Ok(Json(ApiResponse {
        data: response,
        meta: Some(PaginationMeta { total, ..params.into() }),
    }))
}
```

### Routing

```rust
// Group routes by resource
pub fn domain_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_domains).post(create_domain))
        .route("/{domain}", get(show_domain).put(update_domain).delete(delete_domain))
        .route("/{domain}/active", patch(toggle_domain))
}

// Compose in the main router
pub fn api_v1_routes() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/domains", domain_routes())
        .nest("/mailboxes", mailbox_routes())
        .nest("/aliases", alias_routes())
        .nest("/admins", admin_routes())
        .nest("/auth", auth_routes())
}
```

---
---
## 8. Tests

### Test structure

```rust
// Unit tests — in the same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_name_rejects_invalid_input() {
        assert!(DomainName::new("").is_err());
        assert!(DomainName::new("no-tld").is_err());
        assert!(DomainName::new("-invalid.com").is_err());
    }

    #[test]
    fn domain_name_normalizes_to_lowercase() {
        let name = DomainName::new("Example.COM").unwrap();
        assert_eq!(name.as_str(), "example.com");
    }
}
```

```rust
// Integration tests — in tests/ at the crate root
// tests/domain_repository_test.rs

#[sqlx::test]
async fn create_domain_returns_created_domain(pool: PgPool) {
    let repo = PgDomainRepository::new(pool);
    let input = CreateDomain {
        domain: "test.com".to_string(),
        ..Default::default()
    };

    let result = repo.create(&input).await.unwrap();
    assert_eq!(result.name.as_str(), "test.com");
    assert!(result.active);
}
```

### Testcontainers

```rust
// For integration tests with real databases
use testcontainers::{clients, images::postgres::Postgres};

#[tokio::test]
async fn test_with_real_postgres() {
    let docker = clients::Cli::default();
    let container = docker.run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);

    let pool = PgPool::connect(&format!("postgresql://postgres:postgres@localhost:{port}/postgres"))
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // Tests...
}
```

### Test rules

- Each public function must have at least one test
- Edge cases and error cases must be tested
- Tests must be independent and reproducible
- Use fixtures/factories for test data
- Name tests descriptively: `{action}_{condition}_{expected_result}`

---
---
## 9. Performance

### Allocation

- Prefer `&str` over `String` when ownership isn't necessary
- Use `Cow<'_, str>` when the owned/borrowed choice depends on runtime
- Pre-allocate `Vec` when size is known (`Vec::with_capacity`)
- Avoid unnecessary clones, prefer references

### SQL Queries

- Always paginate results (no `SELECT *` without `LIMIT`)
- Use `fetch_optional` instead of `fetch_one` + error handling
- Counts should use `COUNT(*)` and not load all records
- Index columns used in `WHERE` and `ORDER BY`

### Hashing

- Password hashing is blocking → `spawn_blocking`
- Hide domain DNS validation results
- Use properly sized connection pools

---
---
## 10. Security

### SQL Injection
- Use only sqlx parameterized queries exclusively
- Never concatenate strings in SQL queries

### Passwords
- Never logged, never serialized in API responses
- Timing-safe comparisons
- Stored hash, never plaintext password

### Sessions
- HttpOnly, Secure, SameSite=Strict cookies
- Regeneration after authentication
- CSRF token on every POST form

### User Input
- All inputs are validated and sanitized
- Askama escapes HTML by default (XSS protection)
- URL parameters are validated before use

### Dependencies
- Regular audit with `cargo audit`
- Minimize dependencies
- Prefer well-maintained and audited crates

---
---
## 11. Documentation

### Code documentation

```rust
/// Creates a new domain in the system.

/// Checks global limits and domain name validity
/// before database insertion. An audit log is created.

/// # Errors

/// Returns `DomainError::AlreadyExists` if the domain already exists.
/// Returns `DomainError::Validation` if the name is invalid.
pub async fn create_domain(&self, input: CreateDomain) -> Result<Domain, DomainError> {
```

### Rules

- Document all public functions with `///`
- Modules must have a `//!` header explaining their role
- `# Errors` and `# Panics` are mandatory when applicable
- Documentation examples (`# Examples`) are encouraged
- Do not document the obvious

---

---
---
## 12. Tooling and CI

### Formatting

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
```

- `cargo fmt` must pass without modifications
- Configured in the pre-commit hook

### Linting

```toml
# Clippy — in Cargo.toml or clippy.toml
[lints.clippy]
pedantic = "warn"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

- `cargo clippy -- -D warnings` must pass without error

### CI Pipeline

```
1. cargo fmt --check
2. cargo clippy -- -D warnings
3. cargo test
4. cargo audit
5. cargo build --release
6. Integration tests (testcontainers)
```

---

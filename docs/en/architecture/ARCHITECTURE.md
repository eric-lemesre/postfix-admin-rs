> **Language:** English | [Francais](../fr/architecture/ARCHITECTURE.md)

# Architecture — postfix-admin-rs

## Overview

```
                    ┌─────────────────────────────────────┐
                    │          Clients                     │
                    │  Browser  │  CLI  │  API clients     │
                    └─────┬─────┴───┬───┴──────┬──────────┘
                          │         │          │
                    ┌─────▼─────────▼──────────▼──────────┐
                    │         par-server                    │
                    │   (composition + startup)             │
                    └──┬──────────┬──────────┬─────────────┘
                       │          │          │
              ┌────────▼──┐  ┌───▼────┐  ┌──▼──────┐
              │  par-web   │  │par-api │  │ par-cli │
              │  (HTMX +   │  │(REST + │  │ (clap)  │
              │  Askama)   │  │ gRPC)  │  │         │
              └──────┬─────┘  └───┬────┘  └────┬────┘
                     │            │             │
                     └────────┬───┘             │
                              │                 │
                    ┌─────────▼─────────────────▼─────────┐
                    │            par-auth                   │
                    │  (sessions, JWT, TOTP, passwords)     │
                    └──────────────┬───────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │             par-db                    │
                    │   (repositories, pool, migrations)    │
                    └──────────────┬───────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │            par-core                   │
                    │  (models, traits, types, validation) │
                    └─────────────────────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │     PostgreSQL / MySQL / SQLite       │
                    └─────────────────────────────────────┘
```

## Crates in the workspace

### par-core (library)

Core of the business domain. No dependency on a web framework or database.

**Content:**
- Domain models (`Domain`, `Mailbox`, `Alias`, `Admin`, etc.)
- Valid newtypes (`DomainName`, `EmailAddress`)
- DTOs (`CreateDomain`, `UpdateMailbox`, etc.)
- Abstraction traits (`DomainRepository`, `MailboxRepository`, etc.)
- Domain error types (`DomainError`, `ValidationError`)
- Business validation logic

**Dependencies:** serde, thiserror, validator, chrono

### par-db (library)

Data access layer. Concrete implementations of the Repository traits.

**Content:**
- PostgreSQL implementations (`PgDomainRepository`, etc.)
- MySQL implementations (`MysqlDomainRepository`, etc.)
- SQLite implementations (`SqliteDomainRepository`, etc.)
- Connection pool management
- SQL row types (`DomainRow`, `MailboxRow`, etc.)
- Row to domain model conversions

**Dependencies:** sqlx, par-core

### par-auth (library)

Authentication, authorization and security.

**Content:**
- Multi-schema hashing (argon2, bcrypt, sha-crypt, md5-crypt)
- Transparent detection and rehashing
- TOTP 2FA (generation, verification, recovery codes)
- Session management (creation, validation, destruction)
- JWT (generation, verification, refresh)
- RBAC (axum extractors: `RequireSuperAdmin`, `RequireDomainAdmin`, `RequireUser`)
- CSRF tokens
- Rate limiting

**Dependencies:** argon2, bcrypt, sha-crypt, totp-rs, jsonwebtoken, par-core, par-db

### par-api (library)

REST and gRPC API.

**Content:**
- REST handlers (axum)
- Protobuf definitions and gRPC services (tonic)
- Request/response DTOs
- API authentication middleware
- OpenAPI documentation (utoipa)
- API rate limiting
- API error handling (RFC 7807)

**Dependencies:** axum, tonic, prost, utoipa, par-core, par-db, par-auth

### par-web (library)

Web interface.

**Content:**
- Web routes (axum)
- Askama templates (HTML)
- Static assets (CSS, JS, images)
- Web middleware (sessions, CSRF, flash messages)
- Internationalization (i18n)

**Dependencies:** axum, askama, par-core, par-db, par-auth

### par-cli (binary)

Command line interface.

**Content:**
- Clap commands (domain, mailbox, alias, admin, etc.)
- Output formatting (table, JSON, CSV)
- Utility commands (setup, migrate, hash-password)

**Dependencies:** clap, tabled, par-core, par-db, par-auth

### par-server (binary)

Main entry point. Composes all modules.

**Content:**
- `main()` function: config loading, pool initialization, server startup
- Axum router construction (web + API)
- Optional gRPC server startup
- Signal handling (graceful shutdown)
- Health check endpoint

**Dependencies:** tokio, axum, par-web, par-api, par-db, par-auth, par-core

## Data flow

### Typical web request

```
1. HTTP Request
2. axum Router → route matching
3. Middleware chain (session, auth, CSRF)
4. Handler (par-web or par-api)
5. Auth extractor (RequireSuperAdmin, etc.)
6. Input DTO validation
7. Repository call (trait)
8. SQL execution (concrete impl PG/MySQL/SQLite)
9. Row → Domain model conversion
10. Audit log
11. Model → Response DTO conversion
12. Template rendering (web) or JSON serialization (API)
13. HTTP Response
```

### Authentication

```
1. POST /login {username, password}
2. Repository.find_admin(username)
3. password::verify(input, stored_hash)
4. If old schema → transparent rehash
5. If TOTP enabled → partial session, redirect /login-mfa
6. POST /login-mfa {code}
7. totp::verify(secret, code)
8. Complete session created
9. Session cookie returned
```

## Architectural patterns

### Clean Architecture

Dependencies point inward (toward the domain):

```
Infrastructure (par-db, par-web, par-api)
    ↓
Application (par-auth, handlers)
    ↓
Domain (par-core)
```

The domain doesn't know about infrastructure. Traits defined in `par-core`
are implemented in `par-db`.

### Repository Pattern

Each entity has a Repository trait in `par-core` and concrete implementations
in `par-db` for each SQL backend.

### Dependency Injection

Injection happens via `Arc<dyn Repository>` in shared axum state:

```rust
let state = Arc::new(AppState {
    domain_repo: Arc::new(PgDomainRepository::new(pool.clone())),
    mailbox_repo: Arc::new(PgMailboxRepository::new(pool.clone())),
    // ...
});
```

### Cargo feature flags

```toml
[features]
default = ["postgres"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
sqlite = ["sqlx/sqlite"]
grpc = ["tonic", "prost"]
```

## Deployment

### Single binary

The `par-server` binary includes:
- HTTP server (web + REST API)
- Optional gRPC server
- Static assets (included at compile-time via `include_dir`)
- Templates (compiled by Askama)
- SQL migrations (included via `sqlx::migrate!`)

### Configuration

```
/etc/postfix-admin-rs/
├── config.toml          # Main configuration
└── config.local.toml    # Local overrides (gitignored)
```

### Ports

| Service | Default port |
|---------|--------------|
| HTTP (web + REST) | 8080 |
| gRPC | 50051 |

---

# TODO — Roadmap to v1.0

> Scope v1.0: all features implemented (P0 + P1 + Fetchmail),
> PostgreSQL + MySQL backends, SQLite deferred to v1.1.
> Migration tool from PHP PostfixAdmin deferred (schema compatibility maintained).

## Version plan

| Version    | Milestone | Phase       | Scope                                             | Effort | Status         |
|------------|-----------|-------------|---------------------------------------------------|--------|----------------|
| v0.1.0     | M0        | Foundation  | Workspace bootstrap, CI, tooling                  | S      | Done           |
| v0.2.0     | M1        | Foundation  | Core crate (models, traits, DTOs, validation)     | L      | Done           |
| v0.3.0     | M2        | Foundation  | DB crate (PG + MySQL repos, migrations)           | XL     | Done           |
| v0.4.0     | M3        | Foundation  | Configuration system (config-rs)                  | M      | Done           |
| v0.5.0     | M4        | Auth        | Auth crate (passwords, sessions, JWT, TOTP, RBAC) | XL     | Done           |
| v0.6.0     | M5        | Interfaces  | REST API (axum, OpenAPI, Newman tests)            | XL     | Done (30/30)   |
| v0.7.0     | M6        | Interfaces  | Web interface (Askama, HTMX, Tailwind, i18n)      | XL     | Partial (5/28) |
| v0.7.1     | M7        | Interfaces  | gRPC API (tonic, protobuf)                        | M      | Pending        |
| v0.7.2     | M8        | Interfaces  | CLI (clap, formatters)                            | L      | Pending        |
| v0.8.0     | M9        | Server      | Server composition (startup, shutdown, routing)   | M      | Partial (6/13) |
| v0.8.1     | M10       | Features    | Vacation auto-responder                           | M      | Pending        |
| v0.8.2     | M11       | Features    | DKIM management                                   | M      | Pending        |
| v0.8.3     | M12       | Features    | Fetchmail integration                             | M      | Pending        |
| v0.8.4     | M13       | Features    | Alias domains                                     | S      | Pending        |
| v0.9.0     | M14       | Quality     | Newman API test suite (100% coverage)             | L      | Partial (5/6)  |
| v0.9.1     | M15       | Quality     | Integration tests (testcontainers)                | L      | Pending        |
| v0.9.2     | M16       | Quality     | Security audit                                    | M      | Pending        |
| **v1.0.0** | **M17**   | **Release** | **Release, packaging, documentation**             | **M**  | **Pending**    |

> **Effort scale (solo developer):**
> S = small, M = medium, L = large, XL = very large.
> Critical path: M0 → M1 → M2 → M4 → M9 (each blocks the next).
> M5/M6/M7/M8 can be parallelized after M4 (M5 API before M6 Web recommended).
> M10-M13 can be done in any order after M9.
>
> **Status key:** Done = all items complete, Partial (x/y) = x items done out of y,
> Pending = not started. M5, M6, and M9 have partial implementations from
> groundwork done during earlier milestones (API routes, web scaffolding, server wiring).

---

## Phase 1 — Foundation

### M0: Project bootstrap `v0.1.0` [S] — Done (10/10)
- [x] Create workspace `Cargo.toml` with 7 crates
- [x] Create crate skeletons (`lib.rs` / `main.rs`) for all 7 crates
- [x] Configure `rustfmt.toml` (max_width = 100)
- [x] Configure `clippy.toml` (pedantic, deny unwrap/expect/panic)
- [x] Set up CI pipeline (GitHub Actions): fmt, clippy, test, audit, build
- [x] Add `.githooks/pre-commit` (fmt + clippy)
- [x] Create `config.example.toml` from SPEC-13.1
- [x] Add `deny.toml` (cargo-deny) for license and vulnerability checks
- [x] Set up test infrastructure (testcontainers, fixtures)
- [x] Create Newman test directory structure (`tests/newman/`)

### M1: postfix-admin-core `v0.2.0` [L] — Done (16/16)
> Specs: [SPEC-01.1](docs/en/features/01-domains/domain-management.md) · [SPEC-01.2](docs/en/features/01-domains/alias-domains.md) · [SPEC-02.1](docs/en/features/02-mailboxes/mailbox-management.md) · [SPEC-02.2](docs/en/features/02-mailboxes/quota-management.md) · [SPEC-03.1](docs/en/features/03-aliases/alias-management.md) · [SPEC-04.1](docs/en/features/04-authentication/admin-authentication.md) · [SPEC-04.3](docs/en/features/04-authentication/totp-2fa.md) · [SPEC-04.4](docs/en/features/04-authentication/app-passwords.md) · [SPEC-06.1](docs/en/features/06-vacation/vacation-autoresponder.md) · [SPEC-07.1](docs/en/features/07-fetchmail/fetchmail-integration.md) · [SPEC-08.1](docs/en/features/08-dkim/dkim-management.md) · [SPEC-09.1](docs/en/features/09-logging/audit-logging.md)

- [x] Domain models: `Domain`, `Mailbox`, `Alias`, `Admin`
- [x] Domain models: `Vacation`, `VacationNotification`
- [x] Domain models: `DkimKey`, `DkimSigning`
- [x] Domain models: `Fetchmail`, `Log`
- [x] Domain models: `MailboxAppPassword`, `TotpExceptionAddress`
- [x] Domain models: `AliasDomain`, `Quota`, `Quota2`
- [x] Validated newtypes: `DomainName`, `EmailAddress`, `Password` (zeroize)
- [x] DTOs: `Create*`, `Update*`, `*Response` for all entities
- [x] Repository traits: `DomainRepository`, `MailboxRepository`, `AliasRepository`
- [x] Repository traits: `AdminRepository`, `VacationRepository`
- [x] Repository traits: `DkimRepository`, `FetchmailRepository`, `LogRepository`
- [x] Repository traits: `AppPasswordRepository`, `AliasDomainRepository`
- [x] Error types: `CoreError`, `ValidationError`, `DomainError`
- [x] Business validation logic (validator + custom rules)
- [x] Pagination types: `PageRequest`, `PageResponse<T>`
- [x] Unit tests for all models, newtypes, and validation

### M2: postfix-admin-db `v0.3.0` [XL] — Done (24/24)
> Specs: same as M1

- [x] Connection pool management (sqlx, multi-backend)
- [x] SQL migrations: create all tables (PostgreSQL)
- [x] SQL migrations: create all tables (MySQL)
- [x] SQL migrations: create indexes per SCHEMA.md
- [x] `PgDomainRepository` implementation
- [x] `PgMailboxRepository` implementation
- [x] `PgAliasRepository` implementation
- [x] `PgAdminRepository` implementation
- [x] `PgVacationRepository` implementation
- [x] `PgDkimRepository` implementation
- [x] `PgFetchmailRepository` implementation
- [x] `PgLogRepository` implementation
- [x] `PgAppPasswordRepository` implementation
- [x] `PgAliasDomainRepository` implementation
- [x] `MysqlDomainRepository` implementation
- [x] `MysqlMailboxRepository` implementation
- [x] `MysqlAliasRepository` implementation
- [x] `MysqlAdminRepository` implementation
- [x] `MysqlVacationRepository` implementation
- [x] `MysqlDkimRepository` implementation
- [x] `MysqlFetchmailRepository` implementation
- [x] `MysqlLogRepository` implementation
- [x] `MysqlAppPasswordRepository` implementation
- [x] `MysqlAliasDomainRepository` implementation
- [x] Row types and `From<Row>` conversions
- [x] Transaction support helpers
- [x] Integration tests with testcontainers (PostgreSQL)
- [x] Integration tests with testcontainers (MySQL)

### M3: Configuration system `v0.4.0` [M] — Done (13/13)
> Specs: [SPEC-13.1](docs/en/features/13-configuration/configuration.md)

- [x] Config struct definitions matching `config.toml` structure
- [x] config-rs integration (TOML file + env vars + CLI overrides)
- [x] Resolution priority: CLI > env > secrets.toml > local.toml > {profile}.toml > default.toml > defaults
- [x] Profile system (dev, test, prep, prod) with contextual validation
- [x] Operating mode detection (development `./config/` vs deployed `/etc/postfix-admin-rs/`)
- [x] Startup validation (database URL, secret keys, password schemes, log levels)
- [x] Auto-generation of `secret_key` and `master_key` in dev/test mode
- [x] `SecretString` newtype (zeroize, masked debug, no serialize)
- [x] Secrets file separation (`secrets.toml`, `.gitignore`)
- [x] Unit tests for all config modules
- [x] Integration tests (`config_loading.rs`) with tempfile
- [x] Deployment templates (`dist/config.toml`, `dist/secrets.toml`, `dist/postfix-admin-rs.service`)
- [x] Wire `AppConfig::load()` into server `main.rs`

---

## Phase 2 — Authentication & Security

### M4: postfix-admin-auth `v0.5.0` [XL] — Done (30/30)
> Specs: [SPEC-04.1](docs/en/features/04-authentication/admin-authentication.md) · [SPEC-04.2](docs/en/features/04-authentication/user-authentication.md) · [SPEC-04.3](docs/en/features/04-authentication/totp-2fa.md) · [SPEC-04.4](docs/en/features/04-authentication/app-passwords.md) · [SPEC-04.5](docs/en/features/04-authentication/password-schemes.md) · [SPEC-05.1](docs/en/features/05-authorization/rbac.md)

- [x] Password scheme detection (prefix matching per SPEC-04.5)
- [x] Argon2id hashing and verification (OWASP 2024 parameters)
- [x] Bcrypt hashing and verification
- [x] SHA-512 crypt and SHA-256 crypt support
- [x] MD5 crypt and legacy DES crypt support (read-only, warn + reject)
- [x] Cleartext detection (dev mode only)
- [x] Transparent rehashing on successful auth
- [x] Constant-time password comparison (subtle)
- [x] Session management (server-side, configurable lifetime, MemoryStore)
- [x] Session cookie: HttpOnly, Secure, SameSite=Strict
- [x] Session regeneration after authentication (cycle_id)
- [x] JWT generation (access + refresh tokens, configurable lifetimes)
- [x] JWT verification and refresh flow
- [x] TOTP secret generation (160 bits, base32, totp-rs)
- [x] TOTP QR code generation (otpauth:// URI, PNG base64)
- [x] TOTP verification (SHA-1, 6 digits, 30s, tolerance +/-1)
- [x] TOTP replay protection (last timestamp tracking)
- [x] TOTP recovery codes (10, one-time use, XXXX-XXXX format)
- [x] TOTP IP exceptions (global + per-user)
- [x] App password generation (24 chars alphanumeric, no ambiguous)
- [x] App password hashing (argon2id) and verification
- [x] RBAC extractors: `RequireSuperAdmin`, `RequireDomainAdmin`
- [x] Scope verification: role + domain access + assurance levels
- [x] CSRF token generation and validation (base64url, constant-time)
- [x] Rate limiting (login attempts, configurable, DashMap)
- [x] Brute-force protection (progressive delays 2^n, IP lockout)
- [x] mTLS configuration (`MtlsConfig` struct, validation rules)
- [x] mTLS certificate info extraction (`CertificateInfo`, `MtlsVerifier`)
- [x] mTLS DN parsing (RFC 2253 and OpenSSL formats)
- [x] mTLS middleware integration (extract cert info from request headers)
- [x] mTLS enforcement per role (require cert for superadmin/domain admin)
- [x] Unit tests for all password schemes
- [x] Unit tests for JWT, TOTP, RBAC, rate limiting (75 tests)

---

## Phase 3 — Interfaces

### M5: REST API `v0.6.0` [XL] — Done (30/30)
> Specs: [SPEC-10.1](docs/en/features/10-api/rest-api.md)

- [x] API router setup (`/api/v1/`)
- [x] Authentication middleware (JWT Bearer + API Key)
- [x] Error handling: RFC 7807 (Problem Details for HTTP APIs)
- [x] Pagination: offset-based with meta headers
- [x] Response format: `{ "data": ..., "meta": ... }`
- [x] OpenAPI 3.1 generation (utoipa)
- [x] Swagger UI at `/api/docs`
- [x] Auth endpoints: login, refresh, logout, totp/verify
- [x] Domain endpoints: list, get, create, update, delete, toggle
- [x] Alias domain endpoints: list, create, delete
- [x] Mailbox endpoints: list, get, create, update, delete, change password
- [x] Alias endpoints: list, get, create, update, delete
- [x] Admin endpoints: list, get, create, update, delete
- [x] Vacation endpoints: get, update, delete (per mailbox)
- [x] Fetchmail endpoints: list, get, create, update, delete, test
- [x] DKIM endpoints: keys (list, create, delete), signing (list, create, delete), dns-check
- [x] Log endpoints: list (global + per domain)
- [x] Rate limiting middleware (100 req/min default, X-RateLimit-* headers)
- [x] CORS middleware (configurable origins per config.toml)
- [x] Newman test collection: auth endpoints
- [x] Newman test collection: domain endpoints
- [x] Newman test collection: mailbox endpoints
- [x] Newman test collection: alias endpoints
- [x] Newman test collection: admin endpoints
- [x] Newman test collection: vacation endpoints
- [x] Newman test collection: fetchmail endpoints
- [x] Newman test collection: DKIM endpoints
- [x] Newman test collection: log endpoints
- [x] Newman test collection: alias domain endpoints
- [x] Newman test collection: error cases (400, 401, 403, 404, 409, 422, 429)

### M6: postfix-admin-web `v0.7.0` [XL] — Partial (5/28)
> Specs: [SPEC-12.1](docs/en/features/12-web-ui/web-interface.md)

- [x] Askama template base layout (header, sidebar, main, footer)
- [ ] Tailwind CSS build pipeline (standalone CLI or npm)
- [ ] Dark mode support (class="dark", localStorage, prefers-color-scheme)
- [ ] Responsive design (mobile-first, collapsible sidebar)
- [x] HTMX integration (CDN or vendored)
- [ ] Alpine.js integration (dropdowns, modals, confirmations)
- [ ] i18n system (TOML language files, EN + FR)
- [ ] Language detection (Accept-Language, cookie, config)
- [x] Login page (admin + user)
- [ ] Dashboard (stats, quota charts, recent logs, alerts)
- [ ] Domain list (pagination, sorting, search, bulk actions, inline toggle)
- [x] Domain create/edit form (validation, error display)
- [ ] Mailbox list + create/edit forms
- [ ] Alias list + create/edit forms
- [ ] Admin list + create/edit forms
- [ ] Alias domain list + create/edit forms
- [ ] Vacation configuration page (user scope)
- [ ] DKIM key management pages (generate, list, signing, DNS check)
- [ ] Fetchmail configuration pages
- [ ] Audit log viewer (filterable, paginated, export CSV/JSON)
- [ ] User pages: password change, vacation, app passwords, TOTP setup
- [x] Flash messages (success, error, warning)
- [ ] Breadcrumb navigation
- [ ] CSRF token on all POST forms
- [ ] Security headers middleware (CSP, HSTS, X-Frame-Options, etc.)
- [ ] Static assets (Heroicons SVG, compiled CSS/JS)
- [ ] Accessibility: ARIA, keyboard nav, WCAG 2.1 AA contrast
- [ ] Template unit tests (render without errors)

### M7: gRPC API `v0.7.1` [M] — Pending (0/7)
> Specs: [SPEC-10.2](docs/en/features/10-api/grpc-api.md)

- [ ] Protobuf definitions (proto3): DomainService, MailboxService, AliasService, AdminService
- [ ] tonic service implementations
- [ ] Authentication interceptor (Bearer JWT/API Key in metadata)
- [ ] Optional TLS and mTLS support
- [ ] Reflection service (configurable, disabled in production)
- [ ] gRPC error mapping
- [ ] Integration tests for all gRPC services

### M8: CLI `v0.7.2` [L] — Pending (0/17)
> Specs: [SPEC-11.1](docs/en/features/11-cli/cli-administration.md)

- [ ] clap command structure: `postfix-admin-rs <subcommand> <action>`
- [ ] `domain` subcommand: list, show, add, edit, delete, toggle
- [ ] `mailbox` subcommand: list, show, add, edit, delete, toggle, password, quota
- [ ] `alias` subcommand: list, show, add, edit, delete
- [ ] `admin` subcommand: list, show, add, edit, delete, password, domains
- [ ] Utility: `setup` (first superadmin, interactive)
- [ ] Utility: `migrate`, `migrate --status`
- [ ] Utility: `config check`, `config show`
- [ ] Utility: `hash-password`, `verify-password`
- [ ] Utility: `export` (JSON), `import` (JSON/CSV)
- [ ] Utility: `log list` (filterable)
- [ ] Utility: `stats` (per domain)
- [ ] Utility: `completion` (bash, zsh, fish)
- [ ] Output formatters: table (tabled), JSON, CSV
- [ ] Global options: --config, --database-url, --format, --quiet, --verbose, --yes, --color
- [ ] Exit codes per SPEC-11.1 (0-5, 10)
- [ ] Unit tests for all commands (mock repositories)

---

## Phase 4 — Server & Feature modules

### M9: postfix-admin-server `v0.8.0` [M] — Partial (6/13)
- [x] Main entry point (`main.rs`)
- [x] Config loading and validation
- [x] Database pool initialization
- [x] Axum router construction (mount web + API routes)
- [ ] Optional gRPC server startup (separate port)
- [ ] Health check endpoint (`GET /health`)
- [ ] Prometheus metrics endpoint (`GET /metrics`, optional)
- [x] Graceful shutdown (SIGTERM, SIGINT)
- [ ] Signal handling (SIGHUP for config reload)
- [x] Structured logging setup (tracing, JSON/pretty)
- [ ] Syslog output (optional)
- [ ] Static assets serving (include_dir)
- [ ] Smoke tests (server starts and responds)

### M10: Vacation auto-responder `v0.8.1` [M] — Pending (0/7)
> Specs: [SPEC-06.1](docs/en/features/06-vacation/vacation-autoresponder.md)

- [ ] Vacation CRUD (web + API)
- [ ] Alias modification on activation/deactivation
- [ ] Scheduled activation (active_from / active_until)
- [ ] Deduplication table (VacationNotification)
- [ ] Exclusion rules (MAILER-DAEMON, noreply, Precedence, X-Loop)
- [ ] Configurable deduplication interval
- [ ] Tests

### M11: DKIM management `v0.8.2` [M] — Pending (0/9)
> Specs: [SPEC-08.1](docs/en/features/08-dkim/dkim-management.md)

- [ ] RSA key generation (2048 default, configurable 1024/2048/4096)
- [ ] Private key encryption at rest (AES-256-GCM)
- [ ] Selector management (unique per domain)
- [ ] Signing table management (author patterns with wildcards)
- [ ] DNS record display (TXT format)
- [ ] Optional DNS verification
- [ ] OpenDKIM integration (KeyTable/SigningTable format)
- [ ] Web UI pages + API endpoints
- [ ] Tests

### M12: Fetchmail integration `v0.8.3` [M] — Pending (0/8)
> Specs: [SPEC-07.1](docs/en/features/07-fetchmail/fetchmail-integration.md)

- [ ] Fetchmail config CRUD (web + API)
- [ ] Remote password encryption (AES-256-GCM)
- [ ] Connectivity test endpoint
- [ ] Polling daemon (reads active configs, runs fetchmail periodically)
- [ ] Minimum poll interval enforcement (5 min)
- [ ] Diagnostic storage (returned_text)
- [ ] Web UI pages + API endpoints
- [ ] Tests

### M13: Alias domains `v0.8.4` [S] — Pending (0/5)
> Specs: [SPEC-01.2](docs/en/features/01-domains/alias-domains.md)

- [ ] Alias domain CRUD (web + API + CLI)
- [ ] Transparent routing (mail to alias domain → target domain)
- [ ] Active/inactive toggle
- [ ] Validation: target domain must exist, no self-reference, no circular chains
- [ ] Tests

---

## Phase 5 — Quality & Release

### M14: Newman API test suite `v0.9.0` [L] — Partial (5/6)
- [x] Environment files (dev, ci) with base URL and credentials
- [x] Pre-request scripts for auth token management
- [x] Collection runner configuration for CI
- [x] Verify 100% route coverage (every REST endpoint tested)
- [x] Success cases + error cases for each endpoint
- [ ] CI integration: Newman runs after integration tests

### M15: Integration tests `v0.9.1` [L] — Pending (0/7)
- [ ] testcontainers setup for PostgreSQL
- [ ] testcontainers setup for MySQL
- [ ] End-to-end tests: domain lifecycle (create → mailbox → alias → delete)
- [ ] End-to-end tests: auth flow (login → session → TOTP → logout)
- [ ] End-to-end tests: API flow (JWT → CRUD → pagination → errors)
- [ ] Migration tests (both backends)
- [ ] Performance benchmarks (optional)

### M16: Security audit `v0.9.2` [M] — Pending (0/10)
- [ ] `cargo audit` — zero known vulnerabilities
- [ ] `cargo deny` — license compliance
- [ ] Review all SQL queries (parameterized, no injection)
- [ ] Review all password handling (never logged, never serialized)
- [ ] Review session security (cookie flags, regeneration)
- [ ] Review CSRF protection (all POST forms)
- [ ] Review HTTP security headers
- [ ] Review rate limiting and brute-force protection
- [ ] Review input validation and sanitization
- [ ] Verify TLS configuration guidance in docs

### M17: Release `v1.0.0` [M] — Pending (0/9)
- [ ] Version bump to 1.0.0 in all Cargo.toml
- [ ] CHANGELOG.md
- [ ] Release binaries (Linux amd64, arm64)
- [ ] Docker image (multi-stage build, distroless)
- [ ] Docker Compose example (app + PostgreSQL)
- [ ] Debian/Ubuntu package (.deb)
- [ ] GitHub Release with release notes
- [ ] Update README badges (CI, version, license)
- [ ] Final documentation review (EN/FR sync)

---

## Out of scope for v1.0 (deferred)

- [ ] Packaging (deb, rpm, Docker) — separate milestone
- [ ] SQLite backend support (v1.1)
- [ ] Automated migration tool from PostfixAdmin PHP (v1.1)
- [ ] LDAP authentication backend (v1.2)
- [ ] Prometheus/Grafana dashboard templates (v1.1)
- [ ] Helm chart for Kubernetes (v1.2)
- [ ] Email notifications (password expiry, quota warnings) (v1.1)

---

## Discussion points

> Items that may need further specification refinement.

### Web UI
- [ ] **DISCUSS:** Dashboard chart library — server-rendered SVG vs lightweight JS (Chart.js)?
- [ ] **DISCUSS:** Email template system for vacation — plain text only or HTML support?

### API
- [ ] **DISCUSS:** API key management — generated from web UI only, or also from CLI?
- [ ] **DISCUSS:** Webhook support — notify external systems on domain/mailbox changes?

### Deployment
- [ ] **DISCUSS:** Docker secrets integration — Docker Swarm secrets or just env vars?
- [ ] **DISCUSS:** Systemd socket activation — worth implementing for zero-downtime reload?

### Features
- [ ] **DISCUSS:** Broadcast/announcement system — send message to all users of a domain?
- [ ] **DISCUSS:** Password expiry enforcement — force password change after N days?
- [ ] **DISCUSS:** Quota warning emails — automated alerts when mailbox nears quota limit?

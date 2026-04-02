# TODO — Roadmap to v1.0

> Scope v1.0: all features implemented (P0 + P1 + Fetchmail),
> PostgreSQL + MySQL backends, SQLite deferred to v1.1.
> Migration tool from PHP PostfixAdmin deferred (schema compatibility maintained).

## Version plan

| Version    | Milestone | Phase       | Scope                                             | Effort |
|------------|-----------|-------------|---------------------------------------------------|--------|
| v0.1.0     | M0        | Foundation  | Workspace bootstrap, CI, tooling                  | S      |
| v0.2.0     | M1        | Foundation  | Core crate (models, traits, DTOs, validation)     | L      |
| v0.3.0     | M2        | Foundation  | DB crate (PG + MySQL repos, migrations)           | XL     |
| v0.4.0     | M3        | Foundation  | Configuration system (config-rs)                  | M      |
| v0.5.0     | M4        | Auth        | Auth crate (passwords, sessions, JWT, TOTP, RBAC) | XL     |
| v0.6.0     | M5        | Interfaces  | Web interface (Askama, HTMX, Tailwind, i18n)      | XL     |
| v0.7.0     | M6        | Interfaces  | REST API (axum, OpenAPI, Newman tests)            | XL     |
| v0.7.1     | M7        | Interfaces  | gRPC API (tonic, protobuf)                        | M      |
| v0.7.2     | M8        | Interfaces  | CLI (clap, formatters)                            | L      |
| v0.8.0     | M9        | Server      | Server composition (startup, shutdown, routing)   | M      |
| v0.8.1     | M10       | Features    | Vacation auto-responder                           | M      |
| v0.8.2     | M11       | Features    | DKIM management                                   | M      |
| v0.8.3     | M12       | Features    | Fetchmail integration                             | M      |
| v0.8.4     | M13       | Features    | Alias domains                                     | S      |
| v0.9.0     | M14       | Quality     | Newman API test suite (100% coverage)             | L      |
| v0.9.1     | M15       | Quality     | Integration tests (testcontainers)                | L      |
| v0.9.2     | M16       | Quality     | Security audit                                    | M      |
| **v1.0.0** | **M17**   | **Release** | **Release, packaging, documentation**             | **M**  |

> **Effort scale (solo developer):**
> S = small, M = medium, L = large, XL = very large.
> Critical path: M0 → M1 → M2 → M4 → M9 (each blocks the next).
> M5/M6/M7/M8 can be parallelized after M4.
> M10-M13 can be done in any order after M9.

---

## Phase 1 — Foundation

### M0: Project bootstrap `v0.1.0` [S]
- [ ] Create workspace `Cargo.toml` with 7 crates
- [ ] Create crate skeletons (`lib.rs` / `main.rs`) for all 7 crates
- [ ] Configure `rustfmt.toml` (max_width = 100)
- [ ] Configure `clippy.toml` (pedantic, deny unwrap/expect/panic)
- [ ] Set up CI pipeline (GitHub Actions): fmt, clippy, test, audit, build
- [ ] Add `.githooks/pre-commit` (fmt + clippy)
- [ ] Create `config.example.toml` from SPEC-13.1
- [ ] Add `deny.toml` (cargo-deny) for license and vulnerability checks
- [ ] Set up test infrastructure (testcontainers, fixtures)
- [ ] Create Newman test directory structure (`tests/newman/`)

### M1: postfix-admin-core `v0.2.0` [L]
- [ ] Domain models: `Domain`, `Mailbox`, `Alias`, `Admin`
- [ ] Domain models: `Vacation`, `VacationNotification`
- [ ] Domain models: `DkimKey`, `DkimSigning`
- [ ] Domain models: `Fetchmail`, `Log`
- [ ] Domain models: `MailboxAppPassword`, `TotpExceptionAddress`
- [ ] Domain models: `AliasDomain`, `Quota`, `Quota2`
- [ ] Validated newtypes: `DomainName`, `EmailAddress`, `Password` (zeroize)
- [ ] DTOs: `Create*`, `Update*`, `*Response` for all entities
- [ ] Repository traits: `DomainRepository`, `MailboxRepository`, `AliasRepository`
- [ ] Repository traits: `AdminRepository`, `VacationRepository`
- [ ] Repository traits: `DkimRepository`, `FetchmailRepository`, `LogRepository`
- [ ] Repository traits: `AppPasswordRepository`, `AliasDomainRepository`
- [ ] Error types: `CoreError`, `ValidationError`, `DomainError`
- [ ] Business validation logic (validator + custom rules)
- [ ] Pagination types: `PageRequest`, `PageResponse<T>`
- [ ] Unit tests for all models, newtypes, and validation

### M2: postfix-admin-db `v0.3.0` [XL]
- [ ] Connection pool management (sqlx, multi-backend)
- [ ] SQL migrations: create all tables (PostgreSQL)
- [ ] SQL migrations: create all tables (MySQL)
- [ ] SQL migrations: create indexes per SCHEMA.md
- [ ] `PgDomainRepository` implementation
- [ ] `PgMailboxRepository` implementation
- [ ] `PgAliasRepository` implementation
- [ ] `PgAdminRepository` implementation
- [ ] `PgVacationRepository` implementation
- [ ] `PgDkimRepository` implementation
- [ ] `PgFetchmailRepository` implementation
- [ ] `PgLogRepository` implementation
- [ ] `PgAppPasswordRepository` implementation
- [ ] `PgAliasDomainRepository` implementation
- [ ] `MysqlDomainRepository` implementation
- [ ] `MysqlMailboxRepository` implementation
- [ ] `MysqlAliasRepository` implementation
- [ ] `MysqlAdminRepository` implementation
- [ ] `MysqlVacationRepository` implementation
- [ ] `MysqlDkimRepository` implementation
- [ ] `MysqlFetchmailRepository` implementation
- [ ] `MysqlLogRepository` implementation
- [ ] `MysqlAppPasswordRepository` implementation
- [ ] `MysqlAliasDomainRepository` implementation
- [ ] Row types and `From<Row>` conversions
- [ ] Transaction support helpers
- [ ] Integration tests with testcontainers (PostgreSQL)
- [ ] Integration tests with testcontainers (MySQL)

### M3: Configuration system `v0.4.0` [M]
- [ ] Config struct definitions matching `config.toml` structure
- [ ] config-rs integration (TOML file + env vars + CLI overrides)
- [ ] Resolution priority: CLI > env > config.local.toml > config.toml > defaults
- [ ] Startup validation (database URL, secret keys, TLS paths, etc.)
- [ ] Auto-generation of `secret_key` and `master_key` on first launch
- [ ] Unit tests for config loading and validation

---

## Phase 2 — Authentication & Security

### M4: postfix-admin-auth `v0.5.0` [XL]
- [ ] Password scheme detection (prefix matching per SPEC-04.5)
- [ ] Argon2id hashing and verification (OWASP 2024 parameters)
- [ ] Bcrypt hashing and verification
- [ ] SHA-512 crypt and SHA-256 crypt support
- [ ] MD5 crypt and legacy DES crypt support (read-only)
- [ ] Cleartext detection (dev mode only)
- [ ] Transparent rehashing on successful auth
- [ ] Constant-time password comparison
- [ ] Session management (server-side, configurable lifetime)
- [ ] Session cookie: HttpOnly, Secure, SameSite=Strict
- [ ] Session regeneration after authentication
- [ ] JWT generation (access + refresh tokens, configurable lifetimes)
- [ ] JWT verification and refresh flow
- [ ] TOTP secret generation (160 bits, base32)
- [ ] TOTP QR code generation (otpauth:// URI)
- [ ] TOTP verification (SHA-1, 6 digits, 30s, tolerance +/-1)
- [ ] TOTP replay protection (last timestamp tracking)
- [ ] TOTP recovery codes (10, one-time use)
- [ ] TOTP IP exceptions (global + per-user)
- [ ] App password generation (24 chars alphanumeric)
- [ ] App password hashing (argon2id) and verification
- [ ] RBAC extractors: `RequireSuperAdmin`, `RequireDomainAdmin`, `RequireUser`
- [ ] Scope verification: role + domain access + resource ownership
- [ ] CSRF token generation and validation
- [ ] Rate limiting (login attempts, configurable per SPEC-04.1)
- [ ] Brute-force protection (progressive delays, IP lockout)
- [ ] Unit tests for all password schemes
- [ ] Unit tests for JWT, TOTP, RBAC, rate limiting

---

## Phase 3 — Interfaces

### M5: postfix-admin-web `v0.6.0` [XL]
- [ ] Askama template base layout (header, sidebar, main, footer)
- [ ] Tailwind CSS build pipeline (standalone CLI or npm)
- [ ] Dark mode support (class="dark", localStorage, prefers-color-scheme)
- [ ] Responsive design (mobile-first, collapsible sidebar)
- [ ] HTMX integration (CDN or vendored)
- [ ] Alpine.js integration (dropdowns, modals, confirmations)
- [ ] i18n system (TOML language files, EN + FR)
- [ ] Language detection (Accept-Language, cookie, config)
- [ ] Login page (admin + user)
- [ ] Dashboard (stats, quota charts, recent logs, alerts)
- [ ] Domain list (pagination, sorting, search, bulk actions, inline toggle)
- [ ] Domain create/edit form (validation, error display)
- [ ] Mailbox list + create/edit forms
- [ ] Alias list + create/edit forms
- [ ] Admin list + create/edit forms
- [ ] Alias domain list + create/edit forms
- [ ] Vacation configuration page (user scope)
- [ ] DKIM key management pages (generate, list, signing, DNS check)
- [ ] Fetchmail configuration pages
- [ ] Audit log viewer (filterable, paginated, export CSV/JSON)
- [ ] User pages: password change, vacation, app passwords, TOTP setup
- [ ] Flash messages (success, error, warning)
- [ ] Breadcrumb navigation
- [ ] CSRF token on all POST forms
- [ ] Security headers middleware (CSP, HSTS, X-Frame-Options, etc.)
- [ ] Static assets (Heroicons SVG, compiled CSS/JS)
- [ ] Accessibility: ARIA, keyboard nav, WCAG 2.1 AA contrast
- [ ] Template unit tests (render without errors)

### M6: REST API `v0.7.0` [XL]
- [ ] API router setup (`/api/v1/`)
- [ ] Authentication middleware (JWT Bearer + API Key)
- [ ] Error handling: RFC 7807 (Problem Details for HTTP APIs)
- [ ] Pagination: offset-based with meta headers
- [ ] Response format: `{ "data": ..., "meta": ... }`
- [ ] OpenAPI 3.1 generation (utoipa)
- [ ] Swagger UI at `/api/docs`
- [ ] Auth endpoints: login, refresh, logout, totp/verify
- [ ] Domain endpoints: list, get, create, update, delete, toggle
- [ ] Alias domain endpoints: list, create, delete
- [ ] Mailbox endpoints: list, get, create, update, delete, change password
- [ ] Alias endpoints: list, get, create, update, delete
- [ ] Admin endpoints: list, get, create, update, delete
- [ ] Vacation endpoints: get, update, delete (per mailbox)
- [ ] Fetchmail endpoints: list, get, create, update, delete, test
- [ ] DKIM endpoints: keys (list, create, delete), signing (list, create, delete), dns-check
- [ ] Log endpoints: list (global + per domain)
- [ ] Rate limiting middleware (100 req/min default, X-RateLimit-* headers)
- [ ] CORS middleware (configurable origins per config.toml)
- [ ] Newman test collection: auth endpoints
- [ ] Newman test collection: domain endpoints
- [ ] Newman test collection: mailbox endpoints
- [ ] Newman test collection: alias endpoints
- [ ] Newman test collection: admin endpoints
- [ ] Newman test collection: vacation endpoints
- [ ] Newman test collection: fetchmail endpoints
- [ ] Newman test collection: DKIM endpoints
- [ ] Newman test collection: log endpoints
- [ ] Newman test collection: alias domain endpoints
- [ ] Newman test collection: error cases (400, 401, 403, 404, 409, 422, 429)

### M7: gRPC API `v0.7.1` [M]
- [ ] Protobuf definitions (proto3): DomainService, MailboxService, AliasService, AdminService
- [ ] tonic service implementations
- [ ] Authentication interceptor (Bearer JWT/API Key in metadata)
- [ ] Optional TLS and mTLS support
- [ ] Reflection service (configurable, disabled in production)
- [ ] gRPC error mapping
- [ ] Integration tests for all gRPC services

### M8: CLI `v0.7.2` [L]
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

### M9: postfix-admin-server `v0.8.0` [M]
- [ ] Main entry point (`main.rs`)
- [ ] Config loading and validation
- [ ] Database pool initialization
- [ ] Axum router construction (mount web + API routes)
- [ ] Optional gRPC server startup (separate port)
- [ ] Health check endpoint (`GET /health`)
- [ ] Prometheus metrics endpoint (`GET /metrics`, optional)
- [ ] Graceful shutdown (SIGTERM, SIGINT)
- [ ] Signal handling (SIGHUP for config reload)
- [ ] Structured logging setup (tracing, JSON/pretty)
- [ ] Syslog output (optional)
- [ ] Static assets serving (include_dir)
- [ ] Smoke tests (server starts and responds)

### M10: Vacation auto-responder `v0.8.1` [M]
- [ ] Vacation CRUD (web + API)
- [ ] Alias modification on activation/deactivation
- [ ] Scheduled activation (active_from / active_until)
- [ ] Deduplication table (VacationNotification)
- [ ] Exclusion rules (MAILER-DAEMON, noreply, Precedence, X-Loop)
- [ ] Configurable deduplication interval
- [ ] Tests

### M11: DKIM management `v0.8.2` [M]
- [ ] RSA key generation (2048 default, configurable 1024/2048/4096)
- [ ] Private key encryption at rest (AES-256-GCM)
- [ ] Selector management (unique per domain)
- [ ] Signing table management (author patterns with wildcards)
- [ ] DNS record display (TXT format)
- [ ] Optional DNS verification
- [ ] OpenDKIM integration (KeyTable/SigningTable format)
- [ ] Web UI pages + API endpoints
- [ ] Tests

### M12: Fetchmail integration `v0.8.3` [M]
- [ ] Fetchmail config CRUD (web + API)
- [ ] Remote password encryption (AES-256-GCM)
- [ ] Connectivity test endpoint
- [ ] Polling daemon (reads active configs, runs fetchmail periodically)
- [ ] Minimum poll interval enforcement (5 min)
- [ ] Diagnostic storage (returned_text)
- [ ] Web UI pages + API endpoints
- [ ] Tests

### M13: Alias domains `v0.8.4` [S]
- [ ] Alias domain CRUD (web + API + CLI)
- [ ] Transparent routing (mail to alias domain → target domain)
- [ ] Active/inactive toggle
- [ ] Validation: target domain must exist, no self-reference, no circular chains
- [ ] Tests

---

## Phase 5 — Quality & Release

### M14: Newman API test suite `v0.9.0` [L]
- [ ] Environment files (dev, ci) with base URL and credentials
- [ ] Pre-request scripts for auth token management
- [ ] Collection runner configuration for CI
- [ ] Verify 100% route coverage (every REST endpoint tested)
- [ ] Success cases + error cases for each endpoint
- [ ] CI integration: Newman runs after integration tests

### M15: Integration tests `v0.9.1` [L]
- [ ] testcontainers setup for PostgreSQL
- [ ] testcontainers setup for MySQL
- [ ] End-to-end tests: domain lifecycle (create → mailbox → alias → delete)
- [ ] End-to-end tests: auth flow (login → session → TOTP → logout)
- [ ] End-to-end tests: API flow (JWT → CRUD → pagination → errors)
- [ ] Migration tests (both backends)
- [ ] Performance benchmarks (optional)

### M16: Security audit `v0.9.2` [M]
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

### M17: Release `v1.0.0` [M]
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

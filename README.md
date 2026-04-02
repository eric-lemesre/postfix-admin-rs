> **Language:** English | [Francais](README.fr.md)

# postfix-admin-rs

Web administration for Postfix/Dovecot mail servers, rewritten in Rust.

Functional clone of [PostfixAdmin](https://github.com/postfixadmin/postfixadmin) (PHP) with a modern architecture, REST + gRPC API, and HTMX/Tailwind interface.

## Features

- **Virtual domains** — Full CRUD, quotas, transport, backup MX, domain aliases
- **Mailboxes** — Creation, individual and per-domain quotas, auto-generated maildir
- **Aliases** — Standard aliases, catch-all, mailing lists
- **Vacation** — Auto-responder with deduplication and scheduling
- **DKIM** — Key generation, signature table, DNS verification
- **Fetchmail** — POP3/IMAP retrieval from remote servers
- **Authentication** — Multi-scheme (argon2id, bcrypt, sha512-crypt...), TOTP 2FA, application passwords
- **RBAC** — Superadmin, domain admin, user roles
- **Audit log** — Full traceability of administrative actions
- **Transparent migration** — Compatible with existing PostfixAdmin PHP databases

## Technical stack

| Layer     | Technology                                                                                                |
|-----------|-----------------------------------------------------------------------------------------------------------|
| Language  | Rust (edition 2021)                                                                                       |
| Web       | [axum](https://github.com/tokio-rs/axum)                                                                  |
| Database  | [sqlx](https://github.com/launchbadge/sqlx) — PostgreSQL, MySQL, SQLite                                   |
| Templates | [Askama](https://github.com/djc/askama)                                                                   |
| Frontend  | [HTMX](https://htmx.org/) + [Tailwind CSS](https://tailwindcss.com/) + [Alpine.js](https://alpinejs.dev/) |
| REST API  | axum + [utoipa](https://github.com/juhaku/utoipa) (OpenAPI)                                               |
| gRPC      | [tonic](https://github.com/hyperium/tonic) + [prost](https://github.com/tokio-rs/prost)                   |
| CLI       | [clap](https://github.com/clap-rs/clap)                                                                   |
| Auth      | argon2, bcrypt, sha-crypt, [totp-rs](https://github.com/constantoine/totp-rs)                             |
| Config    | [config-rs](https://github.com/mehcode/config-rs) (TOML)                                                  |
| Logging   | [tracing](https://github.com/tokio-rs/tracing)                                                            |
| Tests     | cargo test, [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)                      |

## Architecture

Multi-crate Cargo workspace following Clean Architecture principles:

```
crates/
├── postfix-admin-core/      Domain models, traits, validation
├── postfix-admin-db/        Repositories (PostgreSQL, MySQL, SQLite)
├── postfix-admin-auth/      Authentication, TOTP, sessions, RBAC
├── postfix-admin-api/       REST + gRPC API
├── postfix-admin-web/       Web interface (Askama + HTMX)
├── postfix-admin-cli/       Administration CLI
└── postfix-admin-server/    Main binary
```

Dependencies flow from the outside in: `postfix-admin-server` → `postfix-admin-web`/`postfix-admin-api` → `postfix-admin-auth` → `postfix-admin-db` → `postfix-admin-core`.

See [docs/architecture/ARCHITECTURE.md](docs/en/architecture/ARCHITECTURE.md) for details.

## Prerequisites

- Rust 1.75+ (edition 2021)
- PostgreSQL 14+, MySQL 8+, or SQLite 3.35+
- Node.js 18+ (for Tailwind CSS compilation only)
- Docker (optional, for testcontainers)

## Installation

### From source

```bash
git clone https://github.com/eric-lemesre/postfix-admin-rs.git
cd postfix-admin-rs
cargo build --release
```

The binary is generated in `target/release/postfix-admin-rs`.

### Debian package

```bash
sudo dpkg -i postfix-admin-rs_1.0.0_amd64.deb
```

### Docker

```bash
docker run -d \
    -p 8080:8080 \
    -e PAR_DATABASE__URL="postgresql://postfix:pass@host:5432/postfix" \
    ghcr.io/eric-lemesre/postfix-admin-rs:latest
```

## Quick start

### 1. Create the database

```sql
-- PostgreSQL
CREATE USER postfix WITH PASSWORD 'choose_a_password';
CREATE DATABASE postfix OWNER postfix ENCODING 'UTF8';
```

### 2. Configure

```bash
sudo mkdir -p /etc/postfix-admin-rs
sudo cp config/default.toml /etc/postfix-admin-rs/config.toml
```

Edit `/etc/postfix-admin-rs/config.toml`:

```toml
[database]
url = "postgresql://postfix:choose_a_password@localhost:5432/postfix"

[server]
bind_address = "0.0.0.0"
port = 8080
```

### 3. Initialize

```bash
# Apply migrations
postfix-admin-rs migrate

# Create first admin user
postfix-admin-rs setup
```

### 4. Start

```bash
postfix-admin-rs serve
```

The interface is available at `http://localhost:8080`.

## Migration from PostfixAdmin PHP

postfix-admin-rs can connect directly to an existing PostfixAdmin PHP database.
Migrations add necessary columns without breaking compatibility.
Passwords are automatically rehashed on login.

```bash
# Point to the existing database
postfix-admin-rs --database-url "postgresql://postfix:pass@localhost/postfix" migrate
postfix-admin-rs serve
```

See [docs/migration/MIGRATION-FROM-PHP.md](docs/en/migration/MIGRATION-FROM-PHP.md) for complete guide.

## CLI

```bash
postfix-admin-rs domain list
postfix-admin-rs domain add example.com --description "My domain"
postfix-admin-rs mailbox add user@example.com --password "secret" --name "User"
postfix-admin-rs alias add info@example.com --goto "user@example.com,other@example.com"
postfix-admin-rs log list --last 20
```

See [docs/features/11-cli/cli-administration.md](docs/en/features/11-cli/cli-administration.md) for all commands.

## API

### REST

All resources are available via the `/api/v1/` prefixed REST API.

```bash
# Authentication
curl -X POST http://localhost:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username": "admin@example.com", "password": "..."}'

# List domains
curl http://localhost:8080/api/v1/domains \
    -H "Authorization: Bearer <token>"
```

Interactive OpenAPI documentation: `http://localhost:8080/api/docs`

### gRPC

Default port: `50051`. Enable in configuration:

```toml
[grpc]
enabled = true
port = 50051
```

## Configuration

Main file: `/etc/postfix-admin-rs/config.toml`

Values can be overridden by environment variables prefixed with `PAR_`:

| Variable                    | Description                                 |
|-----------------------------|---------------------------------------------|
| `PAR_DATABASE__URL`         | Database connection URL                     |
| `PAR_SERVER__PORT`          | HTTP listening port                         |
| `PAR_LOGGING__LEVEL`        | Log level (trace, debug, info, warn, error) |
| `PAR_AUTH__PASSWORD_SCHEME` | Hashing scheme (argon2id, bcrypt)           |

See [docs/features/13-configuration/configuration.md](docs/en/features/13-configuration/configuration.md) for complete reference.

## Development

```bash
# Clone
git clone https://github.com/eric-lemesre/postfix-admin-rs.git
cd postfix-admin-rs

# Configure Git hooks
git config core.hooksPath .githooks

# Dev database via Docker
docker run -d --name postfix-admin-dev-pg \
    -e POSTGRES_DB=postfix -e POSTGRES_USER=postfix -e POSTGRES_PASSWORD=postfix \
    -p 5432:5432 postgres:16-alpine

# Build and test
cargo build
cargo test
cargo clippy -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for complete contribution guide.

## Documentation

| Document                                                   | Description                                     |
|------------------------------------------------------------|-------------------------------------------------|
| [docs/features/](docs/en/features/00-overview.md)          | Functional specifications by module             |
| [docs/architecture/](docs/en/architecture/ARCHITECTURE.md) | Technical architecture and diagrams             |
| [docs/database/](docs/en/database/SCHEMA.md)               | Database schema                                 |
| [docs/guidelines/](docs/en/guidelines/)                    | Rust, JS, CSS, SQL, Git, Code Review guidelines |
| [docs/migration/](docs/en/migration/MIGRATION-FROM-PHP.md) | Migration guide from PHP                        |
| [docs/deployment/](docs/en/deployment/DEPLOYMENT.md)       | Deployment guide                                |

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

Based on work by [PostfixAdmin](https://github.com/postfixadmin/postfixadmin) (GPL v2+).

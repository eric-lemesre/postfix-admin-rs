> **Language:** [English](README.md) | Francais

# postfix-admin-rs

Administration web des serveurs de messagerie Postfix/Dovecot, reecrite en Rust.

Clone fonctionnel de [PostfixAdmin](https://github.com/postfixadmin/postfixadmin) (PHP) avec une architecture moderne, une API REST + gRPC, et une interface HTMX/Tailwind.

## Fonctionnalites

- **Domaines virtuels** — CRUD complet, quotas, transport, backup MX, alias de domaines
- **Boites mail** — Creation, quotas individuels et par domaine, maildir auto-genere
- **Alias** — Alias standard, catch-all, listes de distribution
- **Vacation** — Repondeur automatique avec deduplication et programmation
- **DKIM** — Generation de cles, table de signature, verification DNS
- **Fetchmail** — Recuperation POP3/IMAP depuis des serveurs distants
- **Authentification** — Multi-schema (argon2id, bcrypt, sha512-crypt...), TOTP 2FA, mots de passe applicatifs
- **RBAC** — Superadmin, admin de domaine, utilisateur
- **Journal d'audit** — Tracabilite complete des actions d'administration
- **Migration transparente** — Compatible avec les bases PostfixAdmin PHP existantes

## Stack technique

| Couche          | Technologie                                                                                               |
|-----------------|-----------------------------------------------------------------------------------------------------------|
| Langage         | Rust (edition 2021)                                                                                       |
| Web             | [axum](https://github.com/tokio-rs/axum)                                                                  |
| Base de donnees | [sqlx](https://github.com/launchbadge/sqlx) — PostgreSQL, MySQL, SQLite                                   |
| Templates       | [Askama](https://github.com/djc/askama)                                                                   |
| Frontend        | [HTMX](https://htmx.org/) + [Tailwind CSS](https://tailwindcss.com/) + [Alpine.js](https://alpinejs.dev/) |
| API REST        | axum + [utoipa](https://github.com/juhaku/utoipa) (OpenAPI)                                               |
| gRPC            | [tonic](https://github.com/hyperium/tonic) + [prost](https://github.com/tokio-rs/prost)                   |
| CLI             | [clap](https://github.com/clap-rs/clap)                                                                   |
| Auth            | argon2, bcrypt, sha-crypt, [totp-rs](https://github.com/constantoine/totp-rs)                             |
| Config          | [config-rs](https://github.com/mehcode/config-rs) (TOML)                                                  |
| Logging         | [tracing](https://github.com/tokio-rs/tracing)                                                            |
| Tests           | cargo test, [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)                      |

## Architecture

Workspace Cargo multi-crates suivant les principes de la Clean Architecture :

```
crates/
├── postfix-admin-core/      Modeles de domaine, traits, validation
├── postfix-admin-db/        Repositories (PostgreSQL, MySQL, SQLite)
├── postfix-admin-auth/      Authentification, TOTP, sessions, RBAC
├── postfix-admin-api/       API REST + gRPC
├── postfix-admin-web/       Interface web (Askama + HTMX)
├── postfix-admin-cli/       CLI d'administration
└── postfix-admin-server/    Binaire principal
```

Les dependances vont de l'exterieur vers l'interieur : `postfix-admin-server` → `postfix-admin-web`/`postfix-admin-api` → `postfix-admin-auth` → `postfix-admin-db` → `postfix-admin-core`.

Voir [docs/architecture/ARCHITECTURE.md](docs/fr/architecture/ARCHITECTURE.md) pour le detail.

## Prerequis

- Rust 1.75+ (edition 2021)
- PostgreSQL 14+, MySQL 8+ ou SQLite 3.35+
- Node.js 18+ (compilation Tailwind CSS uniquement)
- Docker (optionnel, pour testcontainers)

## Installation

### Depuis les sources

```bash
git clone https://github.com/eric-lemesre/postfix-admin-rs.git
cd postfix-admin-rs
cargo build --release
```

Le binaire est genere dans `target/release/postfix-admin-rs`.

### Package Debian

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

## Demarrage rapide

### 1. Creer la base de donnees

```sql
-- PostgreSQL
CREATE USER postfix WITH PASSWORD 'choose_a_password';
CREATE DATABASE postfix OWNER postfix ENCODING 'UTF8';
```

### 2. Configurer

```bash
sudo mkdir -p /etc/postfix-admin-rs
sudo cp config/default.toml /etc/postfix-admin-rs/config.toml
```

Editer `/etc/postfix-admin-rs/config.toml` :

```toml
[database]
url = "postgresql://postfix:choose_a_password@localhost:5432/postfix"

[server]
bind_address = "0.0.0.0"
port = 8080
```

### 3. Initialiser

```bash
# Appliquer les migrations
postfix-admin-rs migrate

# Creer le premier administrateur
postfix-admin-rs setup
```

### 4. Demarrer

```bash
postfix-admin-rs serve
```

L'interface est accessible sur `http://localhost:8080`.

## Migration depuis PostfixAdmin PHP

postfix-admin-rs peut se connecter directement a une base PostfixAdmin PHP existante.
Les migrations ajoutent les colonnes necessaires sans casser la compatibilite.
Les mots de passe sont rehashes automatiquement lors des connexions.

```bash
# Pointer vers la base existante
postfix-admin-rs --database-url "postgresql://postfix:pass@localhost/postfix" migrate
postfix-admin-rs serve
```

Voir [docs/migration/MIGRATION-FROM-PHP.md](docs/fr/migration/MIGRATION-FROM-PHP.md) pour le guide complet.

## CLI

```bash
postfix-admin-rs domain list
postfix-admin-rs domain add example.com --description "Mon domaine"
postfix-admin-rs mailbox add user@example.com --password "secret" --name "Utilisateur"
postfix-admin-rs alias add info@example.com --goto "user@example.com,other@example.com"
postfix-admin-rs log list --last 20
```

Voir [docs/features/11-cli/cli-administration.md](docs/fr/features/11-cli/cli-administration.md) pour toutes les commandes.

## API

### REST

Toutes les ressources sont accessibles via l'API REST prefixee `/api/v1/`.

```bash
# Authentification
curl -X POST http://localhost:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username": "admin@example.com", "password": "..."}'

# Lister les domaines
curl http://localhost:8080/api/v1/domains \
    -H "Authorization: Bearer <token>"
```

Documentation OpenAPI interactive : `http://localhost:8080/api/docs`

### gRPC

Port par defaut : `50051`. Activer dans la configuration :

```toml
[grpc]
enabled = true
port = 50051
```

## Configuration

Fichier principal : `/etc/postfix-admin-rs/config.toml`

Les valeurs peuvent etre surchargees par des variables d'environnement prefixees `PAR_` :

| Variable                    | Description                                     |
|-----------------------------|-------------------------------------------------|
| `PAR_DATABASE__URL`         | URL de connexion a la base                      |
| `PAR_SERVER__PORT`          | Port d'ecoute HTTP                              |
| `PAR_LOGGING__LEVEL`        | Niveau de log (trace, debug, info, warn, error) |
| `PAR_AUTH__PASSWORD_SCHEME` | Schema de hashing (argon2id, bcrypt)            |

Voir [docs/features/13-configuration/configuration.md](docs/fr/features/13-configuration/configuration.md) pour la reference complete.

## Developpement

```bash
# Cloner
git clone https://github.com/eric-lemesre/postfix-admin-rs.git
cd postfix-admin-rs

# Configurer les hooks Git
git config core.hooksPath .githooks

# Base de dev via Docker
docker run -d --name postfix-admin-dev-pg \
    -e POSTGRES_DB=postfix -e POSTGRES_USER=postfix -e POSTGRES_PASSWORD=postfix \
    -p 5432:5432 postgres:16-alpine

# Compiler et tester
cargo build
cargo test
cargo clippy -- -D warnings
```

Voir [CONTRIBUTING.md](CONTRIBUTING.fr.md) pour le guide de contribution complet.

## Documentation

| Document                                                   | Description                                     |
|------------------------------------------------------------|-------------------------------------------------|
| [docs/features/](docs/fr/features/00-overview.md)          | Specifications fonctionnelles par module        |
| [docs/architecture/](docs/fr/architecture/ARCHITECTURE.md) | Architecture technique et diagrammes            |
| [docs/database/](docs/fr/database/SCHEMA.md)               | Schema de base de donnees                       |
| [docs/guidelines/](docs/fr/guidelines/)                    | Guidelines Rust, JS, CSS, SQL, Git, Code Review |
| [docs/migration/](docs/fr/migration/MIGRATION-FROM-PHP.md) | Guide de migration depuis PHP                   |
| [docs/deployment/](docs/fr/deployment/DEPLOYMENT.md)       | Guide de deploiement                            |

## Licence

Ce projet est distribue sous la licence [GNU General Public License v3.0](LICENSE).

Basé sur le travail de [PostfixAdmin](https://github.com/postfixadmin/postfixadmin) (GPL v2+).

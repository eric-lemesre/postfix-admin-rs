> **Language:** [English](../en/architecture/ARCHITECTURE.md) | Francais

# Architecture — postfix-admin-rs

## Vue d'ensemble

```
                    ┌─────────────────────────────────────┐
                    │          Clients                     │
                    │  Browser  │  CLI  │  API clients     │
                    └─────┬─────┴───┬───┴──────┬──────────┘
                          │         │          │
                    ┌─────▼─────────▼──────────▼──────────┐
                    │         par-server                    │
                    │   (composition + demarrage)           │
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
                    │  (modeles, traits, types, validation) │
                    └─────────────────────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │     PostgreSQL / MySQL / SQLite       │
                    └─────────────────────────────────────┘
```

## Crates du workspace

### par-core (bibliotheque)

Coeur du domaine metier. Aucune dependance sur un framework web ou une base de donnees.

**Contenu :**
- Modeles de domaine (`Domain`, `Mailbox`, `Alias`, `Admin`, etc.)
- Newtypes valides (`DomainName`, `EmailAddress`)
- DTOs (`CreateDomain`, `UpdateMailbox`, etc.)
- Traits d'abstraction (`DomainRepository`, `MailboxRepository`, etc.)
- Types d'erreurs de domaine (`DomainError`, `ValidationError`)
- Logique de validation metier

**Dependances :** serde, thiserror, validator, chrono

### par-db (bibliotheque)

Couche d'acces aux donnees. Implementations concretes des traits Repository.

**Contenu :**
- Implementations PostgreSQL (`PgDomainRepository`, etc.)
- Implementations MySQL (`MysqlDomainRepository`, etc.)
- Implementations SQLite (`SqliteDomainRepository`, etc.)
- Gestion du pool de connexions
- Types de lignes SQL (`DomainRow`, `MailboxRow`, etc.)
- Conversions Row -> Modele de domaine

**Dependances :** sqlx, par-core

### par-auth (bibliotheque)

Authentification, autorisation et securite.

**Contenu :**
- Hashing multi-schema (argon2, bcrypt, sha-crypt, md5-crypt)
- Detection et rehash transparent
- TOTP 2FA (generation, verification, codes de recuperation)
- Gestion des sessions (creation, validation, destruction)
- JWT (generation, verification, refresh)
- RBAC (extracteurs axum : `RequireSuperAdmin`, `RequireDomainAdmin`, `RequireUser`)
- CSRF tokens
- Rate limiting

**Dependances :** argon2, bcrypt, sha-crypt, totp-rs, jsonwebtoken, par-core, par-db

### par-api (bibliotheque)

API REST et gRPC.

**Contenu :**
- Handlers REST (axum)
- Definitions Protobuf et services gRPC (tonic)
- DTOs de requete/reponse
- Middleware d'authentification API
- Documentation OpenAPI (utoipa)
- Rate limiting API
- Gestion d'erreurs API (RFC 7807)

**Dependances :** axum, tonic, prost, utoipa, par-core, par-db, par-auth

### par-web (bibliotheque)

Interface web.

**Contenu :**
- Routes web (axum)
- Templates Askama (HTML)
- Assets statiques (CSS, JS, images)
- Middleware web (sessions, CSRF, flash messages)
- Internationalisation (i18n)

**Dependances :** axum, askama, par-core, par-db, par-auth

### par-cli (binaire)

Interface en ligne de commande.

**Contenu :**
- Commandes clap (domain, mailbox, alias, admin, etc.)
- Formatage de sortie (table, JSON, CSV)
- Commandes utilitaires (setup, migrate, hash-password)

**Dependances :** clap, tabled, par-core, par-db, par-auth

### par-server (binaire)

Point d'entree principal. Compose tous les modules.

**Contenu :**
- Fonction `main()` : chargement config, initialisation pool, demarrage serveur
- Construction du routeur axum (web + API)
- Demarrage optionnel du serveur gRPC
- Signal handling (graceful shutdown)
- Health check endpoint

**Dependances :** tokio, axum, par-web, par-api, par-db, par-auth, par-core

## Flux de donnees

### Requete web typique

```
1. HTTP Request
2. axum Router → route matching
3. Middleware chain (session, auth, CSRF)
4. Handler (par-web ou par-api)
5. Extracteur d'auth (RequireSuperAdmin, etc.)
6. Validation du DTO d'entree
7. Appel au Repository (trait)
8. Execution SQL (impl concrete PG/MySQL/SQLite)
9. Conversion Row → Modele de domaine
10. Log d'audit
11. Conversion Modele → DTO de reponse
12. Rendu template (web) ou serialisation JSON (API)
13. HTTP Response
```

### Authentification

```
1. POST /login {username, password}
2. Repository.find_admin(username)
3. password::verify(input, stored_hash)
4. Si ancien schema → rehash transparent
5. Si TOTP active → session partielle, redirect /login-mfa
6. POST /login-mfa {code}
7. totp::verify(secret, code)
8. Session complete creee
9. Cookie de session retourne
```

## Patterns architecturaux

### Clean Architecture

Les dependances pointent vers l'interieur (vers le domaine) :

```
Infrastructure (par-db, par-web, par-api)
    ↓
Application (par-auth, handlers)
    ↓
Domaine (par-core)
```

Le domaine ne connait pas l'infrastructure. Les traits definis dans `par-core`
sont implementes dans `par-db`.

### Repository Pattern

Chaque entite a un trait Repository dans `par-core` et des implementations
concretes dans `par-db` pour chaque backend SQL.

### Dependency Injection

L'injection se fait via `Arc<dyn Repository>` dans l'etat partage axum :

```rust
let state = Arc::new(AppState {
    domain_repo: Arc::new(PgDomainRepository::new(pool.clone())),
    mailbox_repo: Arc::new(PgMailboxRepository::new(pool.clone())),
    // ...
});
```

### Feature flags Cargo

```toml
[features]
default = ["postgres"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
sqlite = ["sqlx/sqlite"]
grpc = ["tonic", "prost"]
```

## Deploiement

### Binaire unique

Le binaire `par-server` embarque :
- Le serveur HTTP (web + API REST)
- Le serveur gRPC (optionnel)
- Les assets statiques (inclus au compile-time via `include_dir`)
- Les templates (compiles par Askama)
- Les migrations SQL (incluses via `sqlx::migrate!`)

### Configuration

```
/etc/postfix-admin-rs/
├── config.toml          # Configuration principale
└── config.local.toml    # Overrides locaux (gitignored)
```

### Ports

| Service | Port par defaut |
|---------|----------------|
| HTTP (web + REST) | 8080 |
| gRPC | 50051 |

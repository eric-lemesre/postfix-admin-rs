# Guidelines Rust — postfix-admin-rs

## Table des matières

1. [Organisation du code](#1-organisation-du-code)
2. [Conventions de nommage](#2-conventions-de-nommage)
3. [Gestion des erreurs](#3-gestion-des-erreurs)
4. [Types et modèles](#4-types-et-modèles)
5. [Async et concurrence](#5-async-et-concurrence)
6. [Base de données et sqlx](#6-base-de-données-et-sqlx)
7. [API et handlers](#7-api-et-handlers)
8. [Tests](#8-tests)
9. [Performance](#9-performance)
10. [Sécurité](#10-sécurité)
11. [Documentation](#11-documentation)
12. [Outillage et CI](#12-outillage-et-ci)

---

## 1. Organisation du code

### Structure des crates

Chaque crate du workspace a une responsabilité unique :

```
crates/
├── par-core/      # Modèles de domaine, traits, types partagés
├── par-db/        # Couche d'accès aux données
├── par-auth/      # Authentification et autorisation
├── par-api/       # Handlers REST et gRPC
├── par-web/       # Interface web (templates, routes)
├── par-cli/       # CLI (binaire)
└── par-server/    # Serveur principal (binaire)
```

### Règles de dépendance entre crates

```
par-server → par-web, par-api, par-cli, par-db, par-auth, par-core
par-web    → par-core, par-db, par-auth
par-api    → par-core, par-db, par-auth
par-cli    → par-core, par-db, par-auth
par-auth   → par-core, par-db
par-db     → par-core
par-core   → (aucune dépendance interne)
```

Les dépendances circulaires sont interdites. Si deux crates ont besoin de
fonctionnalités communes, elles doivent être extraites dans `par-core`.

### Organisation d'un module

```rust
// Ordre dans un fichier :
// 1. Imports (regroupés et ordonnés)
// 2. Constantes
// 3. Types (structs, enums)
// 4. Implémentations de traits
// 5. Méthodes impl
// 6. Fonctions libres
// 7. Tests (module #[cfg(test)])
```

### Imports

```rust
// Regroupement par blocs séparés par une ligne vide :
// 1. std
use std::collections::HashMap;
use std::sync::Arc;

// 2. Crates externes
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// 3. Crates du workspace
use par_core::models::Domain;
use par_db::repositories::DomainRepository;

// 4. Modules locaux
use crate::error::AppError;
use super::middleware::RequireAuth;
```

Préférer les imports spécifiques aux imports globaux (`use module::*`).
Exception : les preludes des crates bien connues (`use sqlx::prelude::*`).

---

## 2. Conventions de nommage

### Général

| Élément | Convention | Exemple |
|---------|-----------|---------|
| Crate | kebab-case | `par-core` |
| Module | snake_case | `domain_repository` |
| Type (struct, enum) | PascalCase | `DomainHandler` |
| Trait | PascalCase (adjectif/verbe) | `Validatable`, `Repository` |
| Fonction / méthode | snake_case | `find_by_domain` |
| Constante | SCREAMING_SNAKE_CASE | `MAX_LOGIN_ATTEMPTS` |
| Variable | snake_case | `domain_count` |
| Lifetime | `'a`, `'de`, `'ctx` (court et descriptif) | `'a` |
| Type parameter | `T`, `E`, ou nom descriptif | `T: Repository` |
| Feature flag | kebab-case | `grpc-support` |

### Nommage spécifique au projet

| Concept | Pattern | Exemple |
|---------|---------|---------|
| Modèle de domaine | Nom simple | `Domain`, `Mailbox`, `Alias` |
| DTO de création | `Create{Entity}` | `CreateDomain`, `CreateMailbox` |
| DTO de mise à jour | `Update{Entity}` | `UpdateDomain`, `UpdateMailbox` |
| DTO de réponse | `{Entity}Response` | `DomainResponse`, `MailboxResponse` |
| Repository trait | `{Entity}Repository` | `DomainRepository` |
| Repository impl | `Pg{Entity}Repository` | `PgDomainRepository` |
| Handler / Controller | `{entity}_{action}` | `domain_create`, `mailbox_list` |
| Erreur | `{Module}Error` | `AuthError`, `DbError` |
| Middleware | `Require{Role}` | `RequireSuperAdmin` |

### Constructeurs

```rust
// Préférer new() pour les cas simples
impl Domain {
    pub fn new(name: String, description: String) -> Self { ... }
}

// Utiliser le pattern builder pour les types complexes
impl DomainBuilder {
    pub fn new(name: String) -> Self { ... }
    pub fn description(mut self, desc: String) -> Self { ... }
    pub fn build(self) -> Result<Domain, ValidationError> { ... }
}

// Utiliser from/into pour les conversions
impl From<DomainRow> for Domain { ... }
impl From<CreateDomainRequest> for CreateDomain { ... }
```

---

## 3. Gestion des erreurs

### Architecture des erreurs

```rust
// par-core/src/error.rs — Erreur de base du domaine

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("domaine non trouvé : {0}")]
    NotFound(String),

    #[error("domaine déjà existant : {0}")]
    AlreadyExists(String),

    #[error("validation échouée : {0}")]
    Validation(#[from] ValidationError),

    #[error("opération non autorisée")]
    Unauthorized,

    #[error("limite atteinte : {message}")]
    LimitReached { message: String },
}
```

```rust
// par-db/src/error.rs — Erreur de la couche DB

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("erreur de base de données : {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("enregistrement non trouvé")]
    NotFound,

    #[error("violation de contrainte unique : {0}")]
    UniqueViolation(String),
}
```

```rust
// par-api/src/error.rs — Erreur API convertie en réponse HTTP

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
        // Conversion en RFC 7807 Problem Details
        match self {
            ApiError::Domain(DomainError::NotFound(_)) => StatusCode::NOT_FOUND,
            ApiError::Domain(DomainError::Validation(_)) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Auth(AuthError::Unauthorized) => StatusCode::UNAUTHORIZED,
            // ...
        }
    }
}
```

### Règles

- Utiliser `thiserror` pour les types d'erreur de bibliothèque
- Utiliser `anyhow` uniquement dans les binaires (main, CLI) et les tests
- Ne jamais utiliser `.unwrap()` ou `.expect()` en code de production
  - Exceptions : après une vérification qui garantit le succès, avec un commentaire
- Propager les erreurs avec `?` plutôt que `match` quand c'est possible
- Les erreurs doivent contenir assez de contexte pour le diagnostic

```rust
// Bon : erreur avec contexte
Err(DomainError::NotFound(domain_name.to_string()))

// Mauvais : erreur sans contexte
Err(DomainError::NotFound("".to_string()))
```

- Ne pas logger ET propager la même erreur (choisir l'un ou l'autre)
- Logger au niveau le plus haut (handler), pas dans les couches basses

---

## 4. Types et modèles

### Modèles de domaine vs DTO

```rust
// Modèle de domaine (par-core) — représente la logique métier
pub struct Domain {
    pub name: DomainName,       // Type newtype validé
    pub description: String,
    pub limits: DomainLimits,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTO de création (par-core) — données entrantes
#[derive(Deserialize, Validate)]
pub struct CreateDomain {
    pub domain: String,
    pub description: Option<String>,
    pub aliases: Option<i32>,
    pub mailboxes: Option<i32>,
    // ...
}

// DTO de réponse (par-api) — données sortantes
#[derive(Serialize)]
pub struct DomainResponse {
    pub domain: String,
    pub description: String,
    pub aliases_count: i64,
    pub aliases_limit: i32,
    pub active: bool,
}
```

### Newtypes pour la sécurité des types

```rust
// Utiliser des newtypes pour les identifiants et valeurs métier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainName(String);

impl DomainName {
    pub fn new(name: &str) -> Result<Self, ValidationError> {
        // Validation RFC 1035
        validate_domain_name(name)?;
        Ok(Self(name.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

// Empêche de confondre un DomainName avec un EmailAddress au compile-time
```

### Validation

```rust
// Utiliser le crate `validator` pour la validation déclarative
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

// Pour les validations métier complexes, implémenter un trait custom
pub trait BusinessValidation {
    fn validate_business_rules(&self, ctx: &ValidationContext) -> Result<(), ValidationError>;
}
```

### Sérialisation

```rust
// Dériver Serialize/Deserialize seulement sur les types qui le nécessitent
// Les modèles de domaine ne devraient PAS dériver Serialize/Deserialize
// Seuls les DTO le font

// Les champs sensibles ne sont jamais sérialisés
#[derive(Serialize)]
pub struct MailboxResponse {
    pub username: String,
    pub name: String,
    #[serde(skip_serializing)]  // Jamais dans la réponse
    pub password: String,
}

// Utiliser rename_all pour la cohérence JSON
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiResponse<T> {
    pub data: T,
    pub meta: Option<PaginationMeta>,
}
```

---

## 5. Async et concurrence

### Runtime

- Utiliser `tokio` comme runtime async (feature `full`)
- Toutes les opérations I/O doivent être async
- Éviter `block_in_place` et `spawn_blocking` sauf pour le hashing de mots de passe

### Règles

```rust
// Les fonctions async doivent être marquées clairement
pub async fn find_domain(pool: &PgPool, name: &str) -> Result<Domain, DbError> {
    // ...
}

// Préférer les traits async (async-trait) quand nécessaire
#[async_trait::async_trait]
pub trait DomainRepository: Send + Sync {
    async fn find_by_name(&self, name: &str) -> Result<Option<Domain>, DbError>;
    async fn create(&self, domain: &CreateDomain) -> Result<Domain, DbError>;
    // ...
}

// Le hashing de mot de passe est CPU-intensif → spawn_blocking
pub async fn hash_password(password: &str) -> Result<String, AuthError> {
    let password = password.to_string();
    tokio::task::spawn_blocking(move || {
        argon2::hash_encoded(password.as_bytes(), &salt, &config)
    })
    .await
    .map_err(|_| AuthError::Internal)?
}
```

### État partagé

```rust
// Utiliser Arc pour l'état partagé entre les handlers
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<AppConfig>,
    pub domain_repo: Arc<dyn DomainRepository>,
    pub mailbox_repo: Arc<dyn MailboxRepository>,
    // ...
}

// L'état est passé via axum::extract::State
async fn list_domains(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<ApiResponse<Vec<DomainResponse>>>, ApiError> {
    // ...
}
```

### Annulation

- Les futures doivent être cancellation-safe
- Utiliser `tokio::select!` avec précaution
- Les transactions SQL doivent être commitées ou rollbackées même en cas d'annulation

---

## 6. Base de données et sqlx

### Pattern Repository

```rust
// Trait dans par-core (pas de dépendance sqlx)
#[async_trait::async_trait]
pub trait DomainRepository: Send + Sync {
    async fn find_all(&self, params: &ListParams) -> Result<Vec<Domain>, DbError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Domain>, DbError>;
    async fn create(&self, input: &CreateDomain) -> Result<Domain, DbError>;
    async fn update(&self, name: &str, input: &UpdateDomain) -> Result<Domain, DbError>;
    async fn delete(&self, name: &str) -> Result<(), DbError>;
    async fn count_by_domain(&self, name: &str) -> Result<DomainStats, DbError>;
}

// Implémentation PostgreSQL dans par-db
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

### Requêtes SQL

```rust
// TOUJOURS utiliser des requêtes paramétrées (jamais de format!/concat de SQL)
// Bon
sqlx::query!("SELECT * FROM domain WHERE domain = $1", domain_name)

// Interdit
sqlx::query(&format!("SELECT * FROM domain WHERE domain = '{}'", domain_name))
```

### Transactions

```rust
// Utiliser des transactions pour les opérations multi-tables
pub async fn create_mailbox(&self, input: &CreateMailbox) -> Result<Mailbox, DbError> {
    let mut tx = self.pool.begin().await?;

    // Créer la boîte mail
    let mailbox = sqlx::query_as!(...)
        .fetch_one(&mut *tx)
        .await?;

    // Créer l'alias automatique
    sqlx::query!(...)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(mailbox)
}
```

### Migrations

- Utiliser `sqlx-cli` pour les migrations
- Fichier par migration : `migrations/YYYYMMDDHHMMSS_description.sql`
- Chaque migration doit être réversible (fournir un `down.sql`)
- Tester les migrations sur les trois backends (PG, MySQL, SQLite)

---

## 7. API et handlers

### Structure d'un handler axum

```rust
/// Liste les domaines avec pagination et filtrage.
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

### Routage

```rust
// Regrouper les routes par ressource
pub fn domain_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_domains).post(create_domain))
        .route("/{domain}", get(show_domain).put(update_domain).delete(delete_domain))
        .route("/{domain}/active", patch(toggle_domain))
}

// Composer dans le routeur principal
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

## 8. Tests

### Structure des tests

```rust
// Tests unitaires — dans le même fichier
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
// Tests d'intégration — dans tests/ à la racine du crate
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
// Pour les tests d'intégration avec de vraies bases de données
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

### Règles de test

- Chaque fonction publique doit avoir au moins un test
- Les cas limites et les cas d'erreur doivent être testés
- Les tests doivent être indépendants et reproductibles
- Utiliser des fixtures/factories pour les données de test
- Nommer les tests de manière descriptive : `{action}_{condition}_{expected_result}`

---

## 9. Performance

### Allocation

- Préférer `&str` à `String` quand la propriété n'est pas nécessaire
- Utiliser `Cow<'_, str>` quand le choix owned/borrowed dépend du runtime
- Pré-allouer les `Vec` quand la taille est connue (`Vec::with_capacity`)
- Éviter les clones inutiles, préférer les références

### Requêtes SQL

- Toujours paginer les résultats (pas de `SELECT *` sans `LIMIT`)
- Utiliser `fetch_optional` plutôt que `fetch_one` + gestion d'erreur
- Les comptages doivent utiliser `COUNT(*)` et non charger tous les enregistrements
- Indexer les colonnes utilisées dans les `WHERE` et `ORDER BY`

### Hashing

- Le hashing de mot de passe est bloquant → `spawn_blocking`
- Cacher le résultat des validations de domaine DNS
- Utiliser des pools de connexions correctement dimensionnés

---

## 10. Sécurité

### Injection SQL
- Utiliser exclusivement les requêtes paramétrées sqlx
- Jamais de concaténation de chaînes dans les requêtes SQL

### Mots de passe
- Jamais loggés, jamais sérialisés dans les réponses API
- Comparaisons constantes en temps (timing-safe)
- Hash stocké, jamais le mot de passe en clair

### Sessions
- Cookies HttpOnly, Secure, SameSite=Strict
- Régénération après authentification
- CSRF token sur chaque formulaire POST

### Entrées utilisateur
- Toutes les entrées sont validées et sanitisées
- Askama échappe le HTML par défaut (XSS protection)
- Les paramètres d'URL sont validés avant utilisation

### Dépendances
- Audit régulier avec `cargo audit`
- Minimiser les dépendances
- Préférer les crates bien maintenues et auditées

---

## 11. Documentation

### Documentation de code

```rust
/// Crée un nouveau domaine dans le système.
///
/// Vérifie les limites globales et la validité du nom de domaine
/// avant l'insertion en base de données. Un log d'audit est créé.
///
/// # Errors
///
/// Retourne `DomainError::AlreadyExists` si le domaine existe déjà.
/// Retourne `DomainError::Validation` si le nom est invalide.
pub async fn create_domain(&self, input: CreateDomain) -> Result<Domain, DomainError> {
```

### Règles

- Documenter toutes les fonctions publiques avec `///`
- Les modules doivent avoir un `//!` en en-tête expliquant leur rôle
- Les `# Errors` et `# Panics` sont obligatoires quand applicables
- Les exemples dans la documentation (`# Examples`) sont encouragés
- Ne pas documenter ce qui est évident

---

## 12. Outillage et CI

### Formatage

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
```

- `cargo fmt` doit passer sans modifications
- Configuré dans le pre-commit hook

### Linting

```toml
# Clippy — dans Cargo.toml ou clippy.toml
[lints.clippy]
pedantic = "warn"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

- `cargo clippy -- -D warnings` doit passer sans erreur

### CI Pipeline

```
1. cargo fmt --check
2. cargo clippy -- -D warnings
3. cargo test
4. cargo audit
5. cargo build --release
6. Tests d'intégration (testcontainers)
```

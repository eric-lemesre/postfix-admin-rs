> **Language:** [English](../en/guidelines/GIT-WORKFLOW.md) | Francais

# Guidelines Git — postfix-admin-rs

## Modèle de branches

### Branches principales

| Branche | Rôle | Protection |
|---------|------|-----------|
| `main` | Production stable | Protégée, merge via PR uniquement |
| `develop` | Intégration des features | Protégée, merge via PR |

### Branches de travail

| Préfixe | Usage | Exemple |
|---------|-------|---------|
| `feature/` | Nouvelle fonctionnalité | `feature/domain-crud` |
| `fix/` | Correction de bug | `fix/login-session-timeout` |
| `refactor/` | Refactoring sans changement fonctionnel | `refactor/repository-traits` |
| `docs/` | Documentation uniquement | `docs/api-endpoints` |
| `chore/` | Maintenance, CI, dépendances | `chore/update-sqlx-0.8` |
| `release/` | Préparation d'une release | `release/v1.0.0` |
| `hotfix/` | Correctif urgent sur main | `hotfix/sql-injection-fix` |

### Flux de travail

```
feature/domain-crud ──PR──▶ develop ──PR──▶ main
                                              │
hotfix/critical-fix ──────────────────PR──▶ main
                                              │
                                     tag: v1.0.1
```

---

## Commits

### Format des messages

```
<type>(<scope>): <description>

[corps optionnel]

[footer optionnel]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | Nouvelle fonctionnalité |
| `fix` | Correction de bug |
| `docs` | Documentation |
| `style` | Formatage (pas de changement de code) |
| `refactor` | Refactoring (pas de changement fonctionnel) |
| `perf` | Amélioration de performance |
| `test` | Ajout ou modification de tests |
| `chore` | Maintenance (CI, dépendances, build) |
| `security` | Correction de sécurité |

### Scopes (périmètre)

| Scope | Crate/Module |
|-------|-------------|
| `core` | par-core |
| `db` | par-db |
| `auth` | par-auth |
| `api` | par-api |
| `web` | par-web |
| `cli` | par-cli |
| `server` | par-server |
| `ci` | CI/CD |
| `deps` | Dépendances |

### Exemples

```
feat(db): ajouter le repository PostgreSQL pour les domaines

Implémente PgDomainRepository avec les opérations CRUD.
Les requêtes utilisent sqlx::query_as! pour la vérification compile-time.

Closes #42
```

```
fix(auth): corriger le rehash transparent des mots de passe bcrypt

Le préfixe $2y$ n'était pas détecté correctement, causant un échec
de connexion pour les utilisateurs migrés depuis PostfixAdmin PHP.
```

```
chore(deps): mettre à jour axum 0.7 → 0.8

BREAKING CHANGE: les extracteurs State doivent utiliser la nouvelle API.
```

### Règles

- Le sujet fait 72 caractères maximum
- Le sujet utilise l'impératif présent ("ajouter", pas "ajouté" ou "ajout de")
- Pas de point final dans le sujet
- Le corps explique le **pourquoi**, pas le **quoi** (le diff montre le quoi)
- Référencer les issues avec `Closes #N` ou `Refs #N`
- Un commit = un changement logique (pas de commits fourre-tout)

---

## Pull Requests

### Checklist PR

```markdown
## Description
<!-- Qu'est-ce que cette PR fait et pourquoi ? -->

## Type de changement
- [ ] Nouvelle fonctionnalité (feature)
- [ ] Correction de bug (fix)
- [ ] Refactoring
- [ ] Documentation
- [ ] Maintenance / CI

## Checklist
- [ ] Le code compile sans warnings (`cargo build`)
- [ ] Les tests passent (`cargo test`)
- [ ] Clippy passe (`cargo clippy -- -D warnings`)
- [ ] Le formatage est correct (`cargo fmt --check`)
- [ ] La documentation est mise à jour si nécessaire
- [ ] Les migrations sont réversibles
- [ ] Pas de secrets ou credentials dans le code

## Tests
<!-- Comment tester cette PR ? -->
```

### Règles PR

- Une PR par fonctionnalité ou correction
- Le titre suit le même format que les commits
- Au moins un reviewer requis
- Tous les checks CI doivent passer
- Squash merge vers `develop` (historique propre)
- Merge commit vers `main` (conservation de l'historique)
- Les branches sont supprimées après merge

---

## Tags et releases

### Versioning sémantique (SemVer)

```
MAJOR.MINOR.PATCH

1.0.0  → Première release stable
1.1.0  → Nouvelle fonctionnalité (rétrocompatible)
1.1.1  → Correction de bug
2.0.0  → Changement cassant (breaking change)
```

### Processus de release

```bash
# 1. Créer la branche de release
git checkout -b release/v1.2.0 develop

# 2. Mettre à jour les versions dans Cargo.toml
# 3. Mettre à jour le CHANGELOG
# 4. Tester

# 5. Merger dans main
# 6. Tagger
git tag -a v1.2.0 -m "Release v1.2.0"

# 7. Merger dans develop (pour synchroniser)
# 8. Pousser les tags
git push origin v1.2.0
```

---

## .gitignore

```gitignore
# Rust
/target/
**/*.rs.bk
Cargo.lock    # Pas ignoré pour les binaires (inclus dans le repo)

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Environnement
.env
.env.local
config.local.toml

# Build frontend
node_modules/
static/css/output.css   # Généré par Tailwind

# Tests
*.profraw
lcov.info
```

---

## Hooks Git (pre-commit)

```bash
#!/bin/sh
# .githooks/pre-commit

set -e

echo "=== Vérification du formatage ==="
cargo fmt --check

echo "=== Clippy ==="
cargo clippy -- -D warnings

echo "=== Tests rapides ==="
cargo test --lib

echo "Toutes les vérifications ont réussi."
```

Installation : `git config core.hooksPath .githooks`

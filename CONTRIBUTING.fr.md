> **Language:** [English](CONTRIBUTING.md) | Francais

# Contribuer a postfix-admin-rs

Merci de votre interet pour contribuer a postfix-admin-rs ! Ce document
decrit le processus de contribution.

## Code de conduite

Ce projet suit le [Contributor Covenant](https://www.contributor-covenant.org/).
En participant, vous vous engagez a respecter ce code de conduite.

## Comment contribuer

### Signaler un bug

1. Verifiez que le bug n'a pas deja ete signale dans les [issues](https://github.com/eric-lemesre/postfix-admin-rs/issues)
2. Creez une issue avec :
   - Description claire du probleme
   - Etapes pour reproduire
   - Comportement attendu vs observe
   - Version de postfix-admin-rs, OS, base de donnees
   - Logs pertinents (sans donnees sensibles)

### Proposer une fonctionnalite

1. Ouvrez une issue avec le label `feature-request`
2. Decrivez le besoin et le cas d'utilisation
3. Discutez avec les mainteneurs avant de coder

### Soumettre du code

1. Forkez le repository
2. Creez une branche depuis `develop` :
   ```bash
   git checkout -b feature/ma-fonctionnalite develop
   ```
3. Ecrivez votre code en suivant les [guidelines](docs/fr/guidelines/)
4. Ecrivez des tests
5. Verifiez localement :
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
6. Commitez en suivant les [conventions de commit](docs/fr/guidelines/GIT-WORKFLOW.md)
7. Ouvrez une Pull Request vers `develop`

## Environnement de developpement

### Prerequis

- Rust stable (edition 2021, version 1.75+)
- Docker (pour testcontainers)
- PostgreSQL, MySQL ou SQLite (pour le dev local)
- Node.js 18+ (pour la compilation Tailwind CSS)

### Setup

```bash
# Cloner
git clone https://github.com/eric-lemesre/postfix-admin-rs.git
cd postfix-admin-rs

# Configurer les git hooks
git config core.hooksPath .githooks

# Copier la configuration de developpement
cp config/default.toml config/local.toml
# Editer config/local.toml avec vos parametres DB

# Compiler
cargo build

# Lancer les tests
cargo test

# Lancer le serveur de dev
cargo run --bin postfix-admin-server
```

### Base de donnees de developpement

```bash
# PostgreSQL via Docker
docker run -d --name postfix-admin-dev-pg \
    -e POSTGRES_DB=postfix \
    -e POSTGRES_USER=postfix \
    -e POSTGRES_PASSWORD=postfix \
    -p 5432:5432 \
    postgres:16-alpine

# Appliquer les migrations
cargo run --bin postfix-admin-cli -- migrate
```

## Standards de qualite

- Tout le code passe `cargo fmt` et `cargo clippy -- -D warnings`
- Les fonctions publiques sont documentees (`///`)
- Les nouvelles fonctionnalites ont des tests
- Les migrations SQL sont fournies pour les trois backends
- Le CHANGELOG est mis a jour

## Licence

En contribuant, vous acceptez que vos contributions soient distribuees
sous la licence GPLv3. Voir [LICENSE](LICENSE).

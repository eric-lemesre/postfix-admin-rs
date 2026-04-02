> **Language:** English | [Francais](CONTRIBUTING.fr.md)

---
# Contribute to postfix-admin-rs

Thank you for your interest in contributing to postfix-admin-rs! This document describes the contribution process.

## Code of conduct

This project follows the [Contributor Covenant](https://www.contributor-covenant.org/).
By participating, you agree to abide by this code of conduct.

## How to contribute

### Report a bug

1. Check that the bug hasn't already been reported in the [issues](https://github.com/eric-lemesre/postfix-admin-rs/issues)
2. Create an issue with:
   - Clear description of the problem
   - Steps to reproduce
   - Expected vs observed behavior
   - Version of postfix-admin-rs, OS, database
   - Relevant logs (without sensitive data)

### Suggest a feature

1. Open an issue with the `feature-request` label
2. Describe the need and use case
3. Discuss with maintainers before coding

### Submit code

1. Fork the repository
2. Create a branch from `develop`:
   ```bash
   git checkout -b feature/my-feature develop
   ```
3. Write your code following the [guidelines](docs/en/guidelines/)
4. Write tests
5. Check locally:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
6. Commit following the [commit conventions](docs/en/guidelines/GIT-WORKFLOW.md)
7. Open a Pull Request to `develop`

## Development environment

### Prerequisites

- Rust stable (edition 2021, version 1.75+)
- Docker (for testcontainers)
- PostgreSQL, MySQL or SQLite (for local dev)
- Node.js 18+ (for Tailwind CSS compilation)

### Setup

```bash
# Clone
git clone https://github.com/eric-lemesre/postfix-admin-rs.git
cd postfix-admin-rs

# Configure git hooks
git config core.hooksPath .githooks

# Copy development configuration
cp config/default.toml config/local.toml
# Edit config/local.toml with your DB parameters

# Build
cargo build

# Run tests
cargo test

# Launch dev server
cargo run --bin postfix-admin-server
```

### Development database

```bash
# PostgreSQL via Docker
docker run -d --name postfix-admin-dev-pg \
    -e POSTGRES_DB=postfix \
    -e POSTGRES_USER=postfix \
    -e POSTGRES_PASSWORD=postfix \
    -p 5432:5432 \
    postgres:16-alpine

# Apply migrations
cargo run --bin postfix-admin-cli -- migrate
```

## Quality standards

- All code passes `cargo fmt` and `cargo clippy -- -D warnings`
- Public functions are documented (`///`)
- New features have tests
- SQL migrations are provided for all three backends
- CHANGELOG is updated

## License

By contributing, you accept that your contributions will be distributed under the GPLv3 license. See [LICENSE](LICENSE).

---

# postfix-admin-rs task runner
# Usage: just <recipe>
# List all recipes: just --list

set dotenv-load := false

# Default recipe: show available commands
default:
    @just --list

# --------------------------------------------------------------------------
# Development
# --------------------------------------------------------------------------

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all --check

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all unit tests
test:
    cargo test --all

# Build in debug mode
build:
    cargo build --all

# Build in release mode
build-release:
    cargo build --release

# Run the server in development mode
run:
    RUST_LOG=debug cargo run --bin postfix-admin-server

# Run the CLI
cli *ARGS:
    cargo run --bin postfix-admin-cli -- {{ ARGS }}

# Clean build artifacts
clean:
    cargo clean

# --------------------------------------------------------------------------
# Quality assurance
# --------------------------------------------------------------------------

# Run the full CI pipeline locally
ci: fmt-check clippy test audit deny build-release
    @echo "CI pipeline passed."

# Check code quality (fmt + clippy)
check: fmt-check clippy

# Run security audit
audit:
    cargo audit

# Run cargo-deny checks (licenses, vulnerabilities)
deny:
    cargo deny check

# --------------------------------------------------------------------------
# Testing
# --------------------------------------------------------------------------

# Run tests with output shown
test-verbose:
    cargo test --all -- --nocapture

# Run a specific test by name
test-one NAME:
    cargo test --all -- {{ NAME }}

# Run integration tests (requires Docker)
test-integration:
    cargo test --all -- --ignored

# --------------------------------------------------------------------------
# Documentation
# --------------------------------------------------------------------------

# Generate and open documentation
doc:
    cargo doc --no-deps --all --open

# --------------------------------------------------------------------------
# Utilities
# --------------------------------------------------------------------------

# Show dependency tree
deps:
    cargo tree

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated -R

# Count lines of code (requires tokei)
loc:
    tokei crates/

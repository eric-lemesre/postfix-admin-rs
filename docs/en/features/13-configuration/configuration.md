> **Language:** English | [Francais](../fr/features/13-configuration/configuration.md)

---
# SPEC-13.1 — Configuration System

## Summary

Centralized configuration in TOML with multi-source support (file, environment
variables, CLI arguments). Based on the `config-rs` crate.

## Main Configuration File

Default path: `/etc/postfix-admin-rs/config.toml`

Override: `--config /path/to/config.toml` or `PAR_CONFIG=/path/to/config.toml`

## Resolution Priority

```
1. CLI Arguments (--database-url, etc.)        ← Highest priority
2. Environment Variables (PAR_*)
3. config.local.toml File (local overrides)
4. config.toml File (main configuration)
5. Compiled Default Values                ← Lowest priority
```

## Full Structure

```toml
# ============================================================
# postfix-admin-rs — Configuration
# ============================================================

[server]
bind_address = "0.0.0.0"
port = 8080
workers = 0                    # 0 = auto (number of CPUs)
base_url = "https://mail.example.com"
secret_key = ""                # Encryption key (generated at setup if empty)

[server.tls]
enabled = false
cert_path = "/etc/ssl/certs/mail.pem"
key_path = "/etc/ssl/private/mail.key"

[database]
# Supported formats:
# PostgreSQL: "postgresql://user:pass@host:5432/dbname"
# MySQL      : "mysql://user:pass@host:3306/dbname"
# SQLite     : "sqlite:///path/to/database.db"
url = "postgresql://postfix:password@localhost:5432/postfix"
max_connections = 10
min_connections = 2
connect_timeout_seconds = 5
idle_timeout_seconds = 300
# Optional prefix for table names
table_prefix = ""

[grpc]
enabled = false
bind_address = "0.0.0.0"
port = 50051
tls_enabled = false
tls_cert_path = ""
tls_key_path = ""

[auth]
# Session duration in seconds
session_lifetime = 3600
# Maximum login attempts per IP / 15 min
max_login_attempts = 5
# Lockout duration after exceeding (seconds)
lockout_duration = 900
# Hashing algorithm for new passwords
password_scheme = "argon2id"
# Allow plaintext (NEVER in production)
allow_cleartext = false

[auth.argon2]
memory_cost = 19456
time_cost = 2
parallelism = 1

[auth.jwt]
# JWT lifetime (seconds)
access_token_lifetime = 900
refresh_token_lifetime = 604800

[password_policy]
min_length = 8
max_length = 256
require_uppercase = true
require_lowercase = true
require_digit = true
require_special = false

[mail]
# SMTP configuration for outgoing emails (password retrieval, etc.)
smtp_host = "localhost"
smtp_port = 25
smtp_tls = false
smtp_username = ""
smtp_password = ""
from_address = "postmaster@example.com"
from_name = "PostfixAdmin"

[vacation]
enabled = true
domain = "autoreply.example.com"

[fetchmail]
enabled = true
# Minimum polling interval (minutes)
min_poll_interval = 5

[dkim]
enabled = true
# Default RSA key size
default_key_size = 2048

[logging]
# Level: trace, debug, info, warn, error
level = "info"
# Format: json, pretty, compact
format = "pretty"
# Audit log retention (days, 0 = unlimited)
audit_retention_days = 365
# Syslog
syslog_enabled = false
syslog_facility = "mail"

[ui]
# Items per page
page_size = 20
# Default language
default_language = "en"
# Available languages
available_languages = ["en", "fr"]
# Default theme: "light", "dark", "auto"
default_theme = "auto"
# Name displayed in the interface
site_name = "PostfixAdmin"

[domain_defaults]
aliases = 0
mailboxes = 0
maxquota = 0
quota = 0
transport = "virtual:"
backupmx = false

[security]
# DNS verification of domains
dns_check_enabled = true
# Restrict aliases to local domains only
local_alias_only = false
# HTTP security headers
csp_enabled = true
hsts_enabled = true
hsts_max_age = 31536000

[encryption]
# Encryption key for secrets (TOTP, fetchmail passwords)
# Must be exactly 32 bytes in base64
# Automatically generated at setup if empty
master_key = ""
```

## Environment Variables

All configuration values can be overridden by environment variables prefixed with `PAR_` using `__` as a level separator.

| Variable | Corresponds to |
|----------|---------------|
| `PAR_SERVER__PORT` | `server.port` |
| `PAR_DATABASE__URL` | `database.url` |
| `PAR_AUTH__SESSION_LIFETIME` | `auth.session_lifetime` |
| `PAR_LOGGING__LEVEL` | `logging.level` |

## Startup Validation

At launch, the configuration is validated:

1. `database.url` must be defined and valid
2. `server.secret_key` must be defined (or generated on first launch)
3. `encryption.master_key` must be defined (or generated on first launch)
4. TLS certificate paths must exist if TLS is enabled
5. `password_policy.min_length >= 6`
6. `auth.password_scheme` must be a supported scheme

Any validation error prevents startup with an explicit message.

---

> **Language:** English | [Francais](../fr/features/13-configuration/configuration.md)

---
# SPEC-13.1 — Configuration System

## Implementation Status

| Component                              | Crate                | Status    | Milestone |
|----------------------------------------|----------------------|-----------|-----------|
| Config struct definitions              | `postfix-admin-core` | Done      | M3        |
| config-rs integration                  | `postfix-admin-core` | Done      | M3        |
| Resolution priority (CLI > env > file) | `postfix-admin-core` | Done      | M3        |
| Startup validation                     | `postfix-admin-core` | Done      | M3        |
| Auto-generation of secrets             | `postfix-admin-core` | Done      | M3        |
| Profile system (dev/test/prep/prod)    | `postfix-admin-core` | Done      | M3        |
| Operating mode detection               | `postfix-admin-core` | Done      | M3        |
| Secrets file separation                | `postfix-admin-core` | Done      | M3        |

## Summary

Centralized configuration in TOML with multi-source support (file, environment
variables, CLI arguments). Based on the `config-rs` crate.

Two operating modes:
- **Development**: loads from `./config/` directory
- **Deployed**: loads from `/etc/postfix-admin-rs/` directory

Four profiles: `dev`, `test`, `prep`, `prod`.

## Configuration Files

### Development mode (`./config/` exists)

```
config/
├── default.toml     # Shared defaults (committed)
├── dev.toml         # Development profile (committed)
├── test.toml        # Test profile (committed)
├── prep.toml        # Pre-production profile (committed)
├── prod.toml        # Production profile (committed)
├── local.toml       # Personal overrides (.gitignore)
└── secrets.toml     # Dev secrets (.gitignore)
```

### Deployed mode (`/etc/postfix-admin-rs/` exists)

```
/etc/postfix-admin-rs/
├── config.toml        # Main configuration
├── config.local.toml  # Local overrides
└── secrets.toml       # Secrets (mode 0600)
```

## Resolution Priority

### Development mode

```
1. CLI Arguments (--database-url, etc.)        ← Highest priority
2. PAR_* Environment Variables
3. config/secrets.toml
4. config/local.toml
5. config/{profile}.toml
6. config/default.toml
7. Compiled Default Values                     ← Lowest priority
```

Additionally, `.env` files are loaded via `dotenvy` in dev/test profiles.

### Deployed mode

```
1. CLI Arguments                               ← Highest priority
2. PAR_* Environment Variables
3. /etc/postfix-admin-rs/secrets.toml
4. /etc/postfix-admin-rs/config.local.toml
5. /etc/postfix-admin-rs/config.toml
6. Compiled Default Values                     ← Lowest priority
```

## Full Structure

```toml
[server]
bind_address = "127.0.0.1"
port = 8080
workers = 0                    # 0 = auto (number of CPUs)
base_url = "https://mail.example.com"
secret_key = ""                # Encryption key (auto-generated in dev/test)

[server.tls]
enabled = false
cert_path = "/etc/ssl/certs/mail.pem"
key_path = "/etc/ssl/private/mail.key"

[database]
url = "postgresql://postfix_admin:password@localhost:5432/postfix_admin"
max_connections = 10
min_connections = 2
connect_timeout_seconds = 5
idle_timeout_seconds = 300
table_prefix = ""

[grpc]
enabled = false
bind_address = "0.0.0.0"
port = 50051
tls_enabled = false
tls_cert_path = ""
tls_key_path = ""

[auth]
session_lifetime = 3600
max_login_attempts = 5
lockout_duration = 900
password_scheme = "argon2id"
allow_cleartext = false

[auth.argon2]
memory_cost = 19456
time_cost = 2
parallelism = 1

[auth.jwt]
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
min_poll_interval = 5

[dkim]
enabled = true
default_key_size = 2048

[logging]
level = "info"
format = "pretty"
audit_retention_days = 365
syslog_enabled = false
syslog_facility = "mail"

[ui]
page_size = 20
default_language = "en"
available_languages = ["en", "fr"]
default_theme = "auto"
site_name = "PostfixAdmin"

[domain_defaults]
aliases = 0
mailboxes = 0
maxquota = 0
quota = 0
transport = "virtual:"
backupmx = false

[security]
dns_check_enabled = true
local_alias_only = false
csp_enabled = true
hsts_enabled = true
hsts_max_age = 31536000

[encryption]
master_key = ""
```

## Environment Variables

All configuration values can be overridden by environment variables prefixed with `PAR_` using `__` as a level separator.

| Variable                     | Corresponds to          |
|------------------------------|-------------------------|
| `PAR_PROFILE`                | Active profile          |
| `PAR_SERVER__PORT`           | `server.port`           |
| `PAR_DATABASE__URL`          | `database.url`          |
| `PAR_AUTH__SESSION_LIFETIME` | `auth.session_lifetime` |
| `PAR_LOGGING__LEVEL`         | `logging.level`         |

## Startup Validation

At launch, the configuration is validated contextually:

### Always validated

1. `database.url` must not be empty
2. `password_policy.min_length >= 4`
3. `auth.password_scheme` must be a supported scheme (`argon2id`, `bcrypt`, `sha512-crypt`, `sha256-crypt`)
4. `logging.level` must be valid (`trace`, `debug`, `info`, `warn`, `error`)

### Production-like (prep, prod, deployed)

- `auth.allow_cleartext = true` → **fatal error**
- `server.secret_key` empty → **fatal error**
- `encryption.master_key` empty → **fatal error**
- `server.tls.enabled = false` → **warning**
- `logging.level = trace|debug` → **warning**

### Dev/Test

- Empty `secret_key` / `master_key` → **auto-generated** (32 random bytes, base64-encoded)

Any validation error prevents startup with an explicit message.

---

> **Language:** [English](../en/features/13-configuration/configuration.md) | Francais

# SPEC-13.1 — Systeme de configuration

## Statut d'implementation

| Composant                                  | Crate                | Statut    | Milestone |
|--------------------------------------------|----------------------|-----------|-----------|
| Definitions des structs de config          | `postfix-admin-core` | Fait      | M3        |
| Integration config-rs                      | `postfix-admin-core` | Fait      | M3        |
| Priorite de resolution (CLI > env > file)  | `postfix-admin-core` | Fait      | M3        |
| Validation au demarrage                    | `postfix-admin-core` | Fait      | M3        |
| Auto-generation des secrets                | `postfix-admin-core` | Fait      | M3        |
| Systeme de profils (dev/test/prep/prod)    | `postfix-admin-core` | Fait      | M3        |
| Detection du mode de fonctionnement        | `postfix-admin-core` | Fait      | M3        |
| Separation du fichier de secrets           | `postfix-admin-core` | Fait      | M3        |

## Resume

Configuration centralisee en TOML avec support multi-sources (fichier, variables
d'environnement, arguments CLI). Base sur la crate `config-rs`.

Deux modes de fonctionnement :
- **Developpement** : charge depuis le repertoire `./config/`
- **Deploye** : charge depuis le repertoire `/etc/postfix-admin-rs/`

Quatre profils : `dev`, `test`, `prep`, `prod`.

## Fichiers de configuration

### Mode developpement (`./config/` existe)

```
config/
├── default.toml     # Defauts partages (commite)
├── dev.toml         # Profil developpement (commite)
├── test.toml        # Profil test (commite)
├── prep.toml        # Profil pre-production (commite)
├── prod.toml        # Profil production (commite)
├── local.toml       # Overrides personnels (.gitignore)
└── secrets.toml     # Secrets dev (.gitignore)
```

### Mode deploye (`/etc/postfix-admin-rs/` existe)

```
/etc/postfix-admin-rs/
├── config.toml        # Configuration principale
├── config.local.toml  # Overrides locaux
└── secrets.toml       # Secrets (mode 0600)
```

## Priorite de resolution

### Mode developpement

```
1. Arguments CLI (--database-url, etc.)        ← Plus haute priorite
2. Variables d'environnement PAR_*
3. config/secrets.toml
4. config/local.toml
5. config/{profil}.toml
6. config/default.toml
7. Valeurs par defaut compilees                ← Plus basse priorite
```

De plus, les fichiers `.env` sont charges via `dotenvy` en profils dev/test.

### Mode deploye

```
1. Arguments CLI                               ← Plus haute priorite
2. Variables d'environnement PAR_*
3. /etc/postfix-admin-rs/secrets.toml
4. /etc/postfix-admin-rs/config.local.toml
5. /etc/postfix-admin-rs/config.toml
6. Valeurs par defaut compilees                ← Plus basse priorite
```

## Structure complete

```toml
[server]
bind_address = "127.0.0.1"
port = 8080
workers = 0                    # 0 = auto (nombre de CPUs)
base_url = "https://mail.example.com"
secret_key = ""                # Cle de chiffrement (auto-generee en dev/test)

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
tls_ca_cert_path = ""
require_client_cert = false

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

[auth.mtls]
enabled = false
trusted_proxy_header = "X-SSL-Client-Verify"
subject_header = "X-SSL-Client-S-DN"
serial_header = "X-SSL-Client-Serial"
require_for_superadmin = false
require_for_domain_admin = false
cn_field = "emailAddress"

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

## Variables d'environnement

Toutes les valeurs de configuration peuvent etre surchargees par des variables
d'environnement prefixees par `PAR_` avec `__` comme separateur de niveaux.

| Variable                     | Correspond a            |
|------------------------------|-------------------------|
| `PAR_PROFILE`                | Profil actif            |
| `PAR_SERVER__PORT`           | `server.port`           |
| `PAR_DATABASE__URL`          | `database.url`          |
| `PAR_AUTH__SESSION_LIFETIME` | `auth.session_lifetime` |
| `PAR_LOGGING__LEVEL`         | `logging.level`         |
| `PAR_AUTH__MTLS__ENABLED`    | `auth.mtls.enabled`     |
| `PAR_GRPC__REQUIRE_CLIENT_CERT` | `grpc.require_client_cert` |

## Validation au demarrage

Au lancement, la configuration est validee contextuellement :

### Toujours validee

1. `database.url` ne doit pas etre vide
2. `password_policy.min_length >= 4`
3. `auth.password_scheme` doit etre un schema supporte (`argon2id`, `bcrypt`, `sha512-crypt`, `sha256-crypt`)
4. `logging.level` doit etre valide (`trace`, `debug`, `info`, `warn`, `error`)
5. Si `auth.mtls.enabled` : `trusted_proxy_header`, `subject_header` et `cn_field` ne doivent pas etre vides
6. Si `grpc.require_client_cert` : `tls_enabled` doit etre `true` et `tls_ca_cert_path` ne doit pas etre vide

### Production-like (prep, prod, deploye)

- `auth.allow_cleartext = true` → **erreur fatale**
- `server.secret_key` vide → **erreur fatale**
- `encryption.master_key` vide → **erreur fatale**
- `server.tls.enabled = false` → **warning**
- `logging.level = trace|debug` → **warning**
- `auth.mtls.enabled = false` → **warning** (recommande pour les comptes admin)
- `auth.mtls.enabled = true` et `require_for_superadmin = false` → **warning**

### Dev/Test

- `secret_key` / `master_key` vides → **auto-generes** (32 octets aleatoires, encodes en base64)

Toute erreur de validation empeche le demarrage avec un message explicite.

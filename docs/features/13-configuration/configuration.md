# SPEC-13.1 — Système de configuration

## Résumé

Configuration centralisée en TOML avec support multi-sources (fichier, variables
d'environnement, arguments CLI). Basé sur la crate `config-rs`.

## Fichier de configuration principal

Chemin par défaut : `/etc/postfix-admin-rs/config.toml`

Override : `--config /path/to/config.toml` ou `PAR_CONFIG=/path/to/config.toml`

## Priorité de résolution

```
1. Arguments CLI (--database-url, etc.)        ← Plus haute priorité
2. Variables d'environnement (PAR_*)
3. Fichier config.local.toml (overrides locaux)
4. Fichier config.toml (configuration principale)
5. Valeurs par défaut compilées                ← Plus basse priorité
```

## Structure complète

```toml
# ============================================================
# postfix-admin-rs — Configuration
# ============================================================

[server]
bind_address = "0.0.0.0"
port = 8080
workers = 0                    # 0 = auto (nombre de CPUs)
base_url = "https://mail.example.com"
secret_key = ""                # Clé de chiffrement (généré au setup si vide)

[server.tls]
enabled = false
cert_path = "/etc/ssl/certs/mail.pem"
key_path = "/etc/ssl/private/mail.key"

[database]
# Formats supportés :
# PostgreSQL : "postgresql://user:pass@host:5432/dbname"
# MySQL      : "mysql://user:pass@host:3306/dbname"
# SQLite     : "sqlite:///path/to/database.db"
url = "postgresql://postfix:password@localhost:5432/postfix"
max_connections = 10
min_connections = 2
connect_timeout_seconds = 5
idle_timeout_seconds = 300
# Préfixe optionnel pour les noms de tables
table_prefix = ""

[grpc]
enabled = false
bind_address = "0.0.0.0"
port = 50051
tls_enabled = false
tls_cert_path = ""
tls_key_path = ""

[auth]
# Durée de session en secondes
session_lifetime = 3600
# Nombre maximum de tentatives de login par IP / 15 min
max_login_attempts = 5
# Durée du blocage après dépassement (secondes)
lockout_duration = 900
# Algorithme de hachage pour les nouveaux mots de passe
password_scheme = "argon2id"
# Permettre le texte clair (JAMAIS en production)
allow_cleartext = false

[auth.argon2]
memory_cost = 19456
time_cost = 2
parallelism = 1

[auth.jwt]
# Durée de vie du JWT (secondes)
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
# Configuration SMTP pour les emails sortants (récupération mot de passe, etc.)
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
# Intervalle minimum de polling (minutes)
min_poll_interval = 5

[dkim]
enabled = true
# Taille par défaut des clés RSA
default_key_size = 2048

[logging]
# Niveau : trace, debug, info, warn, error
level = "info"
# Format : json, pretty, compact
format = "pretty"
# Rétention des logs d'audit (jours, 0 = illimité)
audit_retention_days = 365
# Syslog
syslog_enabled = false
syslog_facility = "mail"

[ui]
# Nombre d'éléments par page
page_size = 20
# Langue par défaut
default_language = "en"
# Langues disponibles
available_languages = ["en", "fr"]
# Thème par défaut : "light", "dark", "auto"
default_theme = "auto"
# Nom affiché dans l'interface
site_name = "PostfixAdmin"

[domain_defaults]
aliases = 0
mailboxes = 0
maxquota = 0
quota = 0
transport = "virtual:"
backupmx = false

[security]
# Vérification DNS des domaines
dns_check_enabled = true
# Restreindre les alias aux domaines locaux
local_alias_only = false
# Headers de sécurité HTTP
csp_enabled = true
hsts_enabled = true
hsts_max_age = 31536000

[encryption]
# Clé de chiffrement pour les secrets (TOTP, mots de passe fetchmail)
# Doit être exactement 32 octets en base64
# Généré automatiquement au setup si vide
master_key = ""
```

## Variables d'environnement

Toutes les valeurs de configuration peuvent être overridées par des variables
d'environnement préfixées par `PAR_` avec `__` comme séparateur de niveaux.

| Variable | Correspond à |
|----------|-------------|
| `PAR_SERVER__PORT` | `server.port` |
| `PAR_DATABASE__URL` | `database.url` |
| `PAR_AUTH__SESSION_LIFETIME` | `auth.session_lifetime` |
| `PAR_LOGGING__LEVEL` | `logging.level` |

## Validation au démarrage

Au lancement, la configuration est validée :

1. `database.url` doit être défini et valide
2. `server.secret_key` doit être défini (ou généré au premier lancement)
3. `encryption.master_key` doit être défini (ou généré au premier lancement)
4. Les chemins de certificats TLS doivent exister si TLS est activé
5. `password_policy.min_length >= 6`
6. `auth.password_scheme` doit être un schéma supporté

Toute erreur de validation empêche le démarrage avec un message explicite.

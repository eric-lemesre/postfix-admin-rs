> **Language:** English | [Francais](../fr/migration/MIGRATION-FROM-PHP.md)

---
# Migration Guide — PostfixAdmin PHP to postfix-admin-rs

## Prerequisites

- PostfixAdmin PHP version 3.3+ (recent database schema)
- Complete backup of the database
- Read access to the PostfixAdmin PHP configuration (`config.local.php`)
- The Postfix/Dovecot server continues to operate during migration

## Migration Strategy

The migration is designed to be **non-destructive** and **progressive**:

1. postfix-admin-rs connects to the existing database
2. SQL migrations add new columns (without breaking old ones)
3. PostfixAdmin PHP can continue to run in parallel during transition
4. Passwords are automatically rehashed upon logins

## Steps

### 1. Backup

```bash
# PostgreSQL
pg_dump -Fc postfix > postfix_backup_$(date +%Y%m%d).dump

# MySQL
mysqldump --single-transaction postfix > postfix_backup_$(date +%Y%m%d).sql

# SQLite
cp /path/to/postfixadmin.db /path/to/postfixadmin.db.backup
```

### 2. Installation of postfix-admin-rs

```bash
# From binary
wget https://github.com/eric-lemesre/PostfixAdminRust/releases/latest/download/postfix-admin-rs-linux-amd64
chmod +x postfix-admin-rs-linux-amd64
sudo mv postfix-admin-rs-linux-amd64 /usr/local/bin/postfix-admin-rs

# From packages
sudo dpkg -i postfix-admin-rs_1.0.0_amd64.deb
```

### 3. Configuration

Create `/etc/postfix-admin-rs/config.toml` from the existing PHP configuration:

| PostfixAdmin PHP (`config.local.php`) | postfix-admin-rs (`config.toml`) |
|---------------------------------------|----------------------------------|
| `$CONF['database_type'] = 'pgsql'` | `database.url = "postgresql://..."` |
| `$CONF['database_host']` | Included in `database.url` |
| `$CONF['database_user']` | Included in `database.url` |
| `$CONF['database_password']` | Included in `database.url` |
| `$CONF['database_name']` | Included in `database.url` |
| `$CONF['encrypt'] = 'dovecot:BLF-CRYPT'` | `auth.password_scheme = "argon2id"` |
| `$CONF['page_size']` | `ui.page_size` |
| `$CONF['vacation_domain']` | `vacation.domain` |

### 4. Configuration Verification

```bash
postfix-admin-rs config check
```

### 5. Schema Migration

```bash
# View migration status
postfix-admin-rs migrate --status

# Apply migrations
postfix-admin-rs migrate
```

Migrations add:
- Missing columns (with default values)
- New indexes
- New tables (if necessary)
- They do not delete any existing columns

### 6. Parallel Testing

Start postfix-admin-rs on a different port:

```bash
PAR_SERVER__PORT=8081 postfix-admin-rs serve
```

Verify:
- Domain list is correct
- Mailboxes are accessible
- Aliases are correct
- Admin login works (password is automatically rehashed)

### 7. Cutover

```bash
# Stop PostfixAdmin PHP (disable Apache/Nginx vhost)
sudo a2dissite postfixadmin
sudo systemctl reload apache2

# Start postfix-admin-rs on production port
sudo systemctl enable --now postfix-admin-rs
```

### 8. Postfix/Dovecot Configuration

Postfix and Dovecot SQL queries **do not need to be modified**
as the base schema is compatible. The `domain`, `mailbox`, `alias`
tables keep the same columns and indexes.

## Password Compatibility

| PostfixAdmin PHP Format | postfix-admin-rs Support |
|-------------------------|--------------------------|
| `{BLF-CRYPT}$2y$...` | Read + auto rehash |
| `$2y$...` (raw bcrypt) | Read + auto rehash |
| `{SHA512-CRYPT}$6$...` | Read + auto rehash |
| `{MD5-CRYPT}$1$...` | Read + auto rehash |
| `{PLAIN-MD5}...` | Read + auto rehash |
| `{CRYPT}...` | Read + auto rehash |

Upon the first successful login of a user, their password is automatically
rehashed to argon2id (default scheme).

## Rollback

In case of problems:

```bash
# Stop postfix-admin-rs
sudo systemctl stop postfix-admin-rs

# Restore backup (if migrations caused issues)
pg_restore -d postfix postfix_backup_YYYYMMDD.dump

# Re-enable PostfixAdmin PHP
sudo a2ensite postfixadmin
sudo systemctl reload apache2
```

## Points of Attention

- **Table Prefix**: If PostfixAdmin PHP uses table prefixes,
  configure it in `database.table_prefix`
- **Vacation**: The vacation domain (`autoreply.example.com`) must remain the same
- **Fetchmail**: Remote passwords are stored differently (AES-256-GCM encryption
  instead of near-plaintext from PHP). A specific migration converts them
- **TOTP**: If 2FA was already active in PostfixAdmin PHP, TOTP secrets
  are preserved and re-encrypted with the new key

---

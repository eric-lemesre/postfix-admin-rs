> **Language:** English | [Francais](../fr/features/11-cli/cli-administration.md)

---
# SPEC-11.1 — Command Line Interface (CLI)

## Implementation Status

| Component | Crate | Status | Milestone |
|-----------|-------|--------|-----------|
| clap command structure | `postfix-admin-cli` | Pending | M8 |
| Domain/Mailbox/Alias/Admin commands | `postfix-admin-cli` | Pending | M8 |
| Utility commands (setup, migrate, etc.) | `postfix-admin-cli` | Pending | M8 |
| Output formatters (table, JSON, CSV) | `postfix-admin-cli` | Pending | M8 |

## Summary

CLI for server mail administration without web interface.
Based on `clap`, it offers an ergonomic interface with subcommands,
autocompletion and formatted output (table, JSON, CSV).

## Commands

### General structure

```
postfix-admin-rs <SUB-COMMAND> <ACTION> [OPTIONS]
```

### Domain management

```
postfix-admin-rs domain list [--active] [--format json|table|csv]
postfix-admin-rs domain show <DOMAIN>
postfix-admin-rs domain add <DOMAIN> [--description "..."] [--aliases 0] [--mailboxes 0]
                            [--maxquota 0] [--quota 0] [--transport "virtual:"]
                            [--backupmx] [--no-active]
postfix-admin-rs domain edit <DOMAIN> [--description "..."] [--aliases N] ...
postfix-admin-rs domain delete <DOMAIN> [--force]
postfix-admin-rs domain toggle <DOMAIN>
```

### Mailbox management

```
postfix-admin-rs mailbox list [--domain <DOMAIN>] [--active] [--format json|table|csv]
postfix-admin-rs mailbox show <USERNAME>
postfix-admin-rs mailbox add <USERNAME> --password <PASS> [--name "..."] [--quota N]
                             [--no-active]
postfix-admin-rs mailbox edit <USERNAME> [--name "..."] [--quota N]
postfix-admin-rs mailbox delete <USERNAME> [--force]
postfix-admin-rs mailbox toggle <USERNAME>
postfix-admin-rs mailbox password <USERNAME> --password <NEW_PASS>
postfix-admin-rs mailbox quota <USERNAME>
```

### Alias management

```
postfix-admin-rs alias list [--domain <DOMAIN>] [--format json|table|csv]
postfix-admin-rs alias show <ADDRESS>
postfix-admin-rs alias add <ADDRESS> --goto <DEST1,DEST2,...>
postfix-admin-rs alias edit <ADDRESS> --goto <DEST1,DEST2,...>
postfix-admin-rs alias delete <ADDRESS> [--force]
```

### Admin management

```
postfix-admin-rs admin list [--format json|table|csv]
postfix-admin-rs admin show <USERNAME>
postfix-admin-rs admin add <USERNAME> --password <PASS> [--superadmin]
                           [--domains domain1,domain2]
postfix-admin-rs admin edit <USERNAME> [--superadmin] [--no-superadmin]
postfix-admin-rs admin delete <USERNAME> [--force]
postfix-admin-rs admin password <USERNAME> --password <NEW_PASS>
postfix-admin-rs admin domains <USERNAME> --add <DOMAIN>
postfix-admin-rs admin domains <USERNAME> --remove <DOMAIN>
```

### Utilities

```
postfix-admin-rs setup                          # Initial setup (first superadmin)
postfix-admin-rs migrate                        # Run database migrations
postfix-admin-rs migrate --status               # View migration status
postfix-admin-rs config check                   # Check configuration
postfix-admin-rs config show                    # Show active configuration
postfix-admin-rs hash-password <PASSWORD>        # Hash a password
postfix-admin-rs verify-password <HASH> <PASS>   # Verify a hash
postfix-admin-rs export [--domain <DOMAIN>]      # Full JSON export
postfix-admin-rs import <FILE>                   # Import from JSON/CSV
postfix-admin-rs log list [--domain <D>] [--action <A>] [--last N]
postfix-admin-rs stats [--domain <DOMAIN>]       # Statistics
postfix-admin-rs completion <SHELL>              # Autocompletion generation
```

## Global options

| Option | Short | Description |
|--------|-------|-------------|
| `--config` | `-c` | Path to configuration file |
| `--database-url` | | Database connection URL (overrides config) |
| `--format` | `-f` | Output format: `table` (default), `json`, `csv` |
| `--quiet` | `-q` | Minimal output |
| `--verbose` | `-v` | Detailed output (-vv for debug) |
| `--yes` | `-y` | Automatically confirm destructive actions |
| `--color` | | Force/disable colors (`auto`, `always`, `never`) |

## Output format

### Table (default)
```
DOMAIN          DESCRIPTION     ALIASES  MAILBOXES  QUOTA   ACTIVE
example.com     Example Domain  10/50    5/100      2G/10G  yes
other.com       Other Domain    3/0      12/0       0/0     yes
```

### JSON
```json
[
  {
    "domain": "example.com",
    "description": "Example Domain",
    "aliases_count": 10,
    "aliases_limit": 50,
    "active": true
  }
]
```

### CSV
```
domain,description,aliases_count,aliases_limit,active
example.com,Example Domain,10,50,true
```

## CLI Configuration

The CLI uses the same configuration file as the web server.
It can also connect directly to the database without going through
the API (direct mode).

```toml
# Resolution priority:
# 1. --database-url command line option
# 2. DATABASE_URL environment variable
# 3. Configuration file (--config or /etc/postfix-admin-rs/config.toml)
```

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Configuration error |
| `3` | Database connection error |
| `4` | Validation error |
| `5` | Resource not found |
| `10` | Action cancelled by user |

---

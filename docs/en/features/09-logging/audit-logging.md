> **Language:** English | [Francais](../fr/features/09-logging/audit-logging.md)

---
# SPEC-09.1 — Audit log

## Implementation Status

| Component | Crate | Status | Milestone |
|-----------|-------|--------|-----------|
| Model (`Log`) | `postfix-admin-core` | Done | M1 |
| DTOs (`CreateLog`, `LogResponse`, `LogFilter`) | `postfix-admin-core` | Done | M1 |
| Repository trait (`LogRepository`) | `postfix-admin-core` | Done | M1 |
| PostgreSQL repository | `postfix-admin-db` | Pending | M2 |
| MySQL repository | `postfix-admin-db` | Pending | M2 |
| REST API endpoints | `postfix-admin-api` | Pending | M6 |
| Web UI log viewer | `postfix-admin-web` | Pending | M5 |

## Summary

System for logging all administrative actions performed via the web interface,
API or CLI. Enables complete traceability of changes made to the mail server
configuration.

## Entity: `Log`

| Field | Type | Constraint | Description |
|-------|------|-----------|-------------|
| `id` | `BIGSERIAL` | PK | Auto-incremented identifier |
| `timestamp` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date and time of the action |
| `username` | `VARCHAR(255)` | NOT NULL | Author of the action |
| `domain` | `VARCHAR(255)` | NOT NULL | Domain concerned |
| `action` | `VARCHAR(255)` | NOT NULL | Type of action |
| `data` | `TEXT` | NOT NULL, default `''` | Details of the action |
| `ip_address` | `VARCHAR(46)` | NULLABLE | Source IP address |
| `user_agent` | `VARCHAR(512)` | NULLABLE | User-Agent (web) or identifier (API/CLI) |

### Index

- `idx_log_timestamp` : `(timestamp)` — Chronological queries
- `idx_log_domain` : `(domain)` — Filtering by domain
- `idx_log_username` : `(username)` — Filtering by admin

## Logged actions

| Action | Description | Typical data |
|--------|-------------|-----------------|
| `create_domain` | Domain creation | Domain name, parameters |
| `edit_domain` | Domain modification | Modified fields |
| `delete_domain` | Domain deletion | Domain name |
| `create_mailbox` | Mailbox creation | Username, quota |
| `edit_mailbox` | Mailbox modification | Modified fields (never the password) |
| `delete_mailbox` | Mailbox deletion | Username |
| `create_alias` | Alias creation | Address → destinations |
| `edit_alias` | Alias modification | New destinations |
| `delete_alias` | Alias deletion | Address |
| `edit_vacation` | Vacation modification | Activation/deactivation |
| `create_admin` | Admin creation | Username, assigned domains |
| `edit_admin` | Admin modification | Modified fields |
| `delete_admin` | Admin deletion | Username |
| `login_success` | Successful login | IP, user-agent |
| `login_failure` | Login failure | IP, user-agent, reason |
| `password_change` | Password change | Username (never the password) |
| `totp_enable` | 2FA activation | Username |
| `totp_disable` | 2FA deactivation | Username |
| `create_dkim` | DKIM key creation | Domain, selector |
| `toggle_active` | Activation/deactivation | Entity, old state → new state |

## Business rules

### BR-LOG-01: Recording
- Every modification action (CREATE, UPDATE, DELETE) is logged
- Authentications (successes and failures) are logged
- Reads (GET/LIST) are NOT logged (performance)
- Passwords are NEVER included in log data

### BR-LOG-02: Consultation
- Superadmin: access to all logs
- Domain admin: access to logs of their domains only
- User: no access to logs
- Filterable by: period, domain, action, username
- Paginated (default: 50 entries per page)

### BR-LOG-03: Retention
- Configurable retention duration (default: 365 days)
- Automatic cleanup of old entries (periodic task)
- Option to export in CSV/JSON before purge

### BR-LOG-04: Syslog integration
- Optionally, audit logs are also sent to syslog
- Structured format compatible with analysis tools (ELK, Loki, etc.)
- Configurable syslog level (default: LOG_INFO)

## Use cases

### UC-LOG-01: Consult the log
- **Actor**: Superadmin, Domain admin
- **Input**: Filters (period, domain, action, user), pagination
- **Output**: Chronological list of actions

### UC-LOG-02: Export logs
- **Actor**: Superadmin
- **Input**: Filters, format (CSV, JSON)
- **Output**: Downloadable file

## Web routes

| Route | Method | Description |
|-------|---------|-------------|
| `/admin/logs` | GET | Log consultation (paginated, filterable) |
| `/admin/logs/export` | GET | CSV/JSON export |

## API endpoints

| Method | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/logs` | List logs (paginated, filterable) |
| `GET` | `/api/v1/domains/{domain}/logs` | Logs of a domain |

---

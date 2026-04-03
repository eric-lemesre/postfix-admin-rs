> **Language:** English | [Francais](../fr/features/02-mailboxes/quota-management.md)

---
# SPEC-02.2 — Quota Management

## Implementation Status

| Component                                | Crate                | Status  | Milestone |
|------------------------------------------|----------------------|---------|-----------|
| Models (`Quota`, `Quota2`)               | `postfix-admin-core` | Done    | M1        |
| DTOs (`QuotaResponse`, `Quota2Response`) | `postfix-admin-core` | Done    | M1        |
| Validation (quota within domain)         | `postfix-admin-core` | Done    | M1        |
| PostgreSQL repository                    | `postfix-admin-db`   | Pending | M2        |
| MySQL repository                         | `postfix-admin-db`   | Pending | M2        |
| Web UI (quota display, charts)           | `postfix-admin-web`  | Pending | M5        |

## Summary

Two-level quota system (domain and individual mailbox) integrated with
Dovecot for real-time tracking of storage usage.

## Entities

### `Quota` (Dovecot tracking - legacy table)

| Field      | Type           | Constraint     | Description            |
|------------|----------------|----------------|------------------------|
| `username` | `VARCHAR(255)` | PK (composite) | Email address          |
| `path`     | `VARCHAR(100)` | PK (composite) | Storage path           |
| `current`  | `BIGINT`       | default `0`    | Current usage in bytes |

### `Quota2` (Dovecot tracking >= 1.2)

| Field      | Type           | Constraint            | Description        |
|------------|----------------|-----------------------|--------------------|
| `username` | `VARCHAR(100)` | PK                    | Email address      |
| `bytes`    | `BIGINT`       | default `0`           | Bytes used         |
| `messages` | `INTEGER`      | NOT NULL, default `0` | Number of messages |

## Quota Levels

### Mailbox quota
- Defined in `mailbox.quota` (in bytes)
- `0` = unlimited (or limited only by domain quota)
- Cannot exceed `domain.maxquota` (if this > 0)

### Domain quota
- Defined in `domain.quota` (in MB)
- Sum of all mailboxes quotas for the domain
- `0` = unlimited

### Max quota per mailbox (domain level)
- Defined in `domain.maxquota` (in MB)
- Individual ceiling for each mailbox in the domain
- `0` = no individual ceiling

## Business Rules

### BR-QUO-01: Verification on creation/modification
```
If domain.maxquota > 0:
    mailbox.quota <= domain.maxquota * 1024 * 1024

If domain.quota > 0:
    SUM(mailbox.quota for the domain) <= domain.quota * 1024 * 1024
```

### BR-QUO-02: Display
- Automatic unit conversion (bytes → KB → MB → GB)
- Visual progress bar (green < 70%, orange 70-90%, red > 90%)
- Special indicator for mailboxes that have exceeded their quota

### BR-QUO-03: Dovecot Integration
- `quota` and `quota2` tables are managed by Dovecot (read-only from app)
- The application reads these tables to display current usage
- Dovecot configuration points to these tables for enforcement

## Use Cases

### UC-QUO-01: View a domain's quota usage
- **Actor**: Superadmin, Domain Admin
- **Output**: Summary table per mailbox, domain total, percentages

### UC-QUO-02: View own quota
- **Actor**: User
- **Output**: Current usage, limit, percentage, number of messages

### UC-QUO-03: Modify a mailbox's quota
- **Actor**: Superadmin, Domain Admin
- **Validation**: BR-QUO-01

## API Endpoints

| Method | Route                                | Description                 |
|--------|--------------------------------------|-----------------------------|
| `GET`  | `/api/v1/domains/{domain}/quota`     | Domain quota summary        |
| `GET`  | `/api/v1/mailboxes/{username}/quota` | Detailed quota of a mailbox |

---

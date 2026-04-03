> **Language:** English | [Francais](../fr/features/03-aliases/alias-management.md)

---
# SPEC-03.1 — Alias Management

## Implementation Status

| Component | Crate | Status | Milestone |
|-----------|-------|--------|-----------|
| Model (`Alias`) | `postfix-admin-core` | Done | M1 |
| DTOs (`CreateAlias`, `UpdateAlias`, `AliasResponse`) | `postfix-admin-core` | Done | M1 |
| Repository trait (`AliasRepository`) | `postfix-admin-core` | Done | M1 |
| Validation (alias destinations) | `postfix-admin-core` | Done | M1 |
| PostgreSQL repository | `postfix-admin-db` | Pending | M2 |
| MySQL repository | `postfix-admin-db` | Pending | M2 |
| REST API endpoints | `postfix-admin-api` | Pending | M6 |
| Web UI pages | `postfix-admin-web` | Pending | M5 |
| CLI commands | `postfix-admin-cli` | Pending | M8 |

## Summary

Aliases define email redirection rules. An alias maps a source address to one or more destination addresses. It's the fundamental mechanism for virtual mail routing in Postfix.

## Entity: `Alias`

| Field | Type | Constraint | Description |
|-------|------|-----------|-------------|
| `address` | `VARCHAR(255)` | PK | Source address (e.g., `info@example.com`) |
| `goto` | `TEXT` | NOT NULL | Comma-separated destinations |
| `domain` | `VARCHAR(255)` | FK → `domain.domain` | Alias domain |
| `active` | `BOOLEAN` | NOT NULL, default `true` | Active/inactive |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Creation date |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Last update |

### Index

- `idx_alias_address_active` : `(address, active)` — Postfix lookup
- `idx_alias_domain` : `(domain)`

## Alias Types

### Standard alias
- One address → one or more destination addresses
- Example: `info@example.com` → `alice@example.com,bob@example.com`

### Catch-all alias
- Address in the form `@example.com` (no local part)
- Captures all undelivered mail for the domain
- Low priority: explicit aliases and mailboxes are checked first

### Automatic alias (mailbox)
- Automatically created when a mailbox is created
- `user@example.com` → `user@example.com`
- Required for Postfix to route mail to Dovecot

### External redirection alias
- Destination to a non-locally managed domain
- Example: `forward@example.com` → `user@gmail.com`
- May be restricted by configuration (`emailcheck_localaliasonly`)

## Business Rules

### Creation (BR-ALI-01)
- The source address must be valid (RFC 5321) or in the form `@domain` (catch-all)
- The domain of the source address must exist in the `domain` table
- The number of aliases for the domain must not exceed `domain.aliases` (if > 0)
- Each destination must be a valid email address
- If `emailcheck_localaliasonly` is enabled, destinations must be local domains
- No direct loop: `a@x.com → a@x.com` (except automatic mailbox alias)

### Modification (BR-ALI-02)
- The source address cannot be modified (immutable PK)
- Destinations are fully replaced (no individual add/remove via SQL)

### Deletion (BR-ALI-03)
- Automatic mailbox aliases can only be deleted by deleting the mailbox
- Deleting an alias does not affect destination mailboxes

### `goto` Format (BR-ALI-04)
- Comma as separator: `dest1@x.com,dest2@y.com`
- No spaces around commas in storage
- UI displays one destination per line
- Maximum number of destinations configurable (default: 100)

## Use Cases

### UC-ALI-01: List aliases for a domain
- **Actor**: Superadmin, Domain admin
- **Input**: Domain, filters (search, active, type), pagination
- **Output**: List with address, truncated destinations, status
- **Note**: Automatic mailbox aliases are hidden by default (toggle)

### UC-ALI-02: Create an alias
- **Actor**: Superadmin, Domain admin
- **Input**: Source address (local_part + domain), destinations (textarea, one per line)
- **Validation**: BR-ALI-01

### UC-ALI-03: Modify alias destinations
- **Actor**: Superadmin, Domain admin
- **Input**: New destinations
- **Validation**: BR-ALI-02

### UC-ALI-04: Delete an alias
- **Actor**: Superadmin, Domain admin
- **Validation**: BR-ALI-03

## API Endpoints

| Method | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/domains/{domain}/aliases` | List aliases |
| `GET` | `/api/v1/aliases/{address}` | Alias details |
| `POST` | `/api/v1/domains/{domain}/aliases` | Create alias |
| `PUT` | `/api/v1/aliases/{address}` | Update alias |
| `DELETE` | `/api/v1/aliases/{address}` | Delete alias |
| `PATCH` | `/api/v1/aliases/{address}/active` | Activate/deactivate |

## Postfix Integration Notes

The typical SQL query for the Postfix `virtual_alias_maps` lookup:

```sql
SELECT goto FROM alias WHERE address = '%s' AND active = true
UNION
SELECT goto FROM alias WHERE address = '@%d' AND active = true
```

The second part handles catch-all. The UNION order ensures explicit aliases are returned first.

---

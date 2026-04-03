> **Language:** English | [Francais](../fr/features/01-domains/alias-domains.md)

---
# SPEC-01.2 — Alias Domains

## Implementation Status

| Component                                         | Crate                  | Status  | Milestone |
|---------------------------------------------------|------------------------|---------|-----------|
| Model (`AliasDomain`)                             | `postfix-admin-core`   | Done    | M1        |
| DTOs (`CreateAliasDomain`, `AliasDomainResponse`) | `postfix-admin-core`   | Done    | M1        |
| Repository trait (`AliasDomainRepository`)        | `postfix-admin-core`   | Done    | M1        |
| PostgreSQL repository                             | `postfix-admin-db`     | Pending | M2        |
| MySQL repository                                  | `postfix-admin-db`     | Pending | M2        |
| REST API endpoints                                | `postfix-admin-api`    | Pending | M6        |
| Web UI pages                                      | `postfix-admin-web`    | Pending | M5        |
| CLI commands                                      | `postfix-admin-cli`    | Pending | M8        |
| Routing logic                                     | `postfix-admin-server` | Pending | M13       |

## Summary

An alias domain redirects all mail from one domain to another domain.
For example, `alias-example.com` → `example.com`: any email sent to `user@alias-example.com`
is delivered to `user@example.com`.

## Entity: `AliasDomain`

| Field           | Type           | Constraint                | Description           |
|-----------------|----------------|---------------------------|-----------------------|
| `alias_domain`  | `VARCHAR(255)` | PK                        | Source domain (alias) |
| `target_domain` | `VARCHAR(255)` | FK → `domain.domain`      | Target domain         |
| `active`        | `BOOLEAN`      | NOT NULL, default `true`  | Active/inactive       |
| `created_at`    | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Creation date         |
| `updated_at`    | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Last update           |

### Index

- `idx_alias_domain_active`: `(alias_domain, active)`
- `idx_alias_domain_target`: `(target_domain)`

## Business Rules

### BR-ADOM-01: Creation
- The alias domain must not exist as a real domain in the `domain` table
- The alias domain must not already exist in `alias_domain`
- The target domain must exist in the `domain` table and be active
- No loops: the target domain cannot itself be a domain alias
- DNS validation identical to domains (RFC 1035)

### BR-ADOM-02: Deletion
- Deletion is simple (no complex cascade)
- Mail for the alias domain will no longer be delivered

### BR-ADOM-03: Interaction with classic aliases
- Classic aliases (table `alias`) take precedence over domain aliases
- If `admin@alias-example.com` has an explicit alias, it is used
  instead of redirecting to `admin@example.com`

## Use Cases

### UC-ADOM-01: List alias domains
- **Actor**: Superadmin, Admin of concerned domains
- **Output**: Paginated list (alias_domain → target_domain, status)

### UC-ADOM-02: Create an alias domain
- **Actor**: Superadmin
- **Input**: Alias domain + target domain (select from existing domains)
- **Validation**: BR-ADOM-01

### UC-ADOM-03: Delete an alias domain
- **Actor**: Superadmin
- **Validation**: BR-ADOM-02, confirmation required

## API Endpoints

| Method   | Route                                         | Description            |
|----------|-----------------------------------------------|------------------------|
| `GET`    | `/api/v1/alias-domains`                       | List alias domains     |
| `POST`   | `/api/v1/alias-domains`                       | Create an alias domain |
| `DELETE` | `/api/v1/alias-domains/{alias_domain}`        | Delete                 |
| `PATCH`  | `/api/v1/alias-domains/{alias_domain}/active` | Activate/deactivate    |

---

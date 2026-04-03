> **Language:** English | [Francais](../fr/features/01-domains/domain-management.md)

# SPEC-01.1 — Virtual Domain Management

## Implementation Status

| Component                                               | Crate                | Status  | Milestone |
|---------------------------------------------------------|----------------------|---------|-----------|
| Model (`Domain`)                                        | `postfix-admin-core` | Done    | M1        |
| DTOs (`CreateDomain`, `UpdateDomain`, `DomainResponse`) | `postfix-admin-core` | Done    | M1        |
| Repository trait (`DomainRepository`)                   | `postfix-admin-core` | Done    | M1        |
| Validation (quotas, limits, active check)               | `postfix-admin-core` | Done    | M1        |
| PostgreSQL repository                                   | `postfix-admin-db`   | Done    | M2        |
| MySQL repository                                        | `postfix-admin-db`   | Done    | M2        |
| REST API endpoints                                      | `postfix-admin-api`  | Pending | M6        |
| Web UI pages                                            | `postfix-admin-web`  | Pending | M5        |
| CLI commands                                            | `postfix-admin-cli`  | Pending | M8        |

## Summary

CRUD management of virtual email domains hosted by the Postfix server.
Each domain defines limits (number of aliases, mailboxes, quotas) and transport
parameters.

## Entity: `Domain`

| Field             | Type           | Constraint                | Description                                           |
|-------------------|----------------|---------------------------|-------------------------------------------------------|
| `domain`          | `VARCHAR(255)` | PK                        | Domain name (e.g., `example.com`)                     |
| `description`     | `VARCHAR(255)` | NOT NULL, default `''`    | Free description                                      |
| `aliases`         | `INTEGER`      | NOT NULL, default `0`     | Alias limit (0 = unlimited)                           |
| `mailboxes`       | `INTEGER`      | NOT NULL, default `0`     | Mailbox limit (0 = unlimited)                         |
| `maxquota`        | `BIGINT`       | NOT NULL, default `0`     | Max quota per mailbox in MB (0 = unlimited)           |
| `quota`           | `BIGINT`       | NOT NULL, default `0`     | Total domain quota in MB (0 = unlimited)              |
| `transport`       | `VARCHAR(255)` | NULLABLE                  | Postfix transport (e.g., `virtual:`, `lmtp:unix:...`) |
| `backupmx`        | `BOOLEAN`      | NOT NULL, default `false` | Server is backup MX for this domain                   |
| `password_expiry` | `INTEGER`      | NOT NULL, default `0`     | Password expiry in days (0 = disabled)                |
| `active`          | `BOOLEAN`      | NOT NULL, default `true`  | Domain active/inactive                                |
| `created_at`      | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Creation date                                         |
| `updated_at`      | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Last modification                                     |

### Index

- `idx_domain_active` : `(domain, active)` — Used by Postfix lookups

## Business Rules

### Creation (BR-DOM-01)
- The domain name must comply with RFC 1035 (regex: `^([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$`)
- Optional DNS verification: at least one A, AAAA, MX or NS record
- A domain cannot be created if it already exists as a domain or domain alias
- Only superadmins and admins with the `domain:create` right can create a domain

### Modification (BR-DOM-02)
- The domain name cannot be modified after creation (immutable PK)
- Reducing limits (aliases, mailboxes) does not delete existing entities but prevents creating new ones
- Total quota modification is propagated in Dovecot quota checks

### Deletion (BR-DOM-03)
- Deleting a domain triggers the deletion of:
  - All mailboxes for the domain
  - All aliases for the domain
  - All vacation entries for the domain
  - All domain aliases pointing to this domain
  - All DKIM keys for the domain
  - All log entries for the domain
- Mandatory confirmation (double validation on UI and API)
- Maildir files are NOT automatically deleted (security)

### Activation/Deactivation (BR-DOM-04)
- A deactivated domain:
  - No longer receives emails (Postfix lookup filters on `active = true`)
  - Users cannot log in
  - Aliases no longer work
  - Remains visible in the admin interface

## Use Cases

### UC-DOM-01: List domains
- **Actor**: Superadmin, Domain Admin
- **Input**: Optional filters (text search, active/inactive), pagination
- **Output**: Paginated list with statistics (number of aliases, mailboxes, quota usage)
- **Rule**: A domain admin only sees their assigned domains

### UC-DOM-02: Create a domain
- **Actor**: Superadmin
- **Input**: Form with all entity fields
- **Validation**: BR-DOM-01
- **Output**: Domain created, log entry

### UC-DOM-03: Modify a domain
- **Actor**: Superadmin, Domain Admin (limited fields)
- **Input**: Modification form
- **Validation**: BR-DOM-02
- **Output**: Updated domain, log entry

### UC-DOM-04: Delete a domain
- **Actor**: Superadmin only
- **Input**: Explicit confirmation (domain name input)
- **Validation**: BR-DOM-03
- **Output**: Domain and all related entities deleted, log entry

### UC-DOM-05: Activate/Deactivate a domain
- **Actor**: Superadmin, Domain Admin
- **Input**: Active/inactive toggle
- **Validation**: BR-DOM-04
- **Output**: Updated status, log entry

## API Endpoints

| Method   | Route                             | Description                          |
|----------|-----------------------------------|--------------------------------------|
| `GET`    | `/api/v1/domains`                 | List domains (paginated, filterable) |
| `GET`    | `/api/v1/domains/{domain}`        | Domain details                       |
| `POST`   | `/api/v1/domains`                 | Create a domain                      |
| `PUT`    | `/api/v1/domains/{domain}`        | Modify a domain                      |
| `DELETE` | `/api/v1/domains/{domain}`        | Delete a domain                      |
| `PATCH`  | `/api/v1/domains/{domain}/active` | Activate/deactivate                  |

## Web Routes

| Route                           | View               | Description             |
|---------------------------------|--------------------|-------------------------|
| `GET /domains`                  | `domain-list.html` | Domain list             |
| `GET /domains/new`              | `domain-form.html` | Creation form           |
| `GET /domains/{domain}/edit`    | `domain-form.html` | Edit form               |
| `POST /domains`                 | —                  | Creation processing     |
| `POST /domains/{domain}`        | —                  | Modification processing |
| `POST /domains/{domain}/delete` | —                  | Deletion processing     |
| `POST /domains/{domain}/toggle` | —                  | HTMX active toggle      |

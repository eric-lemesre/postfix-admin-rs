> **Language:** English | [Francais](../fr/features/02-mailboxes/mailbox-management.md)

---
# SPEC-02.1 — Mailbox Management

## Implementation Status

| Component                                                  | Crate                | Status  | Milestone |
|------------------------------------------------------------|----------------------|---------|-----------|
| Model (`Mailbox`)                                          | `postfix-admin-core` | Done    | M1        |
| DTOs (`CreateMailbox`, `UpdateMailbox`, `MailboxResponse`) | `postfix-admin-core` | Done    | M1        |
| Repository trait (`MailboxRepository`)                     | `postfix-admin-core` | Done    | M1        |
| Validation (maildir, domain limits)                        | `postfix-admin-core` | Done    | M1        |
| PostgreSQL repository                                      | `postfix-admin-db`   | Done    | M2        |
| MySQL repository                                           | `postfix-admin-db`   | Done    | M2        |
| REST API endpoints                                         | `postfix-admin-api`  | Pending | M6        |
| Web UI pages                                               | `postfix-admin-web`  | Pending | M5        |
| CLI commands                                               | `postfix-admin-cli`  | Pending | M8        |

## Summary

CRUD management of virtual mailboxes. Each mailbox represents a user account
capable of receiving, storing and sending emails via Postfix/Dovecot.

## Entity: `Mailbox`

| Field             | Type           | Constraint                | Description                                       |
|-------------------|----------------|---------------------------|---------------------------------------------------|
| `username`        | `VARCHAR(255)` | PK                        | Full email address (e.g., `user@example.com`)     |
| `password`        | `VARCHAR(255)` | NOT NULL                  | Password hash (multi-scheme)                      |
| `name`            | `VARCHAR(255)` | NOT NULL, default `''`    | User's displayed name                             |
| `maildir`         | `VARCHAR(255)` | NOT NULL                  | Relative maildir path (e.g., `example.com/user/`) |
| `quota`           | `BIGINT`       | NOT NULL, default `0`     | Quota in bytes (0 = unlimited)                    |
| `local_part`      | `VARCHAR(255)` | NOT NULL                  | Local part of the address (e.g., `user`)          |
| `domain`          | `VARCHAR(255)` | FK → `domain.domain`      | Mailbox domain                                    |
| `password_expiry` | `TIMESTAMPTZ`  | NULLABLE                  | Password expiry date                              |
| `totp_secret`     | `VARCHAR(255)` | NULLABLE                  | TOTP secret for 2FA (encrypted)                   |
| `active`          | `BOOLEAN`      | NOT NULL, default `true`  | Active/inactive mailbox                           |
| `created_at`      | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Creation date                                     |
| `updated_at`      | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Last modification                                 |

### Index

- `idx_mailbox_username_active` : `(username, active)` — Dovecot lookup
- `idx_mailbox_domain` : `(domain)` — Domain joins and filters

## Business Rules

### Creation (BR-MBX-01)
- Email address must be valid (RFC 5321) and the domain must exist
- Number of mailboxes for the domain must not exceed `domain.mailboxes` (if > 0)
- Mailbox quota must not exceed `domain.maxquota` (if > 0)
- Total quota of domain mailboxes must not exceed `domain.quota` (if > 0)
- An alias `username → username` is automatically created in the `alias` table
- Maildir format is calculated automatically: `{domain}/{local_part}/`
- Password is hashed according to the configured scheme (default: argon2id)
- Verification that no existing alias already points to this address as source

### Modification (BR-MBX-02)
- `username` cannot be modified (immutable PK)
- Password modification re-hashes with current scheme
- Quota modification checks domain limits
- Name change updates `updated_at`

### Deletion (BR-MBX-03)
- Cascade deletion of:
  - Automatic alias (`username → username`)
  - User's vacation entries
  - User's fetchmail entries
  - Application passwords
- Maildir files are NOT automatically deleted
- Disabling recommended before final deletion

### Password (BR-MBX-04)
- Supported read schemes: `{ARGON2ID}`, `{BLF-CRYPT}`, `{SHA512-CRYPT}`,
  `{SHA256-CRYPT}`, `{MD5-CRYPT}`, `{CRYPT}`, `$2y$` prefix (bcrypt),
  `$6$` prefix (sha512-crypt), `$5$` prefix (sha256-crypt)
- New passwords are always hashed with argon2id (configurable)
- On successful authentication with an old scheme, the hash is
  automatically updated to current scheme (transparent rehash)
- Configurable minimum length (default: 8 characters)

### Password Expiry (BR-MBX-05)
- If `domain.password_expiry > 0`, expiry date is calculated:
  `password_expiry = now() + domain.password_expiry days`
- Expired password prevents IMAP/POP3/SMTP login
- User notified X days before expiration (configurable)

## Use Cases

### UC-MBX-01: List mailboxes
- **Actor**: Superadmin, Domain Admin
- **Input**: Selected domain, filters (search, active/inactive), pagination
- **Output**: List with username, name, used/total quota, status, last login

### UC-MBX-02: Create a mailbox
- **Actor**: Superadmin, Domain Admin
- **Input**: local_part, domain (select), name, password, quota, active
- **Validation**: BR-MBX-01
- **Output**: Created mailbox + automatic alias, log entry

### UC-MBX-03: Modify a mailbox
- **Actor**: Superadmin, Domain Admin, User (limited fields)
- **Input**: Modification form
- **Rule**: User can only modify their name and password

### UC-MBX-04: Delete a mailbox
- **Actor**: Superadmin, Domain Admin
- **Validation**: BR-MBX-03, confirmation required

### UC-MBX-05: Change user password
- **Actor**: Authenticated user
- **Input**: Old password, new password (x2)
- **Validation**: Old password verification, complexity rules

## API Endpoints

| Method   | Route                                   | Description           |
|----------|-----------------------------------------|-----------------------|
| `GET`    | `/api/v1/domains/{domain}/mailboxes`    | List domain mailboxes |
| `GET`    | `/api/v1/mailboxes/{username}`          | Mailbox details       |
| `POST`   | `/api/v1/domains/{domain}/mailboxes`    | Create a mailbox      |
| `PUT`    | `/api/v1/mailboxes/{username}`          | Modify a mailbox      |
| `DELETE` | `/api/v1/mailboxes/{username}`          | Delete a mailbox      |
| `PATCH`  | `/api/v1/mailboxes/{username}/active`   | Activate/deactivate   |
| `POST`   | `/api/v1/mailboxes/{username}/password` | Change password       |

---

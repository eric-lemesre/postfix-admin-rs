> **Language:** English | [Francais](../fr/features/04-authentication/app-passwords.md)

---
# SPEC-04.4 — App Passwords (App Passwords)

## Implementation Status

| Component                                         | Crate                | Status  | Milestone |
|---------------------------------------------------|----------------------|---------|-----------|
| Model (`MailboxAppPassword`)                      | `postfix-admin-core` | Done    | M1        |
| DTOs (`CreateAppPassword`, `AppPasswordResponse`) | `postfix-admin-core` | Done    | M1        |
| Repository trait (`AppPasswordRepository`)        | `postfix-admin-core` | Done    | M1        |
| PostgreSQL repository                             | `postfix-admin-db`   | Pending | M2        |
| MySQL repository                                  | `postfix-admin-db`   | Pending | M2        |
| Password generation and hashing                   | `postfix-admin-auth` | Pending | M4        |
| Web UI management page                            | `postfix-admin-web`  | Pending | M5        |

## Summary

Application passwords allow users to generate dedicated passwords for mail clients (Thunderbird, Outlook, smartphones) without exposing the main password. Particularly useful when TOTP 2FA is enabled, as mail clients do not support interactive 2FA.

## Entity: `MailboxAppPassword`

| Field           | Type           | Constraint                | Description                                 |
|-----------------|----------------|---------------------------|---------------------------------------------|
| `id`            | `SERIAL`       | PK                        | Auto-incremented identifier                 |
| `username`      | `VARCHAR(255)` | FK → `mailbox.username`   | Owner                                       |
| `description`   | `VARCHAR(255)` | NOT NULL                  | Description (e.g., "iPhone", "Thunderbird") |
| `password_hash` | `VARCHAR(255)` | NOT NULL                  | Hash of the application password            |
| `last_used`     | `TIMESTAMPTZ`  | NULLABLE                  | Last usage                                  |
| `created_at`    | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Creation date                               |

## Business Rules

### BR-APP-01: Creation
- The user must be authenticated (including 2FA if enabled)
- The password is generated server-side (24 alphanumeric characters)
- The password is displayed only once after creation
- Stored hashed (argon2id) — no recovery possible
- Configurable maximum number of passwords per mailbox (default: 10)
- A description is mandatory

### BR-APP-02: Usage
- Application passwords are accepted by Dovecot for IMAP/POP3/SMTP auth
- They bypass 2FA (that's their purpose)
- Each usage updates `last_used`
- An application password does not give access to the web interface

### BR-APP-03: Revocation
- The user can individually delete an application password
- Deletion is immediate (no grace period)
- Deleting a mailbox deletes all associated application passwords

## Dovecot Integration

The Dovecot authentication query must check in order:
1. The main password (table `mailbox`)
2. Application passwords (table `mailbox_app_password`)

```sql
-- App password verification (Dovecot passdb)
SELECT password_hash FROM mailbox_app_password
WHERE username = '%u'
```

## Use Cases

### UC-APP-01: Create an application password
- **Actor**: Authenticated user
- **Input**: Client description
- **Output**: Generated password displayed once

### UC-APP-02: List application passwords
- **Actor**: Authenticated user
- **Output**: List (description, creation date, last usage) — not the password

### UC-APP-03: Revoke an application password
- **Actor**: Authenticated user
- **Input**: Password ID
- **Output**: Deletion confirmation

---

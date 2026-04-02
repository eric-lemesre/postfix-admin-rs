> **Language:** English | [Francais](../fr/features/07-fetchmail/fetchmail-integration.md)

---
# SPEC-07.1 — Fetchmail Integration

## Summary

Allows users to retrieve mail from remote servers (POP3/IMAP) and deliver it to their local mailbox. Configuration is stored in the database and read by a fetchmail daemon.

## Entity: `Fetchmail`

| Field | Type | Constraint | Description |
|-------|------|-----------|-------------|
| `id` | `SERIAL` | PK | Auto-incremented identifier |
| `domain` | `VARCHAR(255)` | FK → `domain.domain` | Local domain |
| `mailbox` | `VARCHAR(255)` | FK → `mailbox.username` | Destination mailbox |
| `src_server` | `VARCHAR(255)` | NOT NULL | Remote server (hostname/IP) |
| `src_auth` | `VARCHAR(50)` | default `'password'` | Auth method (password, kerberos, ntlm, etc.) |
| `src_user` | `VARCHAR(255)` | NOT NULL | Remote username |
| `src_password` | `VARCHAR(255)` | NOT NULL | Remote password (encrypted) |
| `src_folder` | `VARCHAR(255)` | default `''` | Source folder (IMAP) |
| `poll_time` | `INTEGER` | NOT NULL, default `10` | Polling interval in minutes |
| `fetchall` | `BOOLEAN` | NOT NULL, default `false` | Retrieve all messages (not just new ones) |
| `keep` | `BOOLEAN` | NOT NULL, default `false` | Keep messages on the remote server |
| `protocol` | `VARCHAR(10)` | NOT NULL, default `'IMAP'` | Protocol (POP3, IMAP) |
| `usessl` | `BOOLEAN` | NOT NULL, default `true` | Use SSL/TLS |
| `sslcertck` | `BOOLEAN` | NOT NULL, default `true` | Verify SSL certificate |
| `extra_options` | `TEXT` | NULLABLE | Additional fetchmail options |
| `mda` | `VARCHAR(255)` | default `''` | Custom MDA |
| `returned_text` | `TEXT` | NULLABLE | Last fetchmail output |
| `active` | `BOOLEAN` | NOT NULL, default `true` | Active/inactive |
| `date` | `TIMESTAMPTZ` | default `now()` | Last polling |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Creation date |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Last update |

## Business Rules

### BR-FM-01: Creation
- The destination mailbox must exist and be active
- The remote server must be a valid hostname or IP address
- Remote password is encrypted in the database (AES-256-GCM, same key as TOTP secrets)
- Minimum `poll_time`: 5 minutes
- Optional verification of connectivity to the remote server

### BR-FM-02: Security
- Remote passwords are encrypted at rest
- `usessl` is enabled by default (unencrypted connection possible but discouraged)
- `sslcertck` is enabled by default
- Passwords are never displayed in plain text in the interface (only `****`)
- Only the owner of the mailbox can view/manage their fetchmail configurations

### BR-FM-03: Operation
- A periodic daemon reads active configurations and runs fetchmail
- `returned_text` stores the last output for diagnostics
- `date` is updated on each execution

## Use Cases

### UC-FM-01: Configure Remote Retrieval
- **Actor**: User, Domain Admin
- **Input**: Server, protocol, credentials, options
- **Output**: Configuration created, optional first test

### UC-FM-02: Test Connection
- **Actor**: User, Admin
- **Input**: ID of the configuration
- **Output**: Result of the connection test

### UC-FM-03: View Retrieval Logs
- **Actor**: User, Admin
- **Output**: Last fetchmail output (`returned_text`)

## API Endpoints

| Method | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/mailboxes/{username}/fetchmail` | List fetchmail configs |
| `POST` | `/api/v1/mailboxes/{username}/fetchmail` | Create a config |
| `PUT` | `/api/v1/fetchmail/{id}` | Update a config |
| `DELETE` | `/api/v1/fetchmail/{id}` | Delete a config |
| `POST` | `/api/v1/fetchmail/{id}/test` | Test connection |

---

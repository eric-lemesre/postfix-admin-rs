> **Language:** English | [Francais](../fr/features/06-vacation/vacation-autoresponder.md)

---
# SPEC-06.1 — Auto-responder (Vacation)

## Implementation Status

| Component                                   | Crate                  | Status  | Milestone |
|---------------------------------------------|------------------------|---------|-----------|
| Models (`Vacation`, `VacationNotification`) | `postfix-admin-core`   | Done    | M1        |
| DTOs (`UpdateVacation`, `VacationResponse`) | `postfix-admin-core`   | Done    | M1        |
| Repository trait (`VacationRepository`)     | `postfix-admin-core`   | Done    | M1        |
| PostgreSQL repository                       | `postfix-admin-db`     | Pending | M2        |
| MySQL repository                            | `postfix-admin-db`     | Pending | M2        |
| Vacation CRUD logic                         | `postfix-admin-server` | Pending | M10       |
| REST API endpoints                          | `postfix-admin-api`    | Pending | M6        |
| Web UI pages                                | `postfix-admin-web`    | Pending | M5        |

## Summary

The auto-responder sends a predefined response to emails received during the user's absence. Integration with Postfix `vacation` transport and a deduplication system to avoid responding multiple times to the same sender.

## Entity: `Vacation`

| Field           | Type           | Constraint                | Description                                |
|-----------------|----------------|---------------------------|--------------------------------------------|
| `email`         | `VARCHAR(255)` | PK                        | Email address of the mailbox               |
| `subject`       | `VARCHAR(255)` | NOT NULL                  | Subject of the auto-response               |
| `body`          | `TEXT`         | NOT NULL, default `''`    | Message body                               |
| `domain`        | `VARCHAR(255)` | FK → `domain.domain`      | Domain                                     |
| `active`        | `BOOLEAN`      | NOT NULL, default `true`  | Active auto-responder                      |
| `active_from`   | `TIMESTAMPTZ`  | NULLABLE                  | Start date (optional)                      |
| `active_until`  | `TIMESTAMPTZ`  | NULLABLE                  | End date (optional)                        |
| `interval_time` | `INTEGER`      | NOT NULL, default `0`     | Interval in seconds before re-notification |
| `created_at`    | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Creation date                              |
| `updated_at`    | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Last update                                |

## Entity: `VacationNotification`

| Field         | Type           | Constraint                                              | Description                    |
|---------------|----------------|---------------------------------------------------------|--------------------------------|
| `on_vacation` | `VARCHAR(255)` | PK (composite), FK → `vacation.email` ON DELETE CASCADE | User on vacation               |
| `notified`    | `VARCHAR(255)` | PK (composite)                                          | Address that has been notified |
| `notified_at` | `TIMESTAMPTZ`  | NOT NULL, default `now()`                               | Notification timestamp         |

## Postfix Transport Mechanism

### Architecture
```
Incoming mail
    │
    ▼
Postfix (transport: vacation)
    │
    ▼
vacation.pl / vacation binary
    │
    ├─▶ Checks active vacation + dates
    ├─▶ Checks deduplication (vacation_notification)
    ├─▶ Sends the response
    └─▶ Records the notification
```

### Vacation Alias

When vacation is activated, a special alias is added:
- `user@example.com` → `user@example.com, user#example.com@autoreply.example.com`

The domain `autoreply.example.com` is a vacation transport domain.

## Business Rules

### BR-VAC-01: Activation
- The user must have an active mailbox
- Subject is mandatory
- Body can be empty (but not recommended)
- Activation creates/modifies the alias to include the vacation destination
- `active_from` and `active_until` allow for advance scheduling

### BR-VAC-02: Deactivation
- Deactivation removes the vacation destination from the alias
- Notification entries are retained (periodic cleanup)
- The vacation entry remains in the database (for quick reactivation)

### BR-VAC-03: Deduplication
- A sender receives the auto-response only once per interval
- Configurable `interval_time` (default: 0 = once)
- If > 0: re-notification after N seconds since last notification

### BR-VAC-04: Exclusions
- No response to addresses like:
  - `MAILER-DAEMON@*`
  - `noreply@*`, `no-reply@*`
  - Addresses listed in headers `Precedence: bulk/list/junk`
  - Addresses containing the user's address in `X-Loop`

### BR-VAC-05: Scheduled Periods
- If `active_from` is defined, vacation activates only on that date
- If `active_until` is defined, vacation deactivates automatically
- A cron job periodically checks vacations to activate/deactivate

## Use Cases

### UC-VAC-01: Configure the auto-responder
- **Actor**: User, Domain Admin
- **Input**: Subject, body, optional dates, interval
- **Output**: Vacation configured, alias modified

### UC-VAC-02: Activate/Deactivate the auto-responder
- **Actor**: User, Domain Admin
- **Input**: Toggle activation
- **Output**: Alias modified accordingly

## Web Routes

| Route                                             | Method   | Description          |
|---------------------------------------------------|----------|----------------------|
| `/user/vacation`                                  | GET      | Vacation form (user) |
| `/user/vacation`                                  | POST     | Save vacation        |
| `/domains/{domain}/mailboxes/{username}/vacation` | GET/POST | Admin management     |

## API Endpoints

| Method   | Route                                   | Description        |
|----------|-----------------------------------------|--------------------|
| `GET`    | `/api/v1/mailboxes/{username}/vacation` | View vacation      |
| `PUT`    | `/api/v1/mailboxes/{username}/vacation` | Configure vacation |
| `DELETE` | `/api/v1/mailboxes/{username}/vacation` | Deactivate         |

---

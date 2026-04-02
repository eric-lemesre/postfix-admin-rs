> **Language:** English | [Francais](../fr/database/SCHEMA.md)

---
# Database Schema — postfix-admin-rs

## Entity-Relation Diagram

```
┌──────────────┐     ┌───────────────────┐     ┌──────────────┐
│    admin      │     │   domain_admins    │     │    domain     │
│──────────────│     │───────────────────│     │──────────────│
│ username (PK) │◄───│ username (FK)      │───►│ domain (PK)   │
│ password      │     │ domain (FK)        │     │ description   │
│ superadmin    │     │ created_at         │     │ aliases       │
│ totp_secret   │     │                   │     │ mailboxes     │
│ totp_enabled  │     └───────────────────┘     │ maxquota      │
│ active        │                                │ quota         │
│ created_at    │     ┌───────────────────┐     │ transport     │
│ updated_at    │     │   alias_domain     │     │ backupmx      │
└──────────────┘     │───────────────────│     │ active        │
                      │ alias_domain (PK)  │     │ created_at    │
                      │ target_domain (FK) │───►│ updated_at    │
                      │ active             │     └───────┬──────┘
                      │ created_at         │             │
                      │ updated_at         │     ┌──────────────┐
                      └───────────────────┘     │   dkim_key    │
         ┌──────────────────────────────────────────────┤
         │                    │                          │
         ▼                    ▼                          ▼
┌──────────────┐     ┌──────────────┐          ┌──────────────┐
│   mailbox     │     │    alias      │          │   dkim_key    │
│──────────────│     │──────────────│          │──────────────│
│ username (PK) │     │ address (PK)  │          │ id (PK)       │
│ password      │     │ goto          │          │ domain_name   │
│ name          │     │ domain (FK)   │          │ selector      │
│ maildir       │     │ active        │          │ private_key   │
│ quota         │     │ created_at    │          │ public_key    │
│ local_part    │     │ updated_at    │          │ created_at    │
│ domain (FK)   │     └──────────────┘          │ updated_at    │
│ active        │                                └──────┬───────┘
│ created_at    │                                       │
│ updated_at    │     ┌──────────────┐          ┌──────▼───────┐
└───────┬───────┘     │   vacation    │          │ dkim_signing  │
        │             │──────────────│          │──────────────│
        │             │ email (PK)    │          │ id (PK)       │
        ├────────────►│ subject       │          │ author        │
        │             │ body          │          │ dkim_id (FK)  │
        │             │ domain (FK)   │          │ created_at    │
        │             │ active        │          │ updated_at    │
        │             │ active_from   │          └──────────────┘
        │             │ active_until  │
        │             │ interval_time │
        │             │ created_at    │
        │             │ updated_at    │
        │             └──────┬───────┘
        │                    │
        │             ┌──────▼────────────────┐
        │             │ vacation_notification   │
        │             │────────────────────────│
        │             │ on_vacation (PK, FK)    │
        │             │ notified (PK)           │
        │             │ notified_at             │
        │             └────────────────────────┘
        │
        ├─────────────────────────────────────┐
        │                                     │
        ▼                                     ▼
┌──────────────────┐              ┌──────────────────────┐
│    fetchmail      │              │ mailbox_app_password   │
│──────────────────│              │──────────────────────│
│ id (PK)           │              │ id (PK)               │
│ domain (FK)       │              │ username (FK)          │
│ mailbox (FK)      │              │ description            │
│ src_server        │              │ password_hash          │
│ src_auth          │              │ last_used              │
│ src_user          │              │ created_at             │
│ src_password      │              └──────────────────────┘
│ src_folder        │     ┌──────────────────────┐
│ poll_time         │     │ totp_exception_address │
│ fetchall          │     │──────────────────────│
│ keep              │     │ id (PK)               │
│ protocol          │     │ ip                     │
│ usessl            │     │ username               │
│ active            │     │ description            │
│ date              │     └──────────────────────┘
│ created_at        │     ┌──────────────────┐
│ updated_at        │     │      log          │
└──────────────────┘     │──────────────────│
                          │ id (PK)           │
                          │ timestamp         │
                          │ username          │
                          │ domain            │
                          │ action            │
                          │ data              │
                          │ ip_address        │
                          │ user_agent        │
                          └──────────────────┘
```

## Detailed Tables

See the specifications of each module in `docs/features/` for complete definitions of columns, types, constraints and indexes.

## Relations

| Relation | Type | FK |
|----------|------|-----|
| domain → mailbox | 1:N | `mailbox.domain` |
| domain → alias | 1:N | `alias.domain` |
| domain → vacation | 1:N | `vacation.domain` |
| domain → dkim_key | 1:N | `dkim_key.domain_name` |
| domain → alias_domain | 1:N | `alias_domain.target_domain` |
| admin → domain_admins | 1:N | `domain_admins.username` |
| domain → domain_admins | 1:N | `domain_admins.domain` |
| mailbox → fetchmail | 1:N | `fetchmail.mailbox` |
| mailbox → mailbox_app_password | 1:N | `mailbox_app_password.username` |
| mailbox → quota2 | 1:1 | `quota2.username` |
| vacation → vacation_notification | 1:N | `vacation_notification.on_vacation` |
| dkim_key → dkim_signing | 1:N | `dkim_signing.dkim_id` |

## PostfixAdmin PHP Compatibility

The schema is designed to be compatible with an existing PostfixAdmin PHP database.
Migrations add new columns with default values, allowing for a progressive migration without service interruption.

---

> **Language:** [English](../en/database/SCHEMA.md) | Francais

# Schema de base de donnees — postfix-admin-rs

## Diagramme Entite-Relation

```
┌──────────────┐     ┌───────────────────┐     ┌──────────────┐
│    admin      │     │   domain_admins    │     │    domain     │
│──────────────│     │───────────────────│     │──────────────│
│ username (PK) │◄───│ username (FK)      │───►│ domain (PK)   │
│ password      │     │ domain (FK)        │     │ description   │
│ superadmin    │     │ created_at         │     │ aliases       │
│ totp_secret   │     └───────────────────┘     │ mailboxes     │
│ totp_enabled  │                                │ maxquota      │
│ active        │     ┌───────────────────┐     │ quota         │
│ created_at    │     │   alias_domain     │     │ transport     │
│ updated_at    │     │───────────────────│     │ backupmx      │
└──────────────┘     │ alias_domain (PK)  │     │ active        │
                      │ target_domain (FK) │───►│ created_at    │
                      │ active             │     │ updated_at    │
                      │ created_at         │     └───────┬──────┘
                      │ updated_at         │             │
                      └───────────────────┘             │
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
│ src_folder        │
│ poll_time         │     ┌──────────────────────┐
│ fetchall          │     │ totp_exception_address │
│ keep              │     │──────────────────────│
│ protocol          │     │ id (PK)               │
│ usessl            │     │ ip                     │
│ active            │     │ username               │
│ date              │     │ description            │
│ created_at        │     └──────────────────────┘
│ updated_at        │
└──────────────────┘     ┌──────────────────┐
                          │      log          │
┌──────────────────┐     │──────────────────│
│     quota2        │     │ id (PK)           │
│──────────────────│     │ timestamp         │
│ username (PK)     │     │ username          │
│ bytes             │     │ domain            │
│ messages          │     │ action            │
└──────────────────┘     │ data              │
                          │ ip_address        │
                          │ user_agent        │
                          └──────────────────┘
```

## Tables detaillees

Voir les specifications de chaque module dans `docs/features/` pour les
definitions completes des colonnes, types, contraintes et index.

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

## Compatibilite PostfixAdmin PHP

Le schema est concu pour etre compatible avec une base PostfixAdmin PHP existante.
Les migrations ajoutent les nouvelles colonnes avec des valeurs par defaut, ce qui
permet une migration progressive sans interruption de service.

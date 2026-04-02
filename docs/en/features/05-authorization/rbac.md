> **Language:** English | [Francais](../fr/features/05-authorization/rbac.md)

---
# SPEC-05.1 — Role-Based Access Control (RBAC)

## Summary

Three-level role system with hierarchical permission model. Each application action is protected by a role and scope check.

## Roles

### Superadmin
- Full access to all features
- Can create/edit/delete domains
- Can create/edit/delete administrators
- Can assign administrators to domains
- Can manage global configuration
- Can view and manage all domains without restriction

### Domain admin
- Limited access to assigned domains (table `domain_admins`)
- Can manage mailboxes, aliases and vacation for their domains
- Can view statistics and logs of their domains
- Cannot create new domains
- Cannot manage other administrators
- Cannot access global configuration

### User (mailbox user)
- Access only to own data
- Can change password
- Can manage vacation/out-of-office
- Can manage application passwords
- Can enable/disable TOTP 2FA
- No access to admin functions

## Permission matrix

| Resource | Action | Superadmin | Domain Admin | User |
|-----------|--------|:----------:|:-------------:|:-----------:|
| **Domains** | List | All | Their domains | — |
| | Create | Yes | — | — |
| | Edit | Yes | Their domains (limited) | — |
| | Delete | Yes | — | — |
| **Domain aliases** | CRUD | Yes | — | — |
| **Admins** | List | Yes | — | — |
| | Create | Yes | — | — |
| | Edit | Yes | Themselves (password) | — |
| | Delete | Yes | — | — |
| **Mailboxes** | List | All | Their domains | — |
| | Create | Yes | Their domains | — |
| | Edit | Yes | Their domains | Themselves (limited) |
| | Delete | Yes | Their domains | — |
| **Aliases** | List | All | Their domains | — |
| | Create | Yes | Their domains | — |
| | Edit | Yes | Their domains | — |
| | Delete | Yes | Their domains | — |
| **Vacation** | Manage | Yes | Their domains | Themselves |
| **Fetchmail** | Manage | Yes | Their domains | Themselves |
| **DKIM** | Manage | Yes | Their domains | — |
| **Logs** | View | All | Their domains | — |
| **Configuration** | Edit | Yes | — | — |
| **App passwords** | Manage | — | — | Themselves |
| **TOTP** | Manage | Themselves | Themselves | Themselves |
| **TOTP exceptions** | Manage | Yes | — | — |
| **Broadcast** | Send | Yes | — | — |

## Implementation

### Authorization middleware (axum)

Three distinct axum extractors:

```
RequireSuperAdmin    → Checks superadmin role
RequireDomainAdmin   → Checks admin role + access to requested domain
RequireUser          → Verifies user identity
```

### Scope verification

For domain admins, each request checks:

```
1. The admin is authenticated and active
2. The target domain exists in domain_admins for this admin
3. The resource (mailbox, alias...) belongs to the authorized domain
```

### Domain admin scope rules

Domain fields modifiable by a domain admin:
- `description` — yes
- `aliases`, `mailboxes`, `maxquota`, `quota` — no (set by superadmin)
- `transport`, `backupmx` — no
- `active` — depends on configuration (`domain_admin_can_disable`)

## Business rules

### BR-RBAC-01: Domain assignment
- An admin can be assigned to 0..N domains
- A domain can have 0..N admins
- Only a superadmin can modify assignments

### BR-RBAC-02: Superadmin self-protection
- The last superadmin cannot be deleted
- A superadmin cannot remove their own superadmin status (another superadmin must do it)

### BR-RBAC-03: Role accumulation
- A superadmin is implicitly admin of all domains
- No need to create entries in `domain_admins` for a superadmin
- An admin can be both an admin and have a mailbox (double session possible)

---

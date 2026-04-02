> **Language:** English | [Francais](../fr/features/04-authentication/user-authentication.md)

---
# SPEC-04.2 — User Authentication

## Summary

Self-service interface for mailbox users. Allows management of password,
auto-responder, and profile. Authentication uses the same credentials as IMAP/SMTP
login (table `mailbox`).

## User Authentication Flow

The user logs in with their email address and mail password.
Verification is done against `mailbox.password`.

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ User Login   │────▶│ Verification │────▶│  User Dashboard   │
│ (email/pass) │     │ vs mailbox   │     │                │
└─────────────┘     └──────────────┘     └──────────────┘
```

## Differences with Admin Authentication

| Aspect | Admin | User |
|--------|-------|-------------|
| Table | `admin` | `mailbox` |
| Identifier | admin username | email address |
| TOTP 2FA | Supported | Supported (optional) |
| Session | Same mechanism | Same mechanism |
| Scope | Multi-domain | Their own mailbox only |

## Functions Accessible to the User

| Function | Description |
|----------|-------------|
| Password Change | Old + new password |
| Vacation Management | Auto-responder activation/editing |
| Profile | Modify displayed name |
| TOTP 2FA | Enable/disable 2FA |
| App Passwords | Manage app passwords |

## Business Rules

### BR-UAUTH-01 : Login
- Identical to BR-AUTH-01 but verifies against `mailbox.password`
- Also checks `mailbox.active = true` and `domain.active = true`
- If `password_expiry` is set and exceeded → redirect to password change

### BR-UAUTH-02 : Password Change
- Old password must be verified
- Configurable complexity rules:
  - Minimum length (default: 8)
  - At least one uppercase, lowercase letter, digit (configurable)
- New password cannot be identical to the old one
- Update `password_expiry` if configured at domain level

### BR-UAUTH-03 : Isolation
- A user can only access their own data
- No visibility on other users in the domain
- No access to admin functions

## Web Routes

| Route | Method | Description |
|-------|---------|-------------|
| `/user/login` | GET | User login form |
| `/user/login` | POST | Process login |
| `/user/logout` | POST | Logout |
| `/user/dashboard` | GET | User dashboard |
| `/user/password` | GET/POST | Change password |
| `/user/vacation` | GET/POST | Manage vacation |
| `/user/profile` | GET/POST | Edit profile |
| `/user/totp` | GET/POST | Configure TOTP |
| `/user/app-passwords` | GET/POST | Manage app passwords |

---

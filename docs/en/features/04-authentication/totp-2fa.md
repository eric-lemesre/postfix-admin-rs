> **Language:** English | [Francais](../fr/features/04-authentication/totp-2fa.md)

---
# SPEC-04.3 — Two-factor authentication TOTP

## Summary

Support for two-factor authentication (2FA) via TOTP (Time-based One-Time Password,
RFC 6238) for admin and user accounts. Compatible with standard authentication
applications (Google Authenticator, Authy, FreeOTP, etc.).

## TOTP Parameters

| Parameter | Value |
|-----------|-------|
| Algorithm | SHA-1 (maximum compatibility with apps) |
| Period | 30 seconds |
| Digits | 6 |
| Tolerance window | 1 period before/after (±30s) |

## Entity: `TotpExceptionAddress`

| Field | Type | Constraint | Description |
|-------|------|-----------|-------------|
| `id` | `SERIAL` | PK | Auto-incremented identifier |
| `ip` | `VARCHAR(46)` | NOT NULL | IP address (v4 or v6) |
| `username` | `VARCHAR(255)` | NULLABLE | Admin/user (NULL = global) |
| `description` | `VARCHAR(255)` | NULLABLE | Exception description |

Unique constraint: `(ip, username)`

## 2FA activation flow

```
1. The user accesses the TOTP page
2. The server generates a random secret (160 bits, base32)
3. A QR code is displayed (URI otpauth://totp/...)
4. The secret is also displayed in plain text (manual entry)
5. The user scans the QR code in their app
6. The user enters a verification code
7. If the code is valid → the secret is saved (encrypted) and 2FA is activated
8. Recovery codes are generated and displayed once
```

## Recovery Codes

- 10 single-use codes generated upon activation
- Format: `XXXX-XXXX` (grouped alphanumeric characters)
- Stored hashed in database
- Each code can only be used once
- Possibility to regenerate all codes (invalidates old ones)

## Business Rules

### BR-TOTP-01 : Activation
- The user must be authenticated (first factor)
- A valid verification code must be provided before activation
- The secret is encrypted in the database with an application key (AES-256-GCM)
- Recovery codes are displayed once only (no retrieval possible)

### BR-TOTP-02 : Verification
- The code is verified after password authentication
- Tolerance window: current code + 1 period before + 1 period after
- In case of failure → possibility to use a recovery code
- The same TOTP code cannot be reused (replay protection)
  → Storage of the last validated TOTP timestamp

### BR-TOTP-03 : IP Exceptions
- Some IP addresses can be exempt from 2FA
- Global exceptions (without username): apply to all
- User-specific exceptions: apply to a specific account
- Use case: monitoring servers, internal networks

### BR-TOTP-04 : Deactivation
- The user must provide a valid TOTP code to deactivate 2FA
- Alternative: a superadmin can force deactivation (reset)
- Deactivation removes the secret and recovery codes

## Use Cases

### UC-TOTP-01 : Enable 2FA
- **Actor** : Admin or User
- **Pre-condition** : Authenticated, 2FA not enabled
- **Output** : QR code, recovery codes

### UC-TOTP-02 : Log in with 2FA
- **Actor** : Admin or User
- **Input** : 6-digit TOTP code or recovery code
- **Output** : Fully authenticated session

### UC-TOTP-03 : Manage IP Exceptions
- **Actor** : Superadmin
- **Input** : IP (v4/v6), optional username, description
- **Output** : Exception created/deleted

### UC-TOTP-04 : Reset a user's 2FA
- **Actor** : Superadmin
- **Input** : Admin/user username
- **Output** : 2FA disabled, secret removed

---

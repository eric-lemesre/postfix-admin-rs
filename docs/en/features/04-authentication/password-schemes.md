> **Language:** English | [Francais](../fr/features/04-authentication/password-schemes.md)

---
# SPEC-04.5 — Password Schemes

## Implementation Status

| Component | Crate | Status | Milestone |
|-----------|-------|--------|-----------|
| Password newtype (`Password`, zeroize) | `postfix-admin-core` | Done | M1 |
| Argon2id hashing | `postfix-admin-auth` | Pending | M4 |
| Bcrypt hashing | `postfix-admin-auth` | Pending | M4 |
| SHA-512/256 crypt | `postfix-admin-auth` | Pending | M4 |
| Legacy scheme detection | `postfix-admin-auth` | Pending | M4 |
| Transparent rehashing | `postfix-admin-auth` | Pending | M4 |

## Summary

To allow for seamless migration from PostfixAdmin PHP and to ensure compatibility with Dovecot, the application supports multiple password hashing schemes for reading, while using argon2id for new hashes.

## Supported Schemes

| Scheme | Prefix/Format | Usage | Support |
|--------|---------------|-------|---------|
| **argon2id** | `{ARGON2ID}$argon2id$...` | Default for new hashes | Read + Write |
| **bcrypt** | `{BLF-CRYPT}$2y$...` or `$2y$...` | Migration from PHP | Read-only |
| **SHA-512 crypt** | `{SHA512-CRYPT}$6$...` or `$6$...` | Legacy Dovecot | Read-only |
| **SHA-256 crypt** | `{SHA256-CRYPT}$5$...` or `$5$...` | Legacy | Read-only |
| **MD5 crypt** | `{MD5-CRYPT}$1$...` or `$1$...` | Legacy (weak) | Read-only |
| **crypt** | `{CRYPT}...` | Very old (DES) | Read-only |
| **PLAIN-MD5** | `{PLAIN-MD5}...` | Legacy PostfixAdmin | Read-only |
| **cleartext** | `{CLEAR}...` or `{CLEARTEXT}...` | Dev only | Configurable |

## Automatic Scheme Detection

The detection algorithm proceeds in order:

```
1. If the hash starts with a Dovecot prefix `{SCHEME}`:
   → Extract scheme and verify with corresponding algorithm

2. If the hash starts with `$argon2id$` → argon2id
3. If the hash starts with `$2y$` or `$2b$` → bcrypt
4. If the hash starts with `$6$` → SHA-512 crypt
5. If the hash starts with `$5$` → SHA-256 crypt
6. If the hash starts with `$1$` → MD5 crypt
7. If the hash is 32 hex characters → MD5 plain
8. Otherwise → Failure (unrecognized scheme)
```

## Transparent Rehashing

Upon successful authentication with a non-current scheme:

```
1. Verify password with old scheme → success
2. Hash cleartext password with current scheme (argon2id)
3. Update hash in database
4. Log migration event (without password)

This mechanism allows for gradual migration without user action.
```

## Argon2id Parameters

| Parameter | Default Value | Description |
|-----------|--------------|-------------|
| `memory_cost` | 19456 (19 MB) | Memory used |
| `time_cost` | 2 | Iterations |
| `parallelism` | 1 | Threads |
| `output_len` | 32 | Hash length |

These parameters are configurable and follow OWASP 2024 recommendations.

## Bcrypt Parameters (Read-only)

| Parameter | Value |
|-----------|-------|
| `cost` | Detected from existing hash |

## Password Validation

Configurable complexity rules:

```toml
[password_policy]
min_length = 8
require_uppercase = true
require_lowercase = true
require_digit = true
require_special = false
max_length = 256
```

## Security Considerations

- Hash comparisons are constant-time (timing-safe)
- Cleartext passwords are never logged
- The `{CLEAR}` scheme is disabled by default and requires explicit activation
- MD5 and DES crypt schemes are considered weak: a warning is logged on each use and rehashing is strongly recommended

---

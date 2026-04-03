> **Language:** English | [Francais](SECURITY.fr.md)

---
# Security policy — postfix-admin-rs

## Supported versions

| Version | Security support |
|---------|------------------|
| 1.x.x   | Yes (active)     |
| < 1.0   | No               |

## Reporting a vulnerability

**DO NOT report security vulnerabilities via public GitHub issues.**

To report a vulnerability:

1. Send an email to **security@example.com** (to be updated with the actual address)
2. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if possible)

### Response time

- Acknowledgment: 48 hours
- Initial assessment: 7 days
- Fix: depending on severity (critical < 7 days, high < 30 days)

### Process

1. Receive and acknowledge the report
2. Assess and confirm the vulnerability
3. Develop a fix
4. Release a fixed version
5. Responsible disclosure (after the fix)

## Security best practices

### For administrators

- Always use HTTPS (TLS) with a reverse proxy
- Keep postfix-admin-rs up to date
- Use strong passwords for admin accounts
- Enable TOTP 2FA for all admin accounts
- Use client certificates (mTLS) for superadmin accounts when possible
- Restrict network access to the administration interface
- Regularly back up the database
- Monitor audit logs

### For developers

- Follow the [security guidelines](docs/en/guidelines/GUIDELINES-Rust.md#10-security)
- Use parameterized queries exclusively
- Never log passwords or secrets
- Run `cargo audit` regularly
- Passwords are hashed with argon2id
- Secrets at rest are encrypted (AES-256-GCM)

## Dependencies

Dependencies are regularly audited using `cargo audit`.
Security updates for dependencies are handled as a priority.

---

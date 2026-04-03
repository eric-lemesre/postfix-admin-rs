> **Language:** English | [Francais](../fr/features/00-overview.md)

---
# postfix-admin-rs — Overview of Features

## Objective

Complete rewrite of [PostfixAdmin](https://github.com/postfixadmin/postfixadmin) in Rust.
Web application for managing virtual domains, mailboxes, aliases and associated services
for a Postfix + Dovecot mail server.

## Functional Scope

| #  | Module                               | Description                                      | Priority | Core | DB | Auth | API | Web | CLI |
|----|--------------------------------------|--------------------------------------------------|----------|:----:|:--:|:----:|:---:|:---:|:---:|
| 01 | [Domains](01-domains/)               | Virtual domain and alias domain management       | P0       | Done | —  |  —   |  —  |  —  |  —  |
| 02 | [Mailboxes](02-mailboxes/)           | Mailboxes, quotas, maildir                       | P0       | Done | —  |  —   |  —  |  —  |  —  |
| 03 | [Aliases](03-aliases/)               | Email aliases, distribution lists                | P0       | Done | —  |  —   |  —  |  —  |  —  |
| 04 | [Authentication](04-authentication/) | Admin/user login, passwords, TOTP, app passwords | P0       | Done | —  |  —   |  —  |  —  |  —  |
| 05 | [Authorization](05-authorization/)   | RBAC: superadmin, domain admin, user             | P0       |  —   | —  |  —   |  —  |  —  |  —  |
| 06 | [Vacation](06-vacation/)             | Automatic replier / out-of-office                | P1       | Done | —  |  —   |  —  |  —  |  —  |
| 07 | [Fetchmail](07-fetchmail/)           | Remote mail retrieval (POP3/IMAP)                | P2       | Done | —  |  —   |  —  |  —  |  —  |
| 08 | [DKIM](08-dkim/)                     | DKIM key and signature management                | P1       | Done | —  |  —   |  —  |  —  |  —  |
| 09 | [Logging](09-logging/)               | Admin action audit log                           | P0       | Done | —  |  —   |  —  |  —  |  —  |
| 10 | [API](10-api/)                       | REST API + gRPC                                  | P1       | Done | —  |  —   |  —  |  —  |  —  |
| 11 | [CLI](11-cli/)                       | Command line interface                           | P1       |  —   | —  |  —   |  —  |  —  |  —  |
| 12 | [Web UI](12-web-ui/)                 | HTMX + Tailwind web interface                    | P0       |  —   | —  |  —   |  —  |  —  |  —  |
| 13 | [Configuration](13-configuration/)   | Multi-source TOML configuration system           | P0       |  —   | —  |  —   |  —  |  —  |  —  |

## Priority Legend

- **P0**: Essential for a functional MVP
- **P1**: Necessary for feature parity with PHP PostfixAdmin
- **P2**: Secondary functionality, implementable after V1

## Compatibility

- Transparent migration from existing PHP PostfixAdmin database
- Support for legacy password formats (md5-crypt, sha512-crypt, bcrypt, dovecot:*)
- Database schema compatible with existing Postfix/Dovecot configurations

## Technical Stack

| Layer         | Technology                         |
|---------------|------------------------------------|
| Language      | Rust (edition 2021)                |
| Web framework | axum                               |
| Database      | sqlx (PostgreSQL, MySQL, SQLite)   |
| Templates     | Askama                             |
| Frontend      | HTMX + Tailwind CSS                |
| gRPC          | tonic + prost                      |
| Auth          | argon2, bcrypt, sha-crypt, totp-rs |
| Config        | config-rs (TOML)                   |
| CLI           | clap                               |
| Logging       | tracing                            |
| Tests         | cargo test, testcontainers-rs      |

## License

GPLv3 — see [LICENSE](../../../LICENSE)

---

> **Language:** [English](../en/features/00-overview.md) | Francais

# postfix-admin-rs — Vue d'ensemble des fonctionnalités

## Objectif

Réécriture complète de [PostfixAdmin](https://github.com/postfixadmin/postfixadmin) en Rust.
Application web d'administration des domaines virtuels, boîtes mail, alias et services
associés pour un serveur de messagerie Postfix + Dovecot.

## Périmètre fonctionnel

| #  | Module                               | Description                                          | Priorité |
|----|--------------------------------------|------------------------------------------------------|----------|
| 01 | [Domains](01-domains/)               | Gestion des domaines virtuels et domaines alias      | P0       |
| 02 | [Mailboxes](02-mailboxes/)           | Boîtes mail, quotas, maildir                         | P0       |
| 03 | [Aliases](03-aliases/)               | Alias email, listes de distribution                  | P0       |
| 04 | [Authentication](04-authentication/) | Login admin/user, mots de passe, TOTP, app passwords | P0       |
| 05 | [Authorization](05-authorization/)   | RBAC : superadmin, admin de domaine, utilisateur     | P0       |
| 06 | [Vacation](06-vacation/)             | Répondeur automatique / out-of-office                | P1       |
| 07 | [Fetchmail](07-fetchmail/)           | Récupération de courrier distant (POP3/IMAP)         | P2       |
| 08 | [DKIM](08-dkim/)                     | Gestion des clés et signatures DKIM                  | P1       |
| 09 | [Logging](09-logging/)               | Journal d'audit des actions admin                    | P0       |
| 10 | [API](10-api/)                       | REST API + gRPC                                      | P1       |
| 11 | [CLI](11-cli/)                       | Interface en ligne de commande                       | P1       |
| 12 | [Web UI](12-web-ui/)                 | Interface web HTMX + Tailwind                        | P0       |
| 13 | [Configuration](13-configuration/)   | Système de configuration TOML multi-sources          | P0       |

## Légende des priorités

- **P0** : Indispensable pour un MVP fonctionnel
- **P1** : Nécessaire pour la parité fonctionnelle avec PostfixAdmin PHP
- **P2** : Fonctionnalité secondaire, implémentable après la V1

## Compatibilité

- Migration transparente depuis une base PostfixAdmin PHP existante
- Support des formats de mots de passe hérités (md5-crypt, sha512-crypt, bcrypt, dovecot:*)
- Schéma de base de données compatible avec les configurations Postfix/Dovecot existantes

## Stack technique

| Couche          | Technologie                        |
|-----------------|------------------------------------|
| Langage         | Rust (edition 2021)                |
| Web framework   | axum                               |
| Base de données | sqlx (PostgreSQL, MySQL, SQLite)   |
| Templates       | Askama                             |
| Frontend        | HTMX + Tailwind CSS                |
| gRPC            | tonic + prost                      |
| Auth            | argon2, bcrypt, sha-crypt, totp-rs |
| Config          | config-rs (TOML)                   |
| CLI             | clap                               |
| Logging         | tracing                            |
| Tests           | cargo test, testcontainers-rs      |

## Licence

GPLv3 — voir [LICENSE](../../../LICENSE)

> **Language:** [English](../en/features/04-authentication/password-schemes.md) | Francais

# SPEC-04.5 — Schémas de mots de passe

## Résumé

Pour permettre une migration transparente depuis PostfixAdmin PHP et pour assurer la
compatibilité avec Dovecot, l'application supporte plusieurs schémas de hashage de
mots de passe en lecture, tout en utilisant argon2id pour les nouveaux hash.

## Schémas supportés

| Schéma            | Préfixe/Format                     | Usage                         | Support            |
|-------------------|------------------------------------|-------------------------------|--------------------|
| **argon2id**      | `{ARGON2ID}$argon2id$...`          | Défaut pour les nouveaux hash | Lecture + Écriture |
| **bcrypt**        | `{BLF-CRYPT}$2y$...` ou `$2y$...`  | Migration depuis PHP          | Lecture seule      |
| **SHA-512 crypt** | `{SHA512-CRYPT}$6$...` ou `$6$...` | Legacy Dovecot                | Lecture seule      |
| **SHA-256 crypt** | `{SHA256-CRYPT}$5$...` ou `$5$...` | Legacy                        | Lecture seule      |
| **MD5 crypt**     | `{MD5-CRYPT}$1$...` ou `$1$...`    | Legacy (faible)               | Lecture seule      |
| **crypt**         | `{CRYPT}...`                       | Très ancien (DES)             | Lecture seule      |
| **PLAIN-MD5**     | `{PLAIN-MD5}...`                   | Legacy PostfixAdmin           | Lecture seule      |
| **cleartext**     | `{CLEAR}...` ou `{CLEARTEXT}...`   | Dev uniquement                | Configurable       |

## Détection automatique du schéma

L'algorithme de détection procède dans l'ordre :

```
1. Si le hash commence par un préfixe Dovecot `{SCHEME}` :
   → Extraire le schéma et vérifier avec l'algorithme correspondant

2. Si le hash commence par `$argon2id$` → argon2id
3. Si le hash commence par `$2y$` ou `$2b$` → bcrypt
4. Si le hash commence par `$6$` → SHA-512 crypt
5. Si le hash commence par `$5$` → SHA-256 crypt
6. Si le hash commence par `$1$` → MD5 crypt
7. Si le hash fait 32 caractères hex → MD5 plain
8. Sinon → Échec (schéma non reconnu)
```

## Rehash transparent

Lors d'une authentification réussie avec un schéma non-courant :

```
1. Vérifier le mot de passe avec l'ancien schéma → succès
2. Hasher le mot de passe en clair avec le schéma courant (argon2id)
3. Mettre à jour le hash en base de données
4. Logger l'événement de migration (sans le mot de passe)
```

Ce mécanisme permet une migration progressive sans action utilisateur.

## Paramètres argon2id

| Paramètre     | Valeur par défaut | Description      |
|---------------|-------------------|------------------|
| `memory_cost` | 19456 (19 Mo)     | Mémoire utilisée |
| `time_cost`   | 2                 | Itérations       |
| `parallelism` | 1                 | Threads          |
| `output_len`  | 32                | Longueur du hash |

Ces paramètres sont configurables et suivent les recommandations OWASP 2024.

## Paramètres bcrypt (lecture seule)

| Paramètre | Valeur                          |
|-----------|---------------------------------|
| `cost`    | Détecté depuis le hash existant |

## Validation du mot de passe

Règles de complexité configurables :

```toml
[password_policy]
min_length = 8
require_uppercase = true
require_lowercase = true
require_digit = true
require_special = false
max_length = 256
```

## Considérations de sécurité

- Les comparaisons de hash sont constantes en temps (timing-safe)
- Les mots de passe en clair ne sont jamais loggés
- Le schéma `{CLEAR}` est désactivé par défaut et nécessite une activation explicite
- Les schémas MD5 et DES crypt sont considérés faibles : un warning est loggé
  lors de chaque utilisation et un rehash est fortement recommandé

> **Language:** [English](../en/features/02-mailboxes/mailbox-management.md) | Francais

# SPEC-02.1 — Gestion des boîtes mail

## Résumé

Gestion CRUD des boîtes mail virtuelles. Chaque boîte représente un compte utilisateur
capable de recevoir, stocker et envoyer du courrier via Postfix/Dovecot.

## Entité : `Mailbox`

| Champ             | Type           | Contrainte                | Description                                      |
|-------------------|----------------|---------------------------|--------------------------------------------------|
| `username`        | `VARCHAR(255)` | PK                        | Adresse email complète (ex: `user@example.com`)  |
| `password`        | `VARCHAR(255)` | NOT NULL                  | Hash du mot de passe (multi-schéma)              |
| `name`            | `VARCHAR(255)` | NOT NULL, default `''`    | Nom affiché de l'utilisateur                     |
| `maildir`         | `VARCHAR(255)` | NOT NULL                  | Chemin maildir relatif (ex: `example.com/user/`) |
| `quota`           | `BIGINT`       | NOT NULL, default `0`     | Quota en octets (0 = illimité)                   |
| `local_part`      | `VARCHAR(255)` | NOT NULL                  | Partie locale de l'adresse (ex: `user`)          |
| `domain`          | `VARCHAR(255)` | FK → `domain.domain`      | Domaine de la boîte                              |
| `password_expiry` | `TIMESTAMPTZ`  | NULLABLE                  | Date d'expiration du mot de passe                |
| `totp_secret`     | `VARCHAR(255)` | NULLABLE                  | Secret TOTP pour le 2FA (chiffré)                |
| `active`          | `BOOLEAN`      | NOT NULL, default `true`  | Boîte active/inactive                            |
| `created_at`      | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Date de création                                 |
| `updated_at`      | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Dernière modification                            |

### Index

- `idx_mailbox_username_active` : `(username, active)` — Lookup Dovecot
- `idx_mailbox_domain` : `(domain)` — Jointures et filtres par domaine

## Règles métier

### Création (BR-MBX-01)
- L'adresse email doit être valide (RFC 5321) et le domaine doit exister
- Le nombre de boîtes du domaine ne doit pas dépasser `domain.mailboxes` (si > 0)
- Le quota de la boîte ne doit pas dépasser `domain.maxquota` (si > 0)
- Le quota cumulé des boîtes du domaine ne doit pas dépasser `domain.quota` (si > 0)
- Un alias `username → username` est automatiquement créé dans la table `alias`
- Le format maildir est calculé automatiquement : `{domain}/{local_part}/`
- Le mot de passe est hashé selon le schéma configuré (par défaut : argon2id)
- Vérification qu'aucun alias existant ne pointe déjà vers cette adresse comme source

### Modification (BR-MBX-02)
- Le `username` ne peut pas être modifié (PK immuable)
- La modification du mot de passe re-hashe avec le schéma courant
- La modification du quota vérifie les limites du domaine
- Le changement de nom met à jour `updated_at`

### Suppression (BR-MBX-03)
- Suppression en cascade de :
  - L'alias automatique (`username → username`)
  - Les entrées vacation de l'utilisateur
  - Les entrées fetchmail de l'utilisateur
  - Les mots de passe applicatifs
- Les fichiers maildir ne sont PAS supprimés automatiquement
- Option de désactivation recommandée avant suppression définitive

### Mot de passe (BR-MBX-04)
- Schémas supportés en lecture : `{ARGON2ID}`, `{BLF-CRYPT}`, `{SHA512-CRYPT}`,
  `{SHA256-CRYPT}`, `{MD5-CRYPT}`, `{CRYPT}`, préfixe `$2y$` (bcrypt),
  préfixe `$6$` (sha512-crypt), préfixe `$5$` (sha256-crypt)
- Les nouveaux mots de passe sont toujours hashés en argon2id (configurable)
- Lors d'une authentification réussie avec un ancien schéma, le hash est
  automatiquement mis à jour vers le schéma courant (rehash transparent)
- Longueur minimale configurable (par défaut : 8 caractères)

### Expiration mot de passe (BR-MBX-05)
- Si `domain.password_expiry > 0`, la date d'expiration est calculée :
  `password_expiry = now() + domain.password_expiry jours`
- Un mot de passe expiré empêche la connexion IMAP/POP3/SMTP
- L'utilisateur est notifié X jours avant expiration (configurable)

## Cas d'utilisation

### UC-MBX-01 : Lister les boîtes mail
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Domaine sélectionné, filtres (recherche, actif/inactif), pagination
- **Sortie** : Liste avec username, nom, quota utilisé/total, statut, dernière connexion

### UC-MBX-02 : Créer une boîte mail
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : local_part, domaine (select), nom, mot de passe, quota, actif
- **Validation** : BR-MBX-01
- **Sortie** : Boîte créée + alias automatique, entrée de log

### UC-MBX-03 : Modifier une boîte mail
- **Acteur** : Superadmin, Admin du domaine, Utilisateur (champs limités)
- **Entrée** : Formulaire de modification
- **Règle** : L'utilisateur ne peut modifier que son nom et son mot de passe

### UC-MBX-04 : Supprimer une boîte mail
- **Acteur** : Superadmin, Admin du domaine
- **Validation** : BR-MBX-03, confirmation requise

### UC-MBX-05 : Changement de mot de passe utilisateur
- **Acteur** : Utilisateur authentifié
- **Entrée** : Ancien mot de passe, nouveau mot de passe (x2)
- **Validation** : Vérification ancien mot de passe, règles de complexité

## Endpoints API

| Méthode  | Route                                   | Description                  |
|----------|-----------------------------------------|------------------------------|
| `GET`    | `/api/v1/domains/{domain}/mailboxes`    | Lister les boîtes du domaine |
| `GET`    | `/api/v1/mailboxes/{username}`          | Détails d'une boîte          |
| `POST`   | `/api/v1/domains/{domain}/mailboxes`    | Créer une boîte              |
| `PUT`    | `/api/v1/mailboxes/{username}`          | Modifier une boîte           |
| `DELETE` | `/api/v1/mailboxes/{username}`          | Supprimer une boîte          |
| `PATCH`  | `/api/v1/mailboxes/{username}/active`   | Activer/désactiver           |
| `POST`   | `/api/v1/mailboxes/{username}/password` | Changer le mot de passe      |

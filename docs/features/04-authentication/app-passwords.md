# SPEC-04.4 — Mots de passe applicatifs (App Passwords)

## Résumé

Les mots de passe applicatifs permettent aux utilisateurs de générer des mots de passe
dédiés pour les clients mail (Thunderbird, Outlook, smartphones) sans exposer le mot de
passe principal. Particulièrement utile lorsque le TOTP 2FA est activé, car les clients
mail ne supportent pas le 2FA interactif.

## Entité : `MailboxAppPassword`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `id` | `SERIAL` | PK | Identifiant auto-incrémenté |
| `username` | `VARCHAR(255)` | FK → `mailbox.username` | Propriétaire |
| `description` | `VARCHAR(255)` | NOT NULL | Description (ex: "iPhone", "Thunderbird") |
| `password_hash` | `VARCHAR(255)` | NOT NULL | Hash du mot de passe applicatif |
| `last_used` | `TIMESTAMPTZ` | NULLABLE | Dernière utilisation |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |

## Règles métier

### BR-APP-01 : Création
- L'utilisateur doit être authentifié (y compris 2FA si activé)
- Le mot de passe est généré côté serveur (24 caractères alphanumériques)
- Le mot de passe est affiché une seule fois après création
- Stocké hashé (argon2id) — pas de récupération possible
- Maximum configurable de mots de passe par boîte (par défaut : 10)
- Une description est obligatoire

### BR-APP-02 : Utilisation
- Les app passwords sont acceptés par Dovecot pour IMAP/POP3/SMTP auth
- Ils contournent le 2FA (c'est leur raison d'être)
- Chaque utilisation met à jour `last_used`
- Un app password ne donne pas accès à l'interface web

### BR-APP-03 : Révocation
- L'utilisateur peut supprimer individuellement un app password
- La suppression est immédiate (pas de délai de grâce)
- La suppression de la boîte mail supprime tous les app passwords associés

## Intégration Dovecot

La requête d'authentification Dovecot doit vérifier dans l'ordre :
1. Le mot de passe principal (table `mailbox`)
2. Les mots de passe applicatifs (table `mailbox_app_password`)

```sql
-- Vérification app password (Dovecot passdb)
SELECT password_hash FROM mailbox_app_password
WHERE username = '%u'
```

## Cas d'utilisation

### UC-APP-01 : Créer un mot de passe applicatif
- **Acteur** : Utilisateur authentifié
- **Entrée** : Description du client
- **Sortie** : Mot de passe généré affiché une seule fois

### UC-APP-02 : Lister ses mots de passe applicatifs
- **Acteur** : Utilisateur authentifié
- **Sortie** : Liste (description, date création, dernière utilisation) — pas le mot de passe

### UC-APP-03 : Révoquer un mot de passe applicatif
- **Acteur** : Utilisateur authentifié
- **Entrée** : ID du mot de passe
- **Sortie** : Confirmation de suppression

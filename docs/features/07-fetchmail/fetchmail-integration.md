# SPEC-07.1 — Intégration Fetchmail

## Résumé

Permet aux utilisateurs de récupérer du courrier depuis des serveurs distants
(POP3/IMAP) et de le délivrer dans leur boîte locale. La configuration est stockée
en base et lue par un démon fetchmail.

## Entité : `Fetchmail`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `id` | `SERIAL` | PK | Identifiant auto-incrémenté |
| `domain` | `VARCHAR(255)` | FK → `domain.domain` | Domaine local |
| `mailbox` | `VARCHAR(255)` | FK → `mailbox.username` | Boîte de destination |
| `src_server` | `VARCHAR(255)` | NOT NULL | Serveur distant (hostname/IP) |
| `src_auth` | `VARCHAR(50)` | default `'password'` | Méthode d'auth (password, kerberos, ntlm, etc.) |
| `src_user` | `VARCHAR(255)` | NOT NULL | Nom d'utilisateur distant |
| `src_password` | `VARCHAR(255)` | NOT NULL | Mot de passe distant (chiffré) |
| `src_folder` | `VARCHAR(255)` | default `''` | Dossier source (IMAP) |
| `poll_time` | `INTEGER` | NOT NULL, default `10` | Intervalle de polling en minutes |
| `fetchall` | `BOOLEAN` | NOT NULL, default `false` | Récupérer tous les messages (pas seulement les nouveaux) |
| `keep` | `BOOLEAN` | NOT NULL, default `false` | Garder les messages sur le serveur distant |
| `protocol` | `VARCHAR(10)` | NOT NULL, default `'IMAP'` | Protocole (POP3, IMAP) |
| `usessl` | `BOOLEAN` | NOT NULL, default `true` | Utiliser SSL/TLS |
| `sslcertck` | `BOOLEAN` | NOT NULL, default `true` | Vérifier le certificat SSL |
| `extra_options` | `TEXT` | NULLABLE | Options fetchmail supplémentaires |
| `mda` | `VARCHAR(255)` | default `''` | MDA personnalisé |
| `returned_text` | `TEXT` | NULLABLE | Dernière sortie fetchmail |
| `active` | `BOOLEAN` | NOT NULL, default `true` | Actif/inactif |
| `date` | `TIMESTAMPTZ` | default `now()` | Dernier polling |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Dernière modification |

## Règles métier

### BR-FM-01 : Création
- La boîte mail de destination doit exister et être active
- Le serveur distant doit être un hostname ou une IP valide
- Le mot de passe distant est chiffré en base (AES-256-GCM, même clé que les secrets TOTP)
- `poll_time` minimum : 5 minutes
- Vérification optionnelle de la connectivité au serveur distant

### BR-FM-02 : Sécurité
- Les mots de passe distants sont chiffrés au repos
- `usessl` est activé par défaut (connexion non chiffrée possible mais déconseillée)
- `sslcertck` est activé par défaut
- Les mots de passe ne sont jamais affichés en clair dans l'interface (seulement `****`)
- Seul le propriétaire de la boîte mail peut voir/gérer ses configurations fetchmail

### BR-FM-03 : Fonctionnement
- Un démon périodique lit les configurations actives et exécute fetchmail
- `returned_text` stocke la dernière sortie pour le diagnostic
- `date` est mis à jour à chaque exécution

## Cas d'utilisation

### UC-FM-01 : Configurer une récupération distante
- **Acteur** : Utilisateur, Admin du domaine
- **Entrée** : Serveur, protocole, credentials, options
- **Sortie** : Configuration créée, premier test optionnel

### UC-FM-02 : Tester la connexion
- **Acteur** : Utilisateur, Admin
- **Entrée** : ID de la configuration
- **Sortie** : Résultat du test de connexion

### UC-FM-03 : Voir les logs de récupération
- **Acteur** : Utilisateur, Admin
- **Sortie** : Dernière sortie fetchmail (`returned_text`)

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/mailboxes/{username}/fetchmail` | Lister les configs fetchmail |
| `POST` | `/api/v1/mailboxes/{username}/fetchmail` | Créer une config |
| `PUT` | `/api/v1/fetchmail/{id}` | Modifier une config |
| `DELETE` | `/api/v1/fetchmail/{id}` | Supprimer une config |
| `POST` | `/api/v1/fetchmail/{id}/test` | Tester la connexion |

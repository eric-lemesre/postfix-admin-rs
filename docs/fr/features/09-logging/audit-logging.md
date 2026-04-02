> **Language:** [English](../en/features/09-logging/audit-logging.md) | Francais

# SPEC-09.1 — Journal d'audit (Logging)

## Résumé

Système de journalisation de toutes les actions d'administration effectuées via
l'interface web, l'API ou le CLI. Permet la traçabilité complète des modifications
apportées à la configuration du serveur mail.

## Entité : `Log`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `id` | `BIGSERIAL` | PK | Identifiant auto-incrémenté |
| `timestamp` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date et heure de l'action |
| `username` | `VARCHAR(255)` | NOT NULL | Auteur de l'action |
| `domain` | `VARCHAR(255)` | NOT NULL | Domaine concerné |
| `action` | `VARCHAR(255)` | NOT NULL | Type d'action |
| `data` | `TEXT` | NOT NULL, default `''` | Détails de l'action |
| `ip_address` | `VARCHAR(46)` | NULLABLE | Adresse IP source |
| `user_agent` | `VARCHAR(512)` | NULLABLE | User-Agent (web) ou identifiant (API/CLI) |

### Index

- `idx_log_timestamp` : `(timestamp)` — Requêtes chronologiques
- `idx_log_domain` : `(domain)` — Filtrage par domaine
- `idx_log_username` : `(username)` — Filtrage par admin

## Actions loguées

| Action | Description | Données typiques |
|--------|-------------|-----------------|
| `create_domain` | Création de domaine | Nom du domaine, paramètres |
| `edit_domain` | Modification de domaine | Champs modifiés |
| `delete_domain` | Suppression de domaine | Nom du domaine |
| `create_mailbox` | Création de boîte mail | Username, quota |
| `edit_mailbox` | Modification de boîte | Champs modifiés (jamais le password) |
| `delete_mailbox` | Suppression de boîte | Username |
| `create_alias` | Création d'alias | Adresse → destinations |
| `edit_alias` | Modification d'alias | Nouvelles destinations |
| `delete_alias` | Suppression d'alias | Adresse |
| `edit_vacation` | Modification vacation | Activation/désactivation |
| `create_admin` | Création admin | Username, domaines assignés |
| `edit_admin` | Modification admin | Champs modifiés |
| `delete_admin` | Suppression admin | Username |
| `login_success` | Connexion réussie | IP, user-agent |
| `login_failure` | Échec de connexion | IP, user-agent, raison |
| `password_change` | Changement de mot de passe | Username (jamais le password) |
| `totp_enable` | Activation 2FA | Username |
| `totp_disable` | Désactivation 2FA | Username |
| `create_dkim` | Création clé DKIM | Domaine, sélecteur |
| `toggle_active` | Activation/désactivation | Entité, ancien état → nouvel état |

## Règles métier

### BR-LOG-01 : Enregistrement
- Chaque action de modification (CREATE, UPDATE, DELETE) est loguée
- Les authentifications (succès et échecs) sont loguées
- Les lectures (GET/LIST) ne sont PAS loguées (performance)
- Les mots de passe ne sont JAMAIS inclus dans les données de log

### BR-LOG-02 : Consultation
- Superadmin : accès à tous les logs
- Admin de domaine : accès aux logs de ses domaines uniquement
- Utilisateur : pas d'accès aux logs
- Filtrable par : période, domaine, action, username
- Paginé (défaut : 50 entrées par page)

### BR-LOG-03 : Rétention
- Durée de rétention configurable (par défaut : 365 jours)
- Nettoyage automatique des anciennes entrées (tâche périodique)
- Option d'export en CSV/JSON avant purge

### BR-LOG-04 : Intégration syslog
- Optionnellement, les logs d'audit sont aussi envoyés vers syslog
- Format structuré compatible avec les outils d'analyse (ELK, Loki, etc.)
- Niveau syslog configurable (par défaut : LOG_INFO)

## Cas d'utilisation

### UC-LOG-01 : Consulter le journal
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Filtres (période, domaine, action, utilisateur), pagination
- **Sortie** : Liste chronologique des actions

### UC-LOG-02 : Exporter les logs
- **Acteur** : Superadmin
- **Entrée** : Filtres, format (CSV, JSON)
- **Sortie** : Fichier téléchargeable

## Routes Web

| Route | Méthode | Description |
|-------|---------|-------------|
| `/admin/logs` | GET | Consultation du journal (paginé, filtrable) |
| `/admin/logs/export` | GET | Export CSV/JSON |

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/logs` | Lister les logs (paginé, filtrable) |
| `GET` | `/api/v1/domains/{domain}/logs` | Logs d'un domaine |

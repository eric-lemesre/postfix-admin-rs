> **Language:** [English](../en/features/01-domains/domain-management.md) | Francais

# SPEC-01.1 — Gestion des domaines virtuels

## Résumé

Gestion CRUD des domaines de messagerie virtuels hébergés par le serveur Postfix.
Chaque domaine définit les limites (nombre d'alias, de boîtes, quotas) et les
paramètres de transport.

## Entité : `Domain`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `domain` | `VARCHAR(255)` | PK | Nom de domaine (ex: `example.com`) |
| `description` | `VARCHAR(255)` | NOT NULL, default `''` | Description libre |
| `aliases` | `INTEGER` | NOT NULL, default `0` | Limite d'alias (0 = illimité) |
| `mailboxes` | `INTEGER` | NOT NULL, default `0` | Limite de boîtes mail (0 = illimité) |
| `maxquota` | `BIGINT` | NOT NULL, default `0` | Quota max par boîte en Mo (0 = illimité) |
| `quota` | `BIGINT` | NOT NULL, default `0` | Quota total du domaine en Mo (0 = illimité) |
| `transport` | `VARCHAR(255)` | NULLABLE | Transport Postfix (ex: `virtual:`, `lmtp:unix:...`) |
| `backupmx` | `BOOLEAN` | NOT NULL, default `false` | Le serveur est MX de backup pour ce domaine |
| `password_expiry` | `INTEGER` | NOT NULL, default `0` | Expiration mots de passe en jours (0 = désactivé) |
| `active` | `BOOLEAN` | NOT NULL, default `true` | Domaine actif/inactif |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Dernière modification |

### Index

- `idx_domain_active` : `(domain, active)` — Utilisé par les lookups Postfix

## Règles métier

### Création (BR-DOM-01)
- Le nom de domaine doit respecter la RFC 1035 (regex : `^([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$`)
- Vérification DNS optionnelle : au moins un enregistrement A, AAAA, MX ou NS
- Un domaine ne peut pas être créé s'il existe déjà en tant que domaine ou alias de domaine
- Seuls les superadmins et les admins ayant le droit `domain:create` peuvent créer un domaine

### Modification (BR-DOM-02)
- Le nom de domaine ne peut pas être modifié après création (PK immuable)
- La réduction des limites (aliases, mailboxes) ne supprime pas les entités existantes
  mais empêche d'en créer de nouvelles
- La modification du quota total est propagée dans les vérifications de quota Dovecot

### Suppression (BR-DOM-03)
- La suppression d'un domaine entraîne la suppression en cascade de :
  - Toutes les boîtes mail du domaine
  - Tous les alias du domaine
  - Toutes les entrées vacation du domaine
  - Tous les alias de domaine pointant vers ce domaine
  - Toutes les clés DKIM du domaine
  - Toutes les entrées de log du domaine
- Confirmation obligatoire (double validation côté UI et API)
- Les fichiers maildir ne sont PAS supprimés automatiquement (sécurité)

### Activation/Désactivation (BR-DOM-04)
- Un domaine désactivé :
  - Ne reçoit plus de courrier (le lookup Postfix filtre sur `active = true`)
  - Les utilisateurs ne peuvent plus se connecter
  - Les alias ne fonctionnent plus
  - Reste visible dans l'interface d'administration

## Cas d'utilisation

### UC-DOM-01 : Lister les domaines
- **Acteur** : Superadmin, Admin de domaine
- **Entrée** : Filtres optionnels (recherche texte, actif/inactif), pagination
- **Sortie** : Liste paginée avec statistiques (nb alias, nb mailboxes, usage quota)
- **Règle** : Un admin de domaine ne voit que ses domaines assignés

### UC-DOM-02 : Créer un domaine
- **Acteur** : Superadmin
- **Entrée** : Formulaire avec tous les champs de l'entité
- **Validation** : BR-DOM-01
- **Sortie** : Domaine créé, entrée de log

### UC-DOM-03 : Modifier un domaine
- **Acteur** : Superadmin, Admin du domaine (champs limités)
- **Entrée** : Formulaire de modification
- **Validation** : BR-DOM-02
- **Sortie** : Domaine mis à jour, entrée de log

### UC-DOM-04 : Supprimer un domaine
- **Acteur** : Superadmin uniquement
- **Entrée** : Confirmation explicite (saisie du nom de domaine)
- **Validation** : BR-DOM-03
- **Sortie** : Domaine et toutes les entités liées supprimés, entrée de log

### UC-DOM-05 : Activer/Désactiver un domaine
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Toggle actif/inactif
- **Validation** : BR-DOM-04
- **Sortie** : Statut mis à jour, entrée de log

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/domains` | Lister les domaines (paginé, filtrable) |
| `GET` | `/api/v1/domains/{domain}` | Détails d'un domaine |
| `POST` | `/api/v1/domains` | Créer un domaine |
| `PUT` | `/api/v1/domains/{domain}` | Modifier un domaine |
| `DELETE` | `/api/v1/domains/{domain}` | Supprimer un domaine |
| `PATCH` | `/api/v1/domains/{domain}/active` | Activer/désactiver |

## Routes Web

| Route | Vue | Description |
|-------|-----|-------------|
| `GET /domains` | `domain-list.html` | Liste des domaines |
| `GET /domains/new` | `domain-form.html` | Formulaire de création |
| `GET /domains/{domain}/edit` | `domain-form.html` | Formulaire d'édition |
| `POST /domains` | — | Traitement création |
| `POST /domains/{domain}` | — | Traitement modification |
| `POST /domains/{domain}/delete` | — | Traitement suppression |
| `POST /domains/{domain}/toggle` | — | HTMX toggle actif |

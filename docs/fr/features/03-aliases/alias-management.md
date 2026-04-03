> **Language:** [English](../en/features/03-aliases/alias-management.md) | Francais

# SPEC-03.1 — Gestion des alias

## Résumé

Les alias définissent les règles de redirection du courrier. Un alias fait correspondre
une adresse source à une ou plusieurs adresses de destination. C'est le mécanisme
fondamental de routage du courrier virtuel dans Postfix.

## Entité : `Alias`

| Champ        | Type           | Contrainte                | Description                             |
|--------------|----------------|---------------------------|-----------------------------------------|
| `address`    | `VARCHAR(255)` | PK                        | Adresse source (ex: `info@example.com`) |
| `goto`       | `TEXT`         | NOT NULL                  | Destinations séparées par des virgules  |
| `domain`     | `VARCHAR(255)` | FK → `domain.domain`      | Domaine de l'alias                      |
| `active`     | `BOOLEAN`      | NOT NULL, default `true`  | Actif/inactif                           |
| `created_at` | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Date de création                        |
| `updated_at` | `TIMESTAMPTZ`  | NOT NULL, default `now()` | Dernière modification                   |

### Index

- `idx_alias_address_active` : `(address, active)` — Lookup Postfix
- `idx_alias_domain` : `(domain)`

## Types d'alias

### Alias standard
- Une adresse → une ou plusieurs adresses de destination
- Ex: `info@example.com` → `alice@example.com,bob@example.com`

### Alias catch-all
- Adresse sous la forme `@example.com` (pas de partie locale)
- Capture tout le courrier non distribué du domaine
- Priorité basse : les alias explicites et les boîtes mail sont vérifiés d'abord

### Alias automatique (boîte mail)
- Créé automatiquement lors de la création d'une boîte mail
- `user@example.com` → `user@example.com`
- Nécessaire pour que Postfix route le courrier vers Dovecot

### Alias de redirection externe
- Destination vers un domaine non géré localement
- Ex: `forward@example.com` → `user@gmail.com`
- Peut être restreint par configuration (`emailcheck_localaliasonly`)

## Règles métier

### Création (BR-ALI-01)
- L'adresse source doit être valide (RFC 5321) ou de la forme `@domain` (catch-all)
- Le domaine de l'adresse source doit exister dans la table `domain`
- Le nombre d'alias du domaine ne doit pas dépasser `domain.aliases` (si > 0)
- Chaque destination doit être une adresse email valide
- Si `emailcheck_localaliasonly` est activé, les destinations doivent être des domaines locaux
- Pas de boucle directe : `a@x.com → a@x.com` (sauf alias automatique de boîte)

### Modification (BR-ALI-02)
- L'adresse source ne peut pas être modifiée (PK immuable)
- Les destinations sont remplacées intégralement (pas d'ajout/suppression individuel via SQL)

### Suppression (BR-ALI-03)
- Les alias automatiques de boîtes mail ne peuvent être supprimés que via la suppression de la boîte
- La suppression d'un alias n'affecte pas les boîtes mail de destination

### Format `goto` (BR-ALI-04)
- Virgule comme séparateur : `dest1@x.com,dest2@y.com`
- Pas d'espace autour des virgules dans le stockage
- L'affichage dans l'UI présente une destination par ligne
- Nombre maximum de destinations configurable (par défaut : 100)

## Cas d'utilisation

### UC-ALI-01 : Lister les alias d'un domaine
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Domaine, filtres (recherche, actif, type), pagination
- **Sortie** : Liste avec adresse, destinations (tronquées), statut
- **Note** : Les alias automatiques de boîtes sont masqués par défaut (toggle)

### UC-ALI-02 : Créer un alias
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Adresse source (local_part + domaine), destinations (textarea, 1 par ligne)
- **Validation** : BR-ALI-01

### UC-ALI-03 : Modifier les destinations d'un alias
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Nouvelles destinations
- **Validation** : BR-ALI-02

### UC-ALI-04 : Supprimer un alias
- **Acteur** : Superadmin, Admin du domaine
- **Validation** : BR-ALI-03

## Endpoints API

| Méthode  | Route                              | Description        |
|----------|------------------------------------|--------------------|
| `GET`    | `/api/v1/domains/{domain}/aliases` | Lister les alias   |
| `GET`    | `/api/v1/aliases/{address}`        | Détails d'un alias |
| `POST`   | `/api/v1/domains/{domain}/aliases` | Créer un alias     |
| `PUT`    | `/api/v1/aliases/{address}`        | Modifier un alias  |
| `DELETE` | `/api/v1/aliases/{address}`        | Supprimer un alias |
| `PATCH`  | `/api/v1/aliases/{address}/active` | Activer/désactiver |

## Notes d'intégration Postfix

La requête SQL typique pour le lookup Postfix `virtual_alias_maps` :

```sql
SELECT goto FROM alias WHERE address = '%s' AND active = true
UNION
SELECT goto FROM alias WHERE address = '@%d' AND active = true
```

La seconde partie gère le catch-all. L'ordre UNION garantit que les alias
explicites sont retournés en premier.

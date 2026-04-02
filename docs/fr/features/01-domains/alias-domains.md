> **Language:** [English](../en/features/01-domains/alias-domains.md) | Francais

# SPEC-01.2 — Domaines alias (Alias Domains)

## Résumé

Un domaine alias redirige l'intégralité du courrier d'un domaine vers un autre domaine.
Par exemple, `alias-example.com` → `example.com` : tout mail envoyé à `user@alias-example.com`
est délivré à `user@example.com`.

## Entité : `AliasDomain`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `alias_domain` | `VARCHAR(255)` | PK | Domaine source (alias) |
| `target_domain` | `VARCHAR(255)` | FK → `domain.domain` | Domaine cible |
| `active` | `BOOLEAN` | NOT NULL, default `true` | Actif/inactif |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Dernière modification |

### Index

- `idx_alias_domain_active` : `(alias_domain, active)`
- `idx_alias_domain_target` : `(target_domain)`

## Règles métier

### BR-ADOM-01 : Création
- Le domaine alias ne doit pas exister en tant que domaine réel dans la table `domain`
- Le domaine alias ne doit pas déjà exister dans `alias_domain`
- Le domaine cible doit exister dans la table `domain` et être actif
- Pas de boucles : le domaine cible ne peut pas être lui-même un alias de domaine
- Validation DNS identique aux domaines (RFC 1035)

### BR-ADOM-02 : Suppression
- La suppression est simple (pas de cascade complexe)
- Le courrier pour le domaine alias ne sera plus délivré

### BR-ADOM-03 : Interaction avec les alias classiques
- Les alias classiques (table `alias`) ont priorité sur les alias de domaine
- Si `admin@alias-example.com` a un alias explicite, celui-ci est utilisé
  plutôt que la redirection vers `admin@example.com`

## Cas d'utilisation

### UC-ADOM-01 : Lister les domaines alias
- **Acteur** : Superadmin, Admin des domaines concernés
- **Sortie** : Liste paginée (alias_domain → target_domain, statut)

### UC-ADOM-02 : Créer un domaine alias
- **Acteur** : Superadmin
- **Entrée** : Domaine alias + domaine cible (select parmi les domaines existants)
- **Validation** : BR-ADOM-01

### UC-ADOM-03 : Supprimer un domaine alias
- **Acteur** : Superadmin
- **Validation** : BR-ADOM-02, confirmation requise

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/alias-domains` | Lister les domaines alias |
| `POST` | `/api/v1/alias-domains` | Créer un domaine alias |
| `DELETE` | `/api/v1/alias-domains/{alias_domain}` | Supprimer |
| `PATCH` | `/api/v1/alias-domains/{alias_domain}/active` | Activer/désactiver |

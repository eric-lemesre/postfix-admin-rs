# SPEC-02.2 — Gestion des quotas

## Résumé

Système de quotas à deux niveaux (domaine et boîte mail individuelle) intégré avec
Dovecot pour le suivi en temps réel de l'utilisation du stockage.

## Entités

### `Quota` (suivi Dovecot — table legacy)

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `username` | `VARCHAR(255)` | PK (composite) | Adresse email |
| `path` | `VARCHAR(100)` | PK (composite) | Chemin de stockage |
| `current` | `BIGINT` | default `0` | Utilisation courante en octets |

### `Quota2` (suivi Dovecot >= 1.2)

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `username` | `VARCHAR(100)` | PK | Adresse email |
| `bytes` | `BIGINT` | default `0` | Octets utilisés |
| `messages` | `INTEGER` | NOT NULL, default `0` | Nombre de messages |

## Niveaux de quota

### Quota par boîte mail
- Défini dans `mailbox.quota` (en octets)
- `0` = illimité (ou limité uniquement par le quota domaine)
- Ne peut pas dépasser `domain.maxquota` (si celui-ci > 0)

### Quota par domaine
- Défini dans `domain.quota` (en Mo)
- Somme de tous les quotas des boîtes du domaine
- `0` = illimité

### Quota max par boîte (niveau domaine)
- Défini dans `domain.maxquota` (en Mo)
- Plafond individuel pour chaque boîte du domaine
- `0` = pas de plafond individuel

## Règles métier

### BR-QUO-01 : Vérification à la création/modification
```
Si domain.maxquota > 0 :
    mailbox.quota <= domain.maxquota * 1024 * 1024

Si domain.quota > 0 :
    SUM(mailbox.quota pour le domaine) <= domain.quota * 1024 * 1024
```

### BR-QUO-02 : Affichage
- Conversion automatique des unités (octets → Ko → Mo → Go)
- Barre de progression visuelle (vert < 70%, orange 70-90%, rouge > 90%)
- Indicateur spécial pour les boîtes qui ont dépassé leur quota

### BR-QUO-03 : Intégration Dovecot
- Les tables `quota` et `quota2` sont gérées par Dovecot (lecture seule côté app)
- L'application lit ces tables pour afficher l'utilisation courante
- La configuration Dovecot pointe vers ces tables pour l'enforcement

## Cas d'utilisation

### UC-QUO-01 : Voir l'utilisation des quotas d'un domaine
- **Acteur** : Superadmin, Admin du domaine
- **Sortie** : Tableau récapitulatif par boîte, total domaine, pourcentages

### UC-QUO-02 : Voir son propre quota
- **Acteur** : Utilisateur
- **Sortie** : Usage courant, limite, pourcentage, nombre de messages

### UC-QUO-03 : Modifier le quota d'une boîte
- **Acteur** : Superadmin, Admin du domaine
- **Validation** : BR-QUO-01

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/domains/{domain}/quota` | Résumé quota du domaine |
| `GET` | `/api/v1/mailboxes/{username}/quota` | Quota détaillé d'une boîte |

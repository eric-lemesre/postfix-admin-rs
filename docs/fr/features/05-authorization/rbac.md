> **Language:** [English](../en/features/05-authorization/rbac.md) | Francais

# SPEC-05.1 — Contrôle d'accès basé sur les rôles (RBAC)

## Résumé

Système d'autorisation à trois niveaux de rôles avec un modèle de permissions
hiérarchique. Chaque action de l'application est protégée par une vérification
de rôle et de périmètre.

## Rôles

### Superadmin
- Accès total à toutes les fonctionnalités
- Peut créer/modifier/supprimer des domaines
- Peut créer/modifier/supprimer des administrateurs
- Peut assigner des administrateurs à des domaines
- Peut gérer la configuration globale
- Peut voir et gérer tous les domaines sans restriction

### Admin de domaine
- Accès limité aux domaines qui lui sont assignés (table `domain_admins`)
- Peut gérer les boîtes mail, alias et vacation de ses domaines
- Peut voir les statistiques et logs de ses domaines
- Ne peut PAS créer de nouveaux domaines
- Ne peut PAS gérer les autres administrateurs
- Ne peut PAS accéder à la configuration globale

### Utilisateur (mailbox user)
- Accès uniquement à ses propres données
- Peut changer son mot de passe
- Peut gérer son vacation/répondeur
- Peut gérer ses mots de passe applicatifs
- Peut activer/désactiver le TOTP 2FA
- Aucun accès aux fonctions d'administration

## Matrice de permissions

| Ressource | Action | Superadmin | Admin domaine | Utilisateur |
|-----------|--------|:----------:|:-------------:|:-----------:|
| **Domaines** | Lister | Tous | Ses domaines | — |
| | Créer | Oui | — | — |
| | Modifier | Oui | Ses domaines (limité) | — |
| | Supprimer | Oui | — | — |
| **Alias domaines** | CRUD | Oui | — | — |
| **Admins** | Lister | Oui | — | — |
| | Créer | Oui | — | — |
| | Modifier | Oui | Soi-même (mot de passe) | — |
| | Supprimer | Oui | — | — |
| **Boîtes mail** | Lister | Toutes | Ses domaines | — |
| | Créer | Oui | Ses domaines | — |
| | Modifier | Oui | Ses domaines | Soi-même (limité) |
| | Supprimer | Oui | Ses domaines | — |
| **Alias** | Lister | Tous | Ses domaines | — |
| | Créer | Oui | Ses domaines | — |
| | Modifier | Oui | Ses domaines | — |
| | Supprimer | Oui | Ses domaines | — |
| **Vacation** | Gérer | Oui | Ses domaines | Soi-même |
| **Fetchmail** | Gérer | Oui | Ses domaines | Soi-même |
| **DKIM** | Gérer | Oui | Ses domaines | — |
| **Logs** | Consulter | Tous | Ses domaines | — |
| **Configuration** | Modifier | Oui | — | — |
| **App passwords** | Gérer | — | — | Soi-même |
| **TOTP** | Gérer | Soi-même | Soi-même | Soi-même |
| **TOTP exceptions** | Gérer | Oui | — | — |
| **Broadcast** | Envoyer | Oui | — | — |

## Implémentation

### Middleware d'autorisation (axum)

Trois extracteurs axum distincts :

```
RequireSuperAdmin    → Vérifie le rôle superadmin
RequireDomainAdmin   → Vérifie le rôle admin + accès au domaine demandé
RequireUser          → Vérifie l'identité de l'utilisateur
```

### Vérification de périmètre

Pour les admin de domaine, chaque requête vérifie :

```
1. L'admin est authentifié et actif
2. Le domaine cible existe dans domain_admins pour cet admin
3. La ressource (mailbox, alias...) appartient bien au domaine autorisé
```

### Règles de périmètre pour Admin de domaine

Champs de domaine modifiables par un admin de domaine :
- `description` — oui
- `aliases`, `mailboxes`, `maxquota`, `quota` — non (fixés par le superadmin)
- `transport`, `backupmx` — non
- `active` — selon configuration (`domain_admin_can_disable`)

## Règles métier

### BR-RBAC-01 : Attribution des domaines
- Un admin peut être assigné à 0..N domaines
- Un domaine peut avoir 0..N admins
- Seul un superadmin peut modifier les attributions

### BR-RBAC-02 : Auto-protection superadmin
- Le dernier superadmin ne peut pas être supprimé
- Un superadmin ne peut pas se retirer le statut superadmin à lui-même
  (un autre superadmin doit le faire)

### BR-RBAC-03 : Cumul de rôles
- Un superadmin est implicitement admin de tous les domaines
- Il n'est pas nécessaire de créer des entrées dans `domain_admins` pour un superadmin
- Un admin peut être à la fois admin et avoir une boîte mail (double session possible)

> **Language:** [English](../en/features/04-authentication/user-authentication.md) | Francais

# SPEC-04.2 — Authentification des utilisateurs

## Statut d'implémentation

| Composant                                         | Crate                | Statut   | Milestone |
|---------------------------------------------------|----------------------|----------|-----------|
| Modèle (`Mailbox` — champs auth)                  | `postfix-admin-core` | Fait     | M1        |
| Hashage et vérification des mots de passe         | `postfix-admin-auth` | Fait     | M4        |
| Gestion de session (HttpOnly, Secure, SameSite)   | `postfix-admin-auth` | Fait     | M4        |
| Génération et rafraîchissement JWT                | `postfix-admin-auth` | Fait     | M4        |
| TOTP génération, vérification, codes récupération | `postfix-admin-auth` | Fait     | M4        |
| Génération et hashage app passwords               | `postfix-admin-auth` | Fait     | M4        |
| Page de login Web UI                              | `postfix-admin-web`  | En cours | M6        |

## Résumé

Interface en libre-service pour les utilisateurs de boîtes mail. Permet la gestion
du mot de passe, du répondeur automatique et du profil. L'authentification utilise
les mêmes credentials que la connexion IMAP/SMTP (table `mailbox`).

## Flux d'authentification utilisateur

L'utilisateur se connecte avec son adresse email et son mot de passe mail.
La vérification se fait contre `mailbox.password`.

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ User Login  │────▶│ Vérification │────▶│  Dashboard   │
│ (email/pass)│     │ vs mailbox   │     │  Utilisateur │
└─────────────┘     └──────────────┘     └──────────────┘
```

## Différences avec l'authentification admin

| Aspect      | Admin          | Utilisateur                |
|-------------|----------------|----------------------------|
| Table       | `admin`        | `mailbox`                  |
| Identifiant | admin username | adresse email              |
| TOTP 2FA    | Supporté       | Supporté (optionnel)       |
| Session     | Même mécanisme | Même mécanisme             |
| Portée      | Multi-domaines | Sa propre boîte uniquement |

## Fonctions accessibles à l'utilisateur

| Fonction                   | Description                                 |
|----------------------------|---------------------------------------------|
| Changement de mot de passe | Ancien + nouveau mot de passe               |
| Gestion vacation           | Activation/édition du répondeur automatique |
| Profil                     | Modification du nom affiché                 |
| TOTP 2FA                   | Activation/désactivation du 2FA             |
| Mots de passe applicatifs  | Gestion des app passwords                   |

## Règles métier

### BR-UAUTH-01 : Login
- Identique à BR-AUTH-01 mais vérifie contre `mailbox.password`
- Vérifie aussi `mailbox.active = true` et `domain.active = true`
- Si `password_expiry` est défini et dépassé → redirection vers changement de mot de passe

### BR-UAUTH-02 : Changement de mot de passe
- L'ancien mot de passe doit être vérifié
- Règles de complexité configurables :
  - Longueur minimale (défaut : 8)
  - Au moins une majuscule, une minuscule, un chiffre (configurable)
- Le nouveau mot de passe ne peut pas être identique à l'ancien
- Mise à jour de `password_expiry` si configuré au niveau domaine

### BR-UAUTH-03 : Isolation
- Un utilisateur ne peut accéder qu'à ses propres données
- Pas de visibilité sur les autres utilisateurs du domaine
- Pas d'accès aux fonctions d'administration

## Routes Web

| Route                 | Méthode  | Description                     |
|-----------------------|----------|---------------------------------|
| `/user/login`         | GET      | Formulaire de login utilisateur |
| `/user/login`         | POST     | Traitement du login             |
| `/user/logout`        | POST     | Déconnexion                     |
| `/user/dashboard`     | GET      | Tableau de bord utilisateur     |
| `/user/password`      | GET/POST | Changement de mot de passe      |
| `/user/vacation`      | GET/POST | Gestion du vacation             |
| `/user/profile`       | GET/POST | Modification du profil          |
| `/user/totp`          | GET/POST | Configuration TOTP              |
| `/user/app-passwords` | GET/POST | Gestion app passwords           |

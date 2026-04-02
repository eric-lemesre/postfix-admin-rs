> **Language:** [English](../en/features/04-authentication/admin-authentication.md) | Francais

# SPEC-04.1 — Authentification des administrateurs

## Résumé

Système d'authentification pour les comptes administrateurs (superadmin et admin de domaine).
Authentification par formulaire web avec session serveur, support optionnel TOTP 2FA.

## Entité : `Admin`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `username` | `VARCHAR(255)` | PK | Identifiant admin (email) |
| `password` | `VARCHAR(255)` | NOT NULL | Hash du mot de passe |
| `superadmin` | `BOOLEAN` | NOT NULL, default `false` | Privilège superadmin |
| `totp_secret` | `VARCHAR(255)` | NULLABLE | Secret TOTP chiffré |
| `totp_enabled` | `BOOLEAN` | NOT NULL, default `false` | 2FA activé |
| `token` | `VARCHAR(255)` | NULLABLE | Token de récupération de mot de passe |
| `token_validity` | `TIMESTAMPTZ` | NULLABLE | Expiration du token |
| `active` | `BOOLEAN` | NOT NULL, default `true` | Compte actif/inactif |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Dernière modification |

### Entité associée : `DomainAdmin`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `username` | `VARCHAR(255)` | FK → `admin.username` | Identifiant admin |
| `domain` | `VARCHAR(255)` | FK → `domain.domain` | Domaine administré |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date d'attribution |

PK composite : `(username, domain)`

## Flux d'authentification

```
┌─────────┐     ┌──────────────┐     ┌──────────────┐     ┌────────────┐
│  Login   │────▶│ Vérification │────▶│  TOTP 2FA ?  │────▶│  Session   │
│  Form    │     │  Password    │     │  (si activé) │     │  Créée     │
└─────────┘     └──────────────┘     └──────────────┘     └────────────┘
                       │                     │
                       ▼                     ▼
                 ┌──────────┐          ┌──────────┐
                 │  Échec   │          │  Échec   │
                 │  (log)   │          │  (log)   │
                 └──────────┘          └──────────┘
```

## Règles métier

### Login (BR-AUTH-01)
- Vérification du couple username/password
- Si le hash utilise un ancien schéma et l'auth réussit → rehash transparent
- Compte `active = false` → refus avec message générique
- Rate limiting : 5 tentatives max par IP sur 15 minutes (configurable)
- Log de chaque tentative (succès et échec)

### Session (BR-AUTH-02)
- Session serveur (cookie HttpOnly, Secure, SameSite=Strict)
- Durée de session configurable (par défaut : 1 heure)
- Régénération de l'ID de session après authentification (prévention session fixation)
- Invalidation automatique après inactivité
- Stockage session : en mémoire (par défaut) ou Redis (optionnel)

### Récupération de mot de passe (BR-AUTH-03)
- Génération d'un token aléatoire (256 bits, encodé base64url)
- Validité du token : 1 heure (configurable)
- Envoi par email via le serveur SMTP local
- Le token est hashé en base (on ne stocke pas le token en clair)
- Un seul token actif par admin à la fois

### Brute-force protection (BR-AUTH-04)
- Compteur d'échecs par IP et par username
- Après N échecs → délai progressif (1s, 2s, 4s, 8s...)
- Après M échecs → blocage temporaire de l'IP (15 min)
- Les informations de blocage sont en mémoire (pas en BDD)
- Header `X-Forwarded-For` respecté si configuré (reverse proxy)

## Cas d'utilisation

### UC-AUTH-01 : Login admin
- **Entrée** : username, password
- **Sortie** : Session créée, redirection vers dashboard — ou page TOTP si 2FA activé

### UC-AUTH-02 : Logout
- **Entrée** : Action utilisateur
- **Sortie** : Session détruite, redirection vers login

### UC-AUTH-03 : Récupération de mot de passe
- **Entrée** : Adresse email admin
- **Sortie** : Email avec lien de réinitialisation (si le compte existe)
- **Sécurité** : Même réponse que le compte existe ou non (timing-safe)

### UC-AUTH-04 : Réinitialisation de mot de passe
- **Entrée** : Token + nouveau mot de passe (x2)
- **Validation** : Token valide et non expiré

## Routes Web

| Route | Méthode | Description |
|-------|---------|-------------|
| `/login` | GET | Formulaire de login |
| `/login` | POST | Traitement du login |
| `/logout` | POST | Déconnexion |
| `/password-recover` | GET | Formulaire de récupération |
| `/password-recover` | POST | Envoi du token |
| `/password-reset/{token}` | GET | Formulaire de nouveau mot de passe |
| `/password-reset/{token}` | POST | Traitement du reset |

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `POST` | `/api/v1/auth/login` | Authentification (retourne JWT) |
| `POST` | `/api/v1/auth/logout` | Invalidation du token |
| `POST` | `/api/v1/auth/refresh` | Rafraîchissement du token |
| `POST` | `/api/v1/auth/totp/verify` | Vérification TOTP |

## Notes de sécurité

- Les messages d'erreur ne distinguent pas "utilisateur inconnu" de "mauvais mot de passe"
- Les comparaisons de mots de passe sont constantes en temps (timing-safe)
- Les tokens de session et de récupération utilisent un CSPRNG
- CSRF protection sur tous les formulaires POST

# SPEC-04.3 — Authentification TOTP à deux facteurs

## Résumé

Support de l'authentification à deux facteurs (2FA) via TOTP (Time-based One-Time Password,
RFC 6238) pour les comptes admin et utilisateur. Compatible avec les applications
d'authentification standards (Google Authenticator, Authy, FreeOTP, etc.).

## Paramètres TOTP

| Paramètre | Valeur |
|-----------|--------|
| Algorithme | SHA-1 (compatibilité maximale avec les apps) |
| Période | 30 secondes |
| Digits | 6 |
| Fenêtre de tolérance | 1 période avant/après (±30s) |

## Entité : `TotpExceptionAddress`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `id` | `SERIAL` | PK | Identifiant auto-incrémenté |
| `ip` | `VARCHAR(46)` | NOT NULL | Adresse IP (v4 ou v6) |
| `username` | `VARCHAR(255)` | NULLABLE | Admin/user (NULL = global) |
| `description` | `VARCHAR(255)` | NULLABLE | Description de l'exception |

Contrainte unique : `(ip, username)`

## Flux d'activation du 2FA

```
1. L'utilisateur accède à la page TOTP
2. Le serveur génère un secret aléatoire (160 bits, base32)
3. Un QR code est affiché (URI otpauth://totp/...)
4. Le secret est aussi affiché en clair (saisie manuelle)
5. L'utilisateur scanne le QR code dans son app
6. L'utilisateur saisit un code de vérification
7. Si le code est valide → le secret est sauvegardé (chiffré) et le 2FA est activé
8. Des codes de récupération sont générés et affichés une seule fois
```

## Codes de récupération

- 10 codes à usage unique générés à l'activation
- Format : `XXXX-XXXX` (8 caractères alphanumériques groupés)
- Stockés hashés en base de données
- Chaque code ne peut être utilisé qu'une seule fois
- Possibilité de régénérer tous les codes (invalide les anciens)

## Règles métier

### BR-TOTP-01 : Activation
- L'utilisateur doit être authentifié (premier facteur)
- Un code de vérification valide doit être fourni avant activation
- Le secret est chiffré en base avec une clé de l'application (AES-256-GCM)
- Les codes de récupération sont affichés une seule fois (pas de récupération possible)

### BR-TOTP-02 : Vérification
- Le code est vérifié après l'authentification par mot de passe
- Fenêtre de tolérance : le code courant + 1 période avant + 1 période après
- En cas d'échec → possibilité d'utiliser un code de récupération
- Le même code TOTP ne peut pas être réutilisé (protection contre le replay)
  → Stockage du dernier timestamp TOTP validé

### BR-TOTP-03 : Exceptions IP
- Certaines adresses IP peuvent être exemptées du 2FA
- Exceptions globales (sans username) : s'appliquent à tous
- Exceptions par utilisateur : s'appliquent à un compte spécifique
- Cas d'usage : serveurs de monitoring, réseaux internes

### BR-TOTP-04 : Désactivation
- L'utilisateur doit fournir un code TOTP valide pour désactiver le 2FA
- Alternative : un superadmin peut forcer la désactivation (reset)
- La désactivation supprime le secret et les codes de récupération

## Cas d'utilisation

### UC-TOTP-01 : Activer le 2FA
- **Acteur** : Admin ou Utilisateur
- **Pré-condition** : Authentifié, 2FA non activé
- **Sortie** : QR code, codes de récupération

### UC-TOTP-02 : Se connecter avec 2FA
- **Acteur** : Admin ou Utilisateur
- **Entrée** : Code TOTP à 6 chiffres ou code de récupération
- **Sortie** : Session complètement authentifiée

### UC-TOTP-03 : Gérer les exceptions IP
- **Acteur** : Superadmin
- **Entrée** : IP (v4/v6), username optionnel, description
- **Sortie** : Exception créée/supprimée

### UC-TOTP-04 : Reset 2FA d'un utilisateur
- **Acteur** : Superadmin
- **Entrée** : Username de l'admin/utilisateur
- **Sortie** : 2FA désactivé, secret supprimé

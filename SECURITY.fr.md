> **Language:** [English](SECURITY.md) | Francais

# Politique de securite — postfix-admin-rs

## Versions supportees

| Version | Support securite |
|---------|-----------------|
| 1.x.x   | Oui (actif)     |
| < 1.0   | Non             |

## Signaler une vulnerabilite

**Ne signalez PAS les vulnerabilites de securite via les issues GitHub publiques.**

Pour signaler une vulnerabilite :

1. Envoyez un email a **security@example.com** (a mettre a jour avec l'adresse reelle)
2. Incluez :
   - Description de la vulnerabilite
   - Etapes pour reproduire
   - Impact potentiel
   - Suggestion de correction (si possible)

### Delai de reponse

- Accuse de reception : 48 heures
- Evaluation initiale : 7 jours
- Correction : selon la severite (critique < 7 jours, haute < 30 jours)

### Processus

1. Reception et accuse de reception
2. Evaluation et confirmation de la vulnerabilite
3. Developpement du correctif
4. Publication d'une version corrigee
5. Divulgation responsable (apres le correctif)

## Bonnes pratiques de securite

### Pour les administrateurs

- Toujours utiliser HTTPS (TLS) avec un reverse proxy
- Garder postfix-admin-rs a jour
- Utiliser des mots de passe forts pour les comptes admin
- Activer le TOTP 2FA pour tous les comptes admin
- Utiliser des certificats clients (mTLS) pour les comptes superadmin si possible
- Restreindre l'acces reseau a l'interface d'administration
- Sauvegarder regulierement la base de donnees
- Surveiller les logs d'audit

### Pour les developpeurs

- Suivre les [guidelines de securite](docs/fr/guidelines/GUIDELINES-Rust.md#10-securite)
- Utiliser exclusivement les requetes parametrees
- Ne jamais logger les mots de passe ou secrets
- Utiliser `cargo audit` regulierement
- Les mots de passe sont hashes avec argon2id
- Les secrets au repos sont chiffres (AES-256-GCM)

## Dependances

Les dependances sont auditees regulierement avec `cargo audit`.
Les mises a jour de securite des dependances sont traitees en priorite.

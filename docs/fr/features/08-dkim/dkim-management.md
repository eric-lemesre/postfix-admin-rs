> **Language:** [English](../en/features/08-dkim/dkim-management.md) | Francais

# SPEC-08.1 — Gestion DKIM

## Résumé

Gestion des clés DKIM (DomainKeys Identified Mail) et des tables de signature
pour l'intégration avec OpenDKIM. Permet de générer, stocker et gérer les
paires de clés cryptographiques utilisées pour signer le courrier sortant.

## Entité : `DkimKey`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `id` | `SERIAL` | PK | Identifiant auto-incrémenté |
| `domain_name` | `VARCHAR(255)` | FK → `domain.domain` ON DELETE CASCADE | Domaine |
| `description` | `VARCHAR(255)` | default `''` | Description de la clé |
| `selector` | `VARCHAR(63)` | NOT NULL, default `'default'` | Sélecteur DKIM |
| `private_key` | `TEXT` | NOT NULL | Clé privée (PEM, chiffrée au repos) |
| `public_key` | `TEXT` | NOT NULL | Clé publique (PEM) |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Dernière modification |

Index : `(domain_name, description)`

## Entité : `DkimSigning`

| Champ | Type | Contrainte | Description |
|-------|------|-----------|-------------|
| `id` | `SERIAL` | PK | Identifiant auto-incrémenté |
| `author` | `VARCHAR(255)` | NOT NULL | Pattern d'auteur (ex: `*@example.com`) |
| `dkim_id` | `INTEGER` | FK → `dkim_key.id` ON DELETE CASCADE | Clé DKIM à utiliser |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Date de création |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | Dernière modification |

Index : `(author)`

## Règles métier

### BR-DKIM-01 : Génération de clé
- Algorithme : RSA 2048 bits (configurable : 1024, 2048, 4096)
- Le sélecteur doit être unique par domaine
- La clé privée est chiffrée au repos (AES-256-GCM)
- La clé publique est affichée en format d'enregistrement DNS

### BR-DKIM-02 : Table de signature
- Le pattern `author` supporte les wildcards : `*@example.com`, `user@example.com`
- Un domaine peut avoir plusieurs clés (rotation)
- Une seule entrée de signature est active par pattern

### BR-DKIM-03 : Intégration OpenDKIM
- Export possible en format compatible OpenDKIM :
  - `KeyTable` : `selector._domainkey.example.com example.com:selector:/path/to/key`
  - `SigningTable` : `*@example.com selector._domainkey.example.com`
- Endpoint API pour qu'OpenDKIM puisse requêter les clés dynamiquement

### BR-DKIM-04 : Enregistrement DNS
- Affichage de l'enregistrement TXT à créer :
  ```
  selector._domainkey.example.com. IN TXT "v=DKIM1; k=rsa; p=<clé_publique_base64>"
  ```
- Vérification DNS optionnelle : l'app peut vérifier que l'enregistrement est en place

## Cas d'utilisation

### UC-DKIM-01 : Générer une clé DKIM
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Domaine, sélecteur, taille de clé
- **Sortie** : Paire de clés générée, enregistrement DNS à créer

### UC-DKIM-02 : Configurer la signature
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Pattern d'auteur, clé DKIM à utiliser
- **Sortie** : Règle de signature créée

### UC-DKIM-03 : Rotation de clé
- **Acteur** : Superadmin, Admin du domaine
- **Processus** :
  1. Générer une nouvelle clé avec un nouveau sélecteur
  2. Publier le nouvel enregistrement DNS
  3. Attendre la propagation DNS
  4. Mettre à jour la table de signature
  5. Supprimer l'ancienne clé (optionnel)

### UC-DKIM-04 : Vérifier la configuration DNS
- **Acteur** : Superadmin, Admin du domaine
- **Entrée** : Domaine + sélecteur
- **Sortie** : Statut de l'enregistrement DNS DKIM

## Endpoints API

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET` | `/api/v1/domains/{domain}/dkim/keys` | Lister les clés |
| `POST` | `/api/v1/domains/{domain}/dkim/keys` | Générer une clé |
| `DELETE` | `/api/v1/dkim/keys/{id}` | Supprimer une clé |
| `GET` | `/api/v1/domains/{domain}/dkim/signing` | Lister les règles de signature |
| `POST` | `/api/v1/domains/{domain}/dkim/signing` | Créer une règle |
| `DELETE` | `/api/v1/dkim/signing/{id}` | Supprimer une règle |
| `GET` | `/api/v1/domains/{domain}/dkim/dns-check` | Vérifier DNS |

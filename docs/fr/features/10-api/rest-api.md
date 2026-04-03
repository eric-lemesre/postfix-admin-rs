> **Language:** [English](../en/features/10-api/rest-api.md) | Francais

# SPEC-10.1 — API REST

## Résumé

API RESTful JSON complète pour l'administration programmatique du serveur mail.
Remplace le XMLRPC optionnel de PostfixAdmin PHP par une API moderne, documentée
et versionée.

## Principes

- **Versionage** : Préfixe `/api/v1/` pour toutes les routes
- **Format** : JSON (Content-Type: application/json)
- **Authentification** : Bearer token (JWT) ou API key
- **Pagination** : Offset-based avec headers `X-Total-Count`, `Link`
- **Erreurs** : Format RFC 7807 (Problem Details)
- **Documentation** : OpenAPI 3.1 auto-générée

## Authentification API

### JWT (sessions temporaires)
```
POST /api/v1/auth/login
Body: { "username": "admin@example.com", "password": "...", "totp": "123456" }
Response: { "token": "eyJ...", "expires_at": "2024-...", "refresh_token": "..." }
```

- Le token JWT a une durée de vie courte (15 min, configurable)
- Le refresh token a une durée de vie longue (7 jours)
- Le TOTP est requis si le 2FA est activé

### API Key (accès permanent)
- Clé générée depuis l'interface admin
- Envoyée via header : `Authorization: Bearer par_key_xxxx`
- Associée à un admin avec des permissions identiques
- Révocable à tout moment

## Format de réponse

### Succès (objet unique)
```json
{
  "data": {
    "domain": "example.com",
    "description": "Example Domain",
    "active": true
  }
}
```

### Succès (collection)
```json
{
  "data": [
    { "domain": "example.com", ... },
    { "domain": "other.com", ... }
  ],
  "meta": {
    "total": 42,
    "page": 1,
    "per_page": 20
  }
}
```

### Erreur (RFC 7807)
```json
{
  "type": "https://postfix-admin-rs.dev/errors/validation",
  "title": "Validation Error",
  "status": 422,
  "detail": "Le domaine 'invalid' n'est pas valide",
  "errors": [
    { "field": "domain", "message": "Format de domaine invalide" }
  ]
}
```

## Codes HTTP utilisés

| Code  | Usage                                      |
|-------|--------------------------------------------|
| `200` | Succès (GET, PUT, PATCH)                   |
| `201` | Création réussie (POST)                    |
| `204` | Suppression réussie (DELETE)               |
| `400` | Requête malformée                          |
| `401` | Non authentifié                            |
| `403` | Pas autorisé (permissions insuffisantes)   |
| `404` | Ressource non trouvée                      |
| `409` | Conflit (doublon, violation de contrainte) |
| `422` | Erreur de validation                       |
| `429` | Rate limit dépassé                         |
| `500` | Erreur serveur                             |

## Pagination

### Requête
```
GET /api/v1/domains?page=2&per_page=20&sort=domain&order=asc
```

### Paramètres communs

| Paramètre  | Type    | Défaut | Description                  |
|------------|---------|--------|------------------------------|
| `page`     | integer | 1      | Page courante                |
| `per_page` | integer | 20     | Éléments par page (max: 100) |
| `sort`     | string  | varie  | Champ de tri                 |
| `order`    | string  | `asc`  | Ordre (`asc`, `desc`)        |
| `search`   | string  | —      | Recherche textuelle          |
| `active`   | boolean | —      | Filtrer par statut actif     |

### Headers de réponse
```
X-Total-Count: 42
Link: </api/v1/domains?page=3&per_page=20>; rel="next",
      </api/v1/domains?page=1&per_page=20>; rel="prev"
```

## Rate Limiting

- Par défaut : 100 requêtes / minute par token
- Headers de réponse : `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
- Configurable par rôle (superadmin peut avoir une limite plus haute)

## Catalogue des endpoints

### Authentification
| Méthode | Route                  | Description        |
|---------|------------------------|--------------------|
| `POST`  | `/api/v1/auth/login`   | Obtenir un JWT     |
| `POST`  | `/api/v1/auth/refresh` | Rafraîchir le JWT  |
| `POST`  | `/api/v1/auth/logout`  | Invalider le token |

### Domaines
| Méthode  | Route                             | Description  |
|----------|-----------------------------------|--------------|
| `GET`    | `/api/v1/domains`                 | Lister       |
| `GET`    | `/api/v1/domains/{domain}`        | Détails      |
| `POST`   | `/api/v1/domains`                 | Créer        |
| `PUT`    | `/api/v1/domains/{domain}`        | Modifier     |
| `DELETE` | `/api/v1/domains/{domain}`        | Supprimer    |
| `PATCH`  | `/api/v1/domains/{domain}/active` | Toggle actif |

### Alias domaines
| Méthode  | Route                           | Description |
|----------|---------------------------------|-------------|
| `GET`    | `/api/v1/alias-domains`         | Lister      |
| `POST`   | `/api/v1/alias-domains`         | Créer       |
| `DELETE` | `/api/v1/alias-domains/{alias}` | Supprimer   |

### Boîtes mail
| Méthode  | Route                                   | Description          |
|----------|-----------------------------------------|----------------------|
| `GET`    | `/api/v1/domains/{domain}/mailboxes`    | Lister par domaine   |
| `GET`    | `/api/v1/mailboxes/{username}`          | Détails              |
| `POST`   | `/api/v1/domains/{domain}/mailboxes`    | Créer                |
| `PUT`    | `/api/v1/mailboxes/{username}`          | Modifier             |
| `DELETE` | `/api/v1/mailboxes/{username}`          | Supprimer            |
| `POST`   | `/api/v1/mailboxes/{username}/password` | Changer mot de passe |

### Alias
| Méthode  | Route                              | Description        |
|----------|------------------------------------|--------------------|
| `GET`    | `/api/v1/domains/{domain}/aliases` | Lister par domaine |
| `GET`    | `/api/v1/aliases/{address}`        | Détails            |
| `POST`   | `/api/v1/domains/{domain}/aliases` | Créer              |
| `PUT`    | `/api/v1/aliases/{address}`        | Modifier           |
| `DELETE` | `/api/v1/aliases/{address}`        | Supprimer          |

### Admins
| Méthode  | Route                       | Description |
|----------|-----------------------------|-------------|
| `GET`    | `/api/v1/admins`            | Lister      |
| `GET`    | `/api/v1/admins/{username}` | Détails     |
| `POST`   | `/api/v1/admins`            | Créer       |
| `PUT`    | `/api/v1/admins/{username}` | Modifier    |
| `DELETE` | `/api/v1/admins/{username}` | Supprimer   |

### Vacation, Fetchmail, DKIM, Logs
Voir les spécifications respectives pour les endpoints détaillés.

## Documentation OpenAPI

- Endpoint : `GET /api/v1/openapi.json`
- Interface Swagger UI intégrée : `GET /api/docs`
- Génération automatique depuis les types Rust via `utoipa`

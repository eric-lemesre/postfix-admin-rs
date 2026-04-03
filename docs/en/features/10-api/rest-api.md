> **Language:** English | [Francais](../fr/features/10-api/rest-api.md)

---
# SPEC-10.1 — REST API

## Implementation Status

| Component                       | Crate                | Status  | Milestone |
|---------------------------------|----------------------|---------|-----------|
| Pagination types                | `postfix-admin-core` | Done    | M1        |
| Error types (for API responses) | `postfix-admin-core` | Done    | M1        |
| API router setup                | `postfix-admin-api`  | Pending | M6        |
| JWT authentication middleware   | `postfix-admin-auth` | Pending | M4        |
| RFC 7807 error handling         | `postfix-admin-api`  | Pending | M6        |
| OpenAPI generation (utoipa)     | `postfix-admin-api`  | Pending | M6        |
| All CRUD endpoints              | `postfix-admin-api`  | Pending | M6        |
| Newman test collections         | `tests/newman/`      | Pending | M14       |

## Summary

Complete JSON RESTful API for programmatic administration of the mail server.
Replaces optional PostfixAdmin PHP XMLRPC with a modern, documented and versioned API.

## Principles

- **Versioning** : `/api/v1/` prefix for all routes
- **Format** : JSON (Content-Type: application/json)
- **Authentication** : Bearer token (JWT) or API key
- **Pagination** : Offset-based with headers `X-Total-Count`, `Link`
- **Errors** : RFC 7807 format (Problem Details)
- **Documentation** : Auto-generated OpenAPI 3.1

## API Authentication

### JWT (temporary sessions)
```
POST /api/v1/auth/login
Body: { "username": "admin@example.com", "password": "...", "totp": "123456" }
Response: { "token": "eyJ...", "expires_at": "2024-...", "refresh_token": "..." }
```

- JWT token has a short lifespan (15 min, configurable)
- Refresh token has a long lifespan (7 days)
- TOTP is required if 2FA is enabled

### API Key (permanent access)
- Generated from the admin interface
- Sent via header: `Authorization: Bearer par_key_xxxx`
- Associated with an admin with identical permissions
- Revocable at any time

## Response Format

### Success (single object)
```json
{
  "data": {
    "domain": "example.com",
    "description": "Example Domain",
    "active": true
  }
}
```

### Success (collection)
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

### Error (RFC 7807)
```json
{
  "type": "https://postfix-admin-rs.dev/errors/validation",
  "title": "Validation Error",
  "status": 422,
  "detail": "The domain 'invalid' is not valid",
  "errors": [
    { "field": "domain", "message": "Invalid domain format" }
  ]
}
```

## HTTP Status Codes Used

| Code  | Usage                                      |
|-------|--------------------------------------------|
| `200` | Success (GET, PUT, PATCH)                  |
| `201` | Successful creation (POST)                 |
| `204` | Successful deletion (DELETE)               |
| `400` | Malformed request                          |
| `401` | Unauthenticated                            |
| `403` | Forbidden (insufficient permissions)       |
| `404` | Resource not found                         |
| `409` | Conflict (duplicate, constraint violation) |
| `422` | Validation error                           |
| `429` | Rate limit exceeded                        |
| `500` | Server error                               |

## Pagination

### Request
```
GET /api/v1/domains?page=2&per_page=20&sort=domain&order=asc
```

### Common Parameters

| Parameter  | Type    | Default | Description               |
|------------|---------|---------|---------------------------|
| `page`     | integer | 1       | Current page              |
| `per_page` | integer | 20      | Items per page (max: 100) |
| `sort`     | string  | varies  | Sort field                |
| `order`    | string  | `asc`   | Order (`asc`, `desc`)     |
| `search`   | string  | —       | Text search               |
| `active`   | boolean | —       | Filter by active status   |

### Response Headers
```
X-Total-Count: 42
Link: </api/v1/domains?page=3&per_page=20>; rel="next",
      </api/v1/domains?page=1&per_page=20>; rel="prev"
```

## Rate Limiting

- Default: 100 requests / minute per token
- Response headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
- Configurable by role (superadmin may have a higher limit)

## Endpoint Catalog

### Authentication
| Method | Route                  | Description      |
|--------|------------------------|------------------|
| `POST` | `/api/v1/auth/login`   | Obtain JWT       |
| `POST` | `/api/v1/auth/refresh` | Refresh JWT      |
| `POST` | `/api/v1/auth/logout`  | Invalidate token |

### Domains
| Method   | Route                             | Description   |
|----------|-----------------------------------|---------------|
| `GET`    | `/api/v1/domains`                 | List          |
| `GET`    | `/api/v1/domains/{domain}`        | Details       |
| `POST`   | `/api/v1/domains`                 | Create        |
| `PUT`    | `/api/v1/domains/{domain}`        | Modify        |
| `DELETE` | `/api/v1/domains/{domain}`        | Delete        |
| `PATCH`  | `/api/v1/domains/{domain}/active` | Toggle active |

### Alias Domains
| Method   | Route                           | Description |
|----------|---------------------------------|-------------|
| `GET`    | `/api/v1/alias-domains`         | List        |
| `POST`   | `/api/v1/alias-domains`         | Create      |
| `DELETE` | `/api/v1/alias-domains/{alias}` | Delete      |

### Mailboxes
| Method   | Route                                   | Description     |
|----------|-----------------------------------------|-----------------|
| `GET`    | `/api/v1/domains/{domain}/mailboxes`    | List by domain  |
| `GET`    | `/api/v1/mailboxes/{username}`          | Details         |
| `POST`   | `/api/v1/domains/{domain}/mailboxes`    | Create          |
| `PUT`    | `/api/v1/mailboxes/{username}`          | Modify          |
| `DELETE` | `/api/v1/mailboxes/{username}`          | Delete          |
| `POST`   | `/api/v1/mailboxes/{username}/password` | Change password |

### Aliases
| Method   | Route                              | Description    |
|----------|------------------------------------|----------------|
| `GET`    | `/api/v1/domains/{domain}/aliases` | List by domain |
| `GET`    | `/api/v1/aliases/{address}`        | Details        |
| `POST`   | `/api/v1/domains/{domain}/aliases` | Create         |
| `PUT`    | `/api/v1/aliases/{address}`        | Modify         |
| `DELETE` | `/api/v1/aliases/{address}`        | Delete         |

### Admins
| Method   | Route                       | Description |
|----------|-----------------------------|-------------|
| `GET`    | `/api/v1/admins`            | List        |
| `GET`    | `/api/v1/admins/{username}` | Details     |
| `POST`   | `/api/v1/admins`            | Create      |
| `PUT`    | `/api/v1/admins/{username}` | Modify      |
| `DELETE` | `/api/v1/admins/{username}` | Delete      |

### Vacation, Fetchmail, DKIM, Logs
See respective specifications for detailed endpoints.

## OpenAPI Documentation

- Endpoint: `GET /api/v1/openapi.json`
- Integrated Swagger UI interface: `GET /api/docs`
- Automatic generation from Rust types via `utoipa`

---

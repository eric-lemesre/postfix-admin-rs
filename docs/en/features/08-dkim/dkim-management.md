> **Language:** English | [Francais](../fr/features/08-dkim/dkim-management.md)

---
# SPEC-08.1 — DKIM Management

## Implementation Status

| Component                                       | Crate                  | Status  | Milestone |
|-------------------------------------------------|------------------------|---------|-----------|
| Models (`DkimKey`, `DkimSigning`)               | `postfix-admin-core`   | Done    | M1        |
| DTOs (`CreateDkimKey`, `DkimKeyResponse`, etc.) | `postfix-admin-core`   | Done    | M1        |
| Repository trait (`DkimRepository`)             | `postfix-admin-core`   | Done    | M1        |
| PostgreSQL repository                           | `postfix-admin-db`     | Pending | M2        |
| MySQL repository                                | `postfix-admin-db`     | Pending | M2        |
| RSA key generation                              | `postfix-admin-server` | Pending | M11       |
| Private key encryption at rest                  | `postfix-admin-auth`   | Pending | M11       |
| REST API endpoints                              | `postfix-admin-api`    | Pending | M6        |
| Web UI pages                                    | `postfix-admin-web`    | Pending | M5        |

## Summary

Management of DKIM (DomainKeys Identified Mail) keys and signature tables
for integration with OpenDKIM. Allows generating, storing, and managing the
cryptographic key pairs used to sign outgoing mail.

## Entity: `DkimKey`

| Field         | Type           | Constraint                             | Description                          |
|---------------|----------------|----------------------------------------|--------------------------------------|
| `id`          | `SERIAL`       | PK                                     | Auto-incremented identifier          |
| `domain_name` | `VARCHAR(255)` | FK → `domain.domain` ON DELETE CASCADE | Domain                               |
| `description` | `VARCHAR(255)` | default `''`                           | Key description                      |
| `selector`    | `VARCHAR(63)`  | NOT NULL, default `'default'`          | DKIM selector                        |
| `private_key` | `TEXT`         | NOT NULL                               | Private key (PEM, encrypted at rest) |
| `public_key`  | `TEXT`         | NOT NULL                               | Public key (PEM)                     |
| `created_at`  | `TIMESTAMPTZ`  | NOT NULL, default `now()`              | Creation date                        |
| `updated_at`  | `TIMESTAMPTZ`  | NOT NULL, default `now()`              | Last update                          |

Index: `(domain_name, description)`

## Entity: `DkimSigning`

| Field        | Type           | Constraint                           | Description                            |
|--------------|----------------|--------------------------------------|----------------------------------------|
| `id`         | `SERIAL`       | PK                                   | Auto-incremented identifier            |
| `author`     | `VARCHAR(255)` | NOT NULL                             | Author pattern (e.g., `*@example.com`) |
| `dkim_id`    | `INTEGER`      | FK → `dkim_key.id` ON DELETE CASCADE | DKIM key to use                        |
| `created_at` | `TIMESTAMPTZ`  | NOT NULL, default `now()`            | Creation date                          |
| `updated_at` | `TIMESTAMPTZ`  | NOT NULL, default `now()`            | Last update                            |

Index: `(author)`

## Business Rules

### BR-DKIM-01: Key Generation
- Algorithm: RSA 2048 bits (configurable: 1024, 2048, 4096)
- The selector must be unique per domain
- Private key is encrypted at rest (AES-256-GCM)
- Public key is displayed in DNS record format

### BR-DKIM-02: Signing Table
- The `author` pattern supports wildcards: `*@example.com`, `user@example.com`
- A domain can have multiple keys (rotation)
- Only one signing entry is active per pattern

### BR-DKIM-03: OpenDKIM Integration
- Possible export in OpenDKIM-compatible format:
  - `KeyTable`: `selector._domainkey.example.com example.com:selector:/path/to/key`
  - `SigningTable`: `*@example.com selector._domainkey.example.com`
- API endpoint for OpenDKIM to query keys dynamically

### BR-DKIM-04: DNS Record
- Display of the TXT record to create:
  ```
  selector._domainkey.example.com. IN TXT "v=DKIM1; k=rsa; p=<base64_public_key>"
  ```
- Optional DNS verification: the app can verify that the record is in place

## Use Cases

### UC-DKIM-01: Generate a DKIM Key
- **Actor**: Superadmin, Domain Admin
- **Input**: Domain, selector, key size
- **Output**: Generated key pair, DNS record to create

### UC-DKIM-02: Configure Signing
- **Actor**: Superadmin, Domain Admin
- **Input**: Author pattern, DKIM key to use
- **Output**: Created signing rule

### UC-DKIM-03: Key Rotation
- **Actor**: Superadmin, Domain Admin
- **Process**:
  1. Generate a new key with a new selector
  2. Publish the new DNS record
  3. Wait for DNS propagation
  4. Update the signing table
  5. Delete the old key (optional)

### UC-DKIM-04: Verify DNS Configuration
- **Actor**: Superadmin, Domain Admin
- **Input**: Domain + selector
- **Output**: Status of DKIM DNS record

## API Endpoints

| Method   | Route                                     | Description        |
|----------|-------------------------------------------|--------------------|
| `GET`    | `/api/v1/domains/{domain}/dkim/keys`      | List keys          |
| `POST`   | `/api/v1/domains/{domain}/dkim/keys`      | Generate a key     |
| `DELETE` | `/api/v1/dkim/keys/{id}`                  | Delete a key       |
| `GET`    | `/api/v1/domains/{domain}/dkim/signing`   | List signing rules |
| `POST`   | `/api/v1/domains/{domain}/dkim/signing`   | Create a rule      |
| `DELETE` | `/api/v1/dkim/signing/{id}`               | Delete a rule      |
| `GET`    | `/api/v1/domains/{domain}/dkim/dns-check` | Check DNS          |

---

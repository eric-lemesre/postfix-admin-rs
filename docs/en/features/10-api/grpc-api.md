> **Language:** English | [Francais](../fr/features/10-api/grpc-api.md)

---
# SPEC-10.2 — gRPC API

## Implementation Status

| Component                     | Crate                | Status  | Milestone |
|-------------------------------|----------------------|---------|-----------|
| Protobuf definitions          | `postfix-admin-api`  | Pending | M7        |
| tonic service implementations | `postfix-admin-api`  | Pending | M7        |
| Authentication interceptor    | `postfix-admin-auth` | Pending | M7        |
| Integration tests             | `postfix-admin-api`  | Pending | M7        |

## Summary

High-performance gRPC API for inter-service integrations and automation.
Complementary to the REST API, it is optimized for service-to-service communications,
advanced scripting, and integrations with provisioning systems.

## Technical stack

- **Framework** : tonic (Rust gRPC)
- **Serialization** : Protocol Buffers v3 (prost)
- **Transport** : HTTP/2 with optional TLS
- **Port** : Configurable, separate from the HTTP port (default: 50051)

## Service definitions (Proto)

### MailService

```protobuf
syntax = "proto3";
package postfixadmin.v1;

service DomainService {
  rpc ListDomains (ListDomainsRequest) returns (ListDomainsResponse);
  rpc GetDomain (GetDomainRequest) returns (Domain);
  rpc CreateDomain (CreateDomainRequest) returns (Domain);
  rpc UpdateDomain (UpdateDomainRequest) returns (Domain);
  rpc DeleteDomain (DeleteDomainRequest) returns (google.protobuf.Empty);
}

service MailboxService {
  rpc ListMailboxes (ListMailboxesRequest) returns (ListMailboxesResponse);
  rpc GetMailbox (GetMailboxRequest) returns (Mailbox);
  rpc CreateMailbox (CreateMailboxRequest) returns (Mailbox);
  rpc UpdateMailbox (UpdateMailboxRequest) returns (Mailbox);
  rpc DeleteMailbox (DeleteMailboxRequest) returns (google.protobuf.Empty);
  rpc ChangePassword (ChangePasswordRequest) returns (google.protobuf.Empty);
}

service AliasService {
  rpc ListAliases (ListAliasesRequest) returns (ListAliasesResponse);
  rpc GetAlias (GetAliasRequest) returns (Alias);
  rpc CreateAlias (CreateAliasRequest) returns (Alias);
  rpc UpdateAlias (UpdateAliasRequest) returns (Alias);
  rpc DeleteAlias (DeleteAliasRequest) returns (google.protobuf.Empty);
}

service AdminService {
  rpc Authenticate (AuthRequest) returns (AuthResponse);
  rpc ListAdmins (ListAdminsRequest) returns (ListAdminsResponse);
  rpc CreateAdmin (CreateAdminRequest) returns (Admin);
  rpc DeleteAdmin (DeleteAdminRequest) returns (google.protobuf.Empty);
}
```

## gRPC Authentication

- Metadata header : `authorization: Bearer <jwt_or_api_key>`
- Global authentication interceptor
- Same tokens as the REST API (JWT or API Key)

## gRPC Advantages

| Aspect         | REST           | gRPC                    |
|----------------|----------------|-------------------------|
| Serialization  | JSON (text)    | Protobuf (binary)       |
| Transport      | HTTP/1.1 or 2  | HTTP/2 mandatory        |
| Streaming      | Not native     | Bidirectional           |
| Typing         | OpenAPI schema | Strict proto            |
| Performance    | Good           | Excellent               |
| Client tooling | curl, browser  | grpcurl, generated code |

## Priority use cases

1. **Automated provisioning** : Scripts for bulk creation of mailboxes/domains
2. **ISP integration** : Hosting panels (cPanel, Plesk, custom)
3. **Monitoring** : Fast collection of quota metrics
4. **Migration** : High-performance import/export of large data sets

## Configuration

```toml
[grpc]
enabled = true
bind_address = "0.0.0.0"
port = 50051
tls_enabled = false
tls_cert_path = ""
tls_key_path = ""
max_message_size_mb = 4
```

## Reflection

The gRPC server exposes the reflection service to enable dynamic discovery of services (compatible with `grpcurl` and testing tools).

---

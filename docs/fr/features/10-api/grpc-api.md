> **Language:** [English](../en/features/10-api/grpc-api.md) | Francais

# SPEC-10.2 — API gRPC

## Résumé

API gRPC haute performance pour les intégrations inter-services et l'automatisation.
Complémentaire à l'API REST, elle est optimisée pour les communications service-to-service,
le scripting avancé et les intégrations avec des systèmes de provisionnement.

## Stack technique

- **Framework** : tonic (Rust gRPC)
- **Sérialisation** : Protocol Buffers v3 (prost)
- **Transport** : HTTP/2 avec TLS optionnel
- **Port** : Configurable, séparé du port HTTP (par défaut : 50051)

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

## Authentification gRPC

- Metadata header : `authorization: Bearer <jwt_or_api_key>`
- Intercepteur d'authentification global
- Mêmes tokens que l'API REST (JWT ou API Key)

## Avantages du gRPC

| Aspect | REST | gRPC |
|--------|------|------|
| Sérialisation | JSON (texte) | Protobuf (binaire) |
| Transport | HTTP/1.1 ou 2 | HTTP/2 obligatoire |
| Streaming | Non natif | Bidirectionnel |
| Typage | Schéma OpenAPI | Proto strict |
| Performance | Bon | Excellent |
| Tooling client | curl, navigateur | grpcurl, code généré |

## Cas d'usage prioritaires

1. **Provisionnement automatisé** : Scripts de création en masse de boîtes/domaines
2. **Intégration ISP** : Panels d'hébergement (cPanel, Plesk, custom)
3. **Monitoring** : Collecte rapide des métriques de quotas
4. **Migration** : Import/export performant de grandes quantités de données

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

Le serveur gRPC expose le service de réflexion pour permettre la découverte
dynamique des services (compatible avec `grpcurl` et les outils de test).

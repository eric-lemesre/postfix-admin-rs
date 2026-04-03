> **Language:** English | [Francais](../../fr/deployment/SECURITY-HARDENING.md)

---

# Security hardening guide — Internet-facing deployment

This guide covers all security measures required when exposing postfix-admin-rs
to the public internet. It complements the [deployment guide](DEPLOYMENT.md)
with hardening configurations.

> **Important:** postfix-admin-rs manages email infrastructure. A compromise
> gives an attacker full control over mail routing, aliases, and mailboxes.
> Treat it as a critical service.

---

## 1. TLS/SSL hardening

Enforce TLS 1.2+ with strong cipher suites and Perfect Forward Secrecy (PFS).

### Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name mail-admin.example.com;

    # TLS 1.2+ only — disable SSLv3, TLSv1.0, TLSv1.1
    ssl_protocols TLSv1.2 TLSv1.3;

    # Strong cipher suites with PFS (ECDHE) — no RSA key exchange
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305';
    ssl_prefer_server_ciphers on;

    # ECDH curve
    ssl_ecdh_curve X25519:secp384r1;

    # OCSP stapling — reduces certificate verification latency
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/mail-admin.example.com/chain.pem;
    resolver 1.1.1.1 8.8.8.8 valid=300s;
    resolver_timeout 5s;

    # Session resumption
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    ssl_session_tickets off;

    # Certificates
    ssl_certificate /etc/letsencrypt/live/mail-admin.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mail-admin.example.com/privkey.pem;

    # DH parameters (generate with: openssl dhparam -out /etc/ssl/dhparam.pem 4096)
    ssl_dhparam /etc/ssl/dhparam.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name mail-admin.example.com;
    return 301 https://$host$request_uri;
}
```

### Apache

```apache
<VirtualHost *:443>
    ServerName mail-admin.example.com

    SSLEngine on
    SSLProtocol all -SSLv3 -TLSv1 -TLSv1.1
    SSLCipherSuite ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305
    SSLHonorCipherOrder on

    SSLUseStapling on
    SSLStaplingResponderTimeout 5
    SSLStaplingReturnResponderErrors off

    SSLCertificateFile /etc/letsencrypt/live/mail-admin.example.com/fullchain.pem
    SSLCertificateKeyFile /etc/letsencrypt/live/mail-admin.example.com/privkey.pem

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
    RequestHeader set X-Forwarded-Proto "https"
</VirtualHost>

# OCSP stapling cache (place in global config)
SSLStaplingCache shmcb:/var/run/ocsp(128000)

# Redirect HTTP to HTTPS
<VirtualHost *:80>
    ServerName mail-admin.example.com
    Redirect permanent / https://mail-admin.example.com/
</VirtualHost>
```

### Validation

```bash
# Test your TLS configuration
openssl s_client -connect mail-admin.example.com:443 -tls1_2
openssl s_client -connect mail-admin.example.com:443 -tls1_3

# Verify OCSP stapling
openssl s_client -connect mail-admin.example.com:443 -status

# Online test (recommended)
# https://www.ssllabs.com/ssltest/ — aim for A+ rating
```

---

## 2. HTTP security headers

### Nginx (add inside the `server` block)

```nginx
# Content Security Policy — restrict resource origins
add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'" always;

# HSTS — force HTTPS for 1 year, include subdomains, allow preload list
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;

# Prevent clickjacking
add_header X-Frame-Options "DENY" always;

# Prevent MIME-type sniffing
add_header X-Content-Type-Options "nosniff" always;

# Control referrer information
add_header Referrer-Policy "strict-origin-when-cross-origin" always;

# Restrict browser features
add_header Permissions-Policy "camera=(), microphone=(), geolocation=(), payment=()" always;
```

### Application configuration (config.toml)

```toml
[security.headers]
csp_enabled = true
hsts_enabled = true
hsts_max_age = 31536000
```

---

## 3. CORS policy for the REST API

Only allow requests from known origins. Never use `*` in production.

### Application configuration (config.toml)

```toml
[api.cors]
allowed_origins = ["https://mail-admin.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]
allowed_headers = ["Content-Type", "Authorization", "X-CSRF-Token"]
allow_credentials = true
max_age = 3600
```

If the API is consumed by external clients, add their specific origins:

```toml
allowed_origins = [
    "https://mail-admin.example.com",
    "https://monitoring.example.com",
]
```

---

## 4. DDoS mitigation

### Nginx rate limiting

```nginx
# Define rate limit zones (place in http block)
limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;
limit_req_zone $binary_remote_addr zone=api:10m rate=30r/s;
limit_req_zone $binary_remote_addr zone=web:10m rate=10r/s;
limit_conn_zone $binary_remote_addr zone=addr:10m;

server {
    # Global connection limit per IP
    limit_conn addr 20;

    # Web interface — 10 req/s with burst
    location / {
        limit_req zone=web burst=20 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }

    # Login endpoint — strict rate limit (5 req/min)
    location /login {
        limit_req zone=login burst=3 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }

    # API — 30 req/s with burst
    location /api/ {
        limit_req zone=api burst=50 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }

    # Request body size limit
    client_max_body_size 1m;

    # Timeouts
    proxy_connect_timeout 5s;
    proxy_read_timeout 30s;
    proxy_send_timeout 10s;
}
```

### nftables rate limiting

```bash
#!/usr/sbin/nft -f
table inet filter {
    chain input {
        type filter hook input priority 0; policy drop;

        # Allow established connections
        ct state established,related accept

        # Allow loopback
        iif lo accept

        # Rate limit new HTTPS connections: 25/second per IP
        tcp dport 443 ct state new limit rate over 25/second drop
        tcp dport 443 ct state new accept

        # SSH (admin only — restrict to your IP)
        tcp dport 22 ip saddr 203.0.113.0/24 accept

        # Drop everything else
        drop
    }
}
```

---

## 5. Firewall rules and network segmentation

### Principle: minimal exposure

- The application listens on `127.0.0.1:8080` (localhost only)
- Only the reverse proxy (Nginx/Apache) is exposed on ports 80/443
- The database is only accessible from the application server
- gRPC port (if used) is not exposed to the internet

### nftables production configuration

```bash
#!/usr/sbin/nft -f
flush ruleset

table inet filter {
    chain input {
        type filter hook input priority 0; policy drop;

        ct state established,related accept
        iif lo accept

        # HTTPS (public)
        tcp dport 443 accept

        # HTTP (redirect only)
        tcp dport 80 accept

        # SSH (admin IP only)
        tcp dport 22 ip saddr { 203.0.113.10, 203.0.113.11 } accept

        # ICMP (ping)
        ip protocol icmp accept
        ip6 nexthdr icmpv6 accept

        # Log and drop everything else
        log prefix "nft-drop: " drop
    }

    chain forward {
        type filter hook forward priority 0; policy drop;
    }

    chain output {
        type filter hook output priority 0; policy accept;
    }
}
```

### Database isolation

If the database runs on a separate host:

```bash
# On the database server — only accept connections from the app server
nft add rule inet filter input ip saddr 10.0.1.10 tcp dport 5432 accept
nft add rule inet filter input tcp dport 5432 drop
```

If using Docker Compose, use an internal network:

```yaml
services:
  app:
    networks:
      - frontend
      - backend

  db:
    networks:
      - backend   # Not exposed to frontend

networks:
  frontend:
  backend:
    internal: true   # No external access
```

---

## 6. WAF (Web Application Firewall)

### ModSecurity with OWASP Core Rule Set (Nginx)

```bash
# Install (Debian/Ubuntu)
sudo apt install libmodsecurity3 libnginx-mod-http-modsecurity

# Enable OWASP CRS
sudo git clone https://github.com/coreruleset/coreruleset /etc/modsecurity/crs
sudo cp /etc/modsecurity/crs/crs-setup.conf.example /etc/modsecurity/crs/crs-setup.conf
```

Nginx configuration:

```nginx
server {
    modsecurity on;
    modsecurity_rules_file /etc/modsecurity/main.conf;

    # ...
}
```

`/etc/modsecurity/main.conf`:

```
Include /etc/modsecurity/modsecurity.conf
Include /etc/modsecurity/crs/crs-setup.conf
Include /etc/modsecurity/crs/rules/*.conf

# Application-specific exclusions (if needed)
SecRuleRemoveById 920350   # Example: exclude a rule causing false positives
```

### Alternative: Cloudflare or AWS WAF

For managed WAF solutions, place the service in front of your reverse proxy
and configure origin IP protection to prevent bypass.

---

## 7. Secrets management

### Generating strong secrets

```bash
# Session secret key (64 bytes, base64 encoded)
openssl rand -base64 64

# Database password
openssl rand -base64 32

# JWT signing key
openssl rand -base64 48

# CSRF master key
openssl rand -base64 32
```

### Environment variables (recommended)

Never store secrets in `config.toml`. Use environment variables:

```bash
# /etc/postfix-admin-rs/env (chmod 600, owned by root)
PAR_DATABASE__URL="postgresql://postfix:STRONG_PASSWORD_HERE@127.0.0.1:5432/postfix"
PAR_SERVER__SECRET_KEY="generated-64-byte-base64-key"
PAR_AUTH__JWT_SECRET="generated-48-byte-base64-key"
PAR_AUTH__MASTER_KEY="generated-32-byte-base64-key"
```

Systemd integration:

```ini
[Service]
EnvironmentFile=/etc/postfix-admin-rs/env
```

### Rules

- **Never** commit secrets to git (add patterns to `.gitignore`)
- **Never** log secrets (the application enforces this)
- **Rotate** secrets periodically (at least annually)
- **Use different** secrets for each environment (dev, staging, production)

### HashiCorp Vault integration pattern

For organizations using Vault:

```bash
# Store secrets
vault kv put secret/postfix-admin-rs \
    database_url="postgresql://..." \
    secret_key="..." \
    jwt_secret="..."

# Retrieve at startup (wrapper script)
#!/bin/bash
export PAR_DATABASE__URL=$(vault kv get -field=database_url secret/postfix-admin-rs)
export PAR_SERVER__SECRET_KEY=$(vault kv get -field=secret_key secret/postfix-admin-rs)
export PAR_AUTH__JWT_SECRET=$(vault kv get -field=jwt_secret secret/postfix-admin-rs)
exec /usr/bin/postfix-admin-rs serve
```

---

## 8. Certificate management

### Let's Encrypt with certbot

```bash
# Install
sudo apt install certbot

# Obtain certificate (webroot method)
sudo certbot certonly --webroot \
    -w /var/www/html \
    -d mail-admin.example.com \
    --email admin@example.com \
    --agree-tos \
    --no-eff-email

# Or with Nginx plugin
sudo certbot --nginx -d mail-admin.example.com
```

### Auto-renewal

```bash
# Test renewal
sudo certbot renew --dry-run

# Systemd timer (usually installed by certbot)
sudo systemctl enable --now certbot.timer
```

Override the renewal hook to reload the reverse proxy:

```bash
# /etc/letsencrypt/renewal-hooks/deploy/reload-nginx.sh
#!/bin/bash
systemctl reload nginx
```

### Certificate monitoring

```bash
# Cron job to alert on expiring certificates (14 days before)
0 8 * * * /usr/bin/openssl s_client -connect mail-admin.example.com:443 -servername mail-admin.example.com 2>/dev/null | openssl x509 -noout -checkend 1209600 || echo "Certificate expiring soon" | mail -s "ALERT: TLS certificate" admin@example.com
```

---

## 9. Client certificate authentication (mTLS) for administrators

Client certificates provide a strong additional authentication factor for privileged accounts. The reverse proxy handles TLS client certificate verification and forwards identity information to the application via HTTP headers.

### Generating a CA and client certificates

```bash
# Create a CA for admin client certificates
openssl genrsa -out admin-ca.key 4096
openssl req -new -x509 -days 3650 -key admin-ca.key \
    -out admin-ca.crt -subj "/CN=PostfixAdmin Admin CA/O=Example Inc"

# Generate a client certificate for an administrator
openssl genrsa -out admin-client.key 2048
openssl req -new -key admin-client.key \
    -out admin-client.csr -subj "/emailAddress=admin@example.com/CN=Admin User/O=Example Inc"
openssl x509 -req -days 365 -in admin-client.csr \
    -CA admin-ca.crt -CAkey admin-ca.key -CAcreateserial \
    -out admin-client.crt

# Create PKCS#12 bundle for browser import
openssl pkcs12 -export -out admin-client.p12 \
    -inkey admin-client.key -in admin-client.crt -certfile admin-ca.crt
```

### Nginx configuration

```nginx
server {
    listen 443 ssl http2;
    server_name mail-admin.example.com;

    # ... existing TLS config ...

    # Client certificate verification
    ssl_client_certificate /etc/ssl/admin-ca.crt;
    ssl_verify_client optional;    # optional = don't require for all users

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Forward client certificate info to the application
        proxy_set_header X-SSL-Client-Verify $ssl_client_verify;
        proxy_set_header X-SSL-Client-S-DN $ssl_client_s_dn;
        proxy_set_header X-SSL-Client-Serial $ssl_client_serial;
    }
}
```

### Apache configuration

```apache
<VirtualHost *:443>
    ServerName mail-admin.example.com

    # ... existing TLS config ...

    # Client certificate verification
    SSLCACertificateFile /etc/ssl/admin-ca.crt
    SSLVerifyClient optional
    SSLVerifyDepth 2

    # Forward client certificate info
    RequestHeader set X-SSL-Client-Verify "%{SSL_CLIENT_VERIFY}s"
    RequestHeader set X-SSL-Client-S-DN "%{SSL_CLIENT_S_DN}s"
    RequestHeader set X-SSL-Client-Serial "%{SSL_CLIENT_M_SERIAL}s"

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
</VirtualHost>
```

### Application configuration

```toml
[auth.mtls]
enabled = true
trusted_proxy_header = "X-SSL-Client-Verify"
subject_header = "X-SSL-Client-S-DN"
serial_header = "X-SSL-Client-Serial"
require_for_superadmin = true
require_for_domain_admin = false
cn_field = "emailAddress"
```

### Security considerations

- **Header spoofing:** The reverse proxy MUST strip any client-set `X-SSL-Client-*` headers before forwarding. Nginx does this automatically for `ssl_client_*` variables; Apache requires `RequestHeader unset` directives for untrusted requests.
- **Certificate revocation:** Use CRL or OCSP to handle revoked certificates. Configure `ssl_crl` (Nginx) or `SSLCARevocationFile` (Apache).
- **Certificate validity:** Client certificates should have a short validity period (90-365 days) and be rotated regularly.
- **CA security:** The admin CA private key must be stored offline or in an HSM. Never store it on the web server.

---

## 10. gRPC security

If the gRPC API is enabled, apply these measures:

### Mutual TLS (mTLS)

```toml
# config.toml
[grpc]
enabled = true
bind = "127.0.0.1:50051"       # Localhost only — never expose directly
tls_enabled = true
tls_cert_path = "/etc/postfix-admin-rs/grpc-server.crt"
tls_key_path = "/etc/postfix-admin-rs/grpc-server.key"
tls_ca_cert_path = "/etc/postfix-admin-rs/ca.crt"   # Client CA for mTLS
require_client_cert = true
```

### Disable reflection in production

gRPC reflection exposes your API schema. Disable it in production:

```toml
[grpc]
reflection_enabled = false    # Only enable in development
```

### Network isolation

The gRPC port should **never** be exposed to the internet. If remote access is
needed, use a VPN or SSH tunnel:

```bash
# SSH tunnel for remote gRPC access
ssh -L 50051:127.0.0.1:50051 admin@mail-admin.example.com
```

---

## 11. Security monitoring and logging

### Audit logging

```toml
# config.toml
[logging]
level = "info"
format = "json"                    # Structured logs for aggregation
audit_enabled = true
audit_retention_days = 365

[logging.audit]
log_authentication = true          # Login attempts (success + failure)
log_authorization = true           # Permission denials
log_data_changes = true            # Domain/mailbox/alias CRUD operations
log_admin_actions = true           # Admin-level operations
```

### fail2ban integration

```ini
# /etc/fail2ban/filter.d/postfix-admin-rs.conf
[Definition]
failregex = ^.*authentication failed.*client_ip="<HOST>".*$
            ^.*rate limit exceeded.*client_ip="<HOST>".*$
            ^.*brute force.*client_ip="<HOST>".*$
ignoreregex =
```

```ini
# /etc/fail2ban/jail.d/postfix-admin-rs.conf
[postfix-admin-rs]
enabled = true
filter = postfix-admin-rs
logpath = /var/log/postfix-admin-rs/audit.log
maxretry = 5
findtime = 900
bantime = 3600
action = nftables-multiport[name=postfix-admin-rs, port="80,443"]
```

### Log aggregation

For production environments, forward structured logs to a centralized system:

- **ELK Stack** (Elasticsearch + Logstash + Kibana) — use Filebeat as shipper
- **Loki + Grafana** — lightweight alternative with LogQL
- **Graylog** — centralized log management with alerting

Filebeat configuration example:

```yaml
# /etc/filebeat/conf.d/postfix-admin-rs.yml
filebeat.inputs:
  - type: log
    paths:
      - /var/log/postfix-admin-rs/*.log
    json.keys_under_root: true
    json.add_error_key: true

output.elasticsearch:
  hosts: ["https://elk.internal:9200"]
  index: "postfix-admin-rs-%{+yyyy.MM.dd}"
```

### Alerting rules

Set up alerts for:

- More than 10 failed login attempts from the same IP in 5 minutes
- Any successful login from an unknown IP
- Configuration changes
- Database connection failures
- Certificate expiration within 14 days
- Application restarts

---

## 12. Data protection and GDPR

### Encryption at rest

#### Database

```sql
-- PostgreSQL: enable pgcrypto extension
CREATE EXTENSION IF NOT EXISTS pgcrypto;
```

Use full-disk encryption (LUKS) on the database volume:

```bash
# Encrypt the database partition
sudo cryptsetup luksFormat /dev/sdb1
sudo cryptsetup open /dev/sdb1 pgdata
sudo mkfs.ext4 /dev/mapper/pgdata
sudo mount /dev/mapper/pgdata /var/lib/postgresql
```

#### Backup encryption

```bash
# Encrypted backup with GPG
pg_dump -U postfix postfix | gpg --symmetric --cipher-algo AES256 \
    --output /backups/postfix-$(date +%Y%m%d).sql.gpg

# Restore
gpg --decrypt /backups/postfix-20250101.sql.gpg | psql -U postfix postfix
```

### Data retention

```toml
# config.toml
[data_retention]
audit_log_days = 365               # Keep audit logs for 1 year
session_data_days = 30             # Purge expired sessions after 30 days
deleted_mailbox_days = 30          # Grace period before permanent deletion
```

### Right to erasure (GDPR Art. 17)

The CLI provides commands for data erasure:

```bash
# Delete all data associated with a user
postfix-admin-rs user purge --email user@example.com --confirm

# Export user data (GDPR Art. 15 — right of access)
postfix-admin-rs user export --email user@example.com --format json
```

### Privacy considerations

- Log files must not contain email message content
- IP addresses in logs are considered personal data under GDPR
- Implement log rotation with automatic deletion after the retention period
- Document your data processing activities (GDPR Art. 30)

---

## 13. Production deployment checklist

### TLS/SSL
- [ ] TLS 1.2+ enforced, TLS 1.0/1.1 disabled
- [ ] Strong cipher suites with Perfect Forward Secrecy
- [ ] OCSP stapling enabled
- [ ] HTTP to HTTPS redirect in place
- [ ] SSL Labs test score: A+

### HTTP security
- [ ] All security headers configured (CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)
- [ ] HSTS preload list submission (if applicable)
- [ ] CORS policy restricts origins to known domains

### Network
- [ ] Application listens on localhost only (127.0.0.1)
- [ ] Only ports 80/443 exposed to the internet
- [ ] Database port not accessible from the internet
- [ ] gRPC port not accessible from the internet
- [ ] Firewall rules applied and tested
- [ ] DDoS rate limiting configured

### Authentication
- [ ] Default credentials changed
- [ ] Strong password policy enforced
- [ ] Rate limiting on login endpoints
- [ ] 2FA (TOTP) enabled for admin accounts
- [ ] Client certificates (mTLS) enabled for superadmin accounts
- [ ] Session cookies: HttpOnly, Secure, SameSite=Strict

### Secrets
- [ ] All secrets generated with cryptographic randomness
- [ ] Secrets stored in environment variables (not in config files)
- [ ] No secrets committed to git
- [ ] Different secrets for each environment

### Certificates
- [ ] Let's Encrypt (or equivalent) certificates in use
- [ ] Auto-renewal configured and tested
- [ ] Certificate expiration monitoring in place

### Monitoring
- [ ] Audit logging enabled
- [ ] fail2ban (or equivalent) configured
- [ ] Log aggregation set up
- [ ] Alerting rules defined
- [ ] Health check endpoint monitored

### System hardening
- [ ] Systemd service with security options (NoNewPrivileges, ProtectSystem, etc.)
- [ ] Application runs as unprivileged user
- [ ] File permissions: config `600`, binary `755`
- [ ] WAF enabled (ModSecurity or managed solution)
- [ ] `cargo audit` run — no known vulnerabilities

### Client certificates (mTLS)
- [ ] Admin CA generated and stored securely (offline/HSM)
- [ ] Client certificates issued to all admin accounts
- [ ] Reverse proxy configured to verify client certs
- [ ] X-SSL-Client-* headers cannot be spoofed by clients
- [ ] Certificate revocation mechanism in place (CRL/OCSP)

### Data protection
- [ ] Database encryption at rest (LUKS or equivalent)
- [ ] Backup encryption configured
- [ ] Data retention policy defined
- [ ] GDPR data export/deletion commands tested
- [ ] Log rotation configured

---

See also: [Deployment guide](DEPLOYMENT.md) | [Architecture](../architecture/ARCHITECTURE.md)

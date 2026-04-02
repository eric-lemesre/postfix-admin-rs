> **Language:** [English](../en/deployment/DEPLOYMENT.md) | Francais

# Guide de deploiement — postfix-admin-rs

## Options de deploiement

### 1. Binaire statique

```bash
# Telecharger la derniere release
wget https://github.com/eric-lemesre/PostfixAdminRust/releases/latest/download/postfix-admin-rs-linux-amd64

# Installer
sudo install -m 755 postfix-admin-rs-linux-amd64 /usr/local/bin/postfix-admin-rs

# Creer la configuration
sudo mkdir -p /etc/postfix-admin-rs
sudo cp config.example.toml /etc/postfix-admin-rs/config.toml
sudo chmod 600 /etc/postfix-admin-rs/config.toml

# Setup initial
postfix-admin-rs setup

# Demarrer
postfix-admin-rs serve
```

### 2. Package Debian/Ubuntu

```bash
sudo dpkg -i postfix-admin-rs_1.0.0_amd64.deb
sudo systemctl enable --now postfix-admin-rs
```

Le package installe :
- `/usr/bin/postfix-admin-rs` — Binaire
- `/etc/postfix-admin-rs/config.toml` — Configuration
- `/lib/systemd/system/postfix-admin-rs.service` — Service systemd

### 3. Docker

```bash
docker run -d \
    --name postfix-admin-rs \
    -p 8080:8080 \
    -v /etc/postfix-admin-rs:/etc/postfix-admin-rs:ro \
    -e PAR_DATABASE__URL="postgresql://postfix:pass@db:5432/postfix" \
    ghcr.io/eric-lemesre/postfix-admin-rs:latest
```

### 4. Docker Compose

```yaml
version: '3.8'
services:
  app:
    image: ghcr.io/eric-lemesre/postfix-admin-rs:latest
    ports:
      - "8080:8080"
    environment:
      PAR_DATABASE__URL: "postgresql://postfix:password@db:5432/postfix"
      PAR_SERVER__SECRET_KEY: "change-me-in-production"
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: postfix
      POSTGRES_USER: postfix
      POSTGRES_PASSWORD: password
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postfix"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  pgdata:
```

## Service systemd

```ini
# /lib/systemd/system/postfix-admin-rs.service
[Unit]
Description=PostfixAdmin Rust - Mail Server Administration
After=network-online.target postgresql.service
Wants=network-online.target

[Service]
Type=simple
User=postfix-admin
Group=postfix-admin
ExecStart=/usr/bin/postfix-admin-rs serve
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5

# Securite
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadOnlyPaths=/etc/postfix-admin-rs
PrivateTmp=true
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictSUIDSGID=true

[Install]
WantedBy=multi-user.target
```

## Reverse proxy

### Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name mail-admin.example.com;

    ssl_certificate /etc/ssl/certs/mail-admin.pem;
    ssl_certificate_key /etc/ssl/private/mail-admin.key;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Apache

```apache
<VirtualHost *:443>
    ServerName mail-admin.example.com

    SSLEngine on
    SSLCertificateFile /etc/ssl/certs/mail-admin.pem
    SSLCertificateKeyFile /etc/ssl/private/mail-admin.key

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/

    RequestHeader set X-Forwarded-Proto "https"
</VirtualHost>
```

## Setup initial

```bash
# 1. Appliquer les migrations
postfix-admin-rs migrate

# 2. Creer le premier superadmin
postfix-admin-rs setup
# Interactif : demande username et mot de passe

# 3. Verifier
postfix-admin-rs config check
```

## Monitoring

### Health check

```
GET /health → 200 OK {"status": "healthy", "database": "connected"}
```

### Metriques (optionnel)

Si les metriques Prometheus sont activees :
```
GET /metrics → metriques au format Prometheus
```

Metriques exposees :
- `par_http_requests_total` — Requetes HTTP par methode/route/status
- `par_http_request_duration_seconds` — Duree des requetes
- `par_db_pool_connections` — Connexions au pool BDD
- `par_auth_login_total` — Tentatives de login (succes/echec)

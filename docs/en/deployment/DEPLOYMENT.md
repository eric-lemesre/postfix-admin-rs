> **Language:** English | [Francais](../fr/deployment/DEPLOYMENT.md)

---
# Deployment guide — postfix-admin-rs

> **See also:** [Security hardening guide](SECURITY-HARDENING.md) for internet-facing deployments.

## Deployment options

### 1. Static binary

```bash
# Download the latest release
wget https://github.com/eric-lemesre/postfix-admin-rs/releases/latest/download/postfix-admin-rs-linux-amd64

# Install
sudo install -m 755 postfix-admin-rs-linux-amd64 /usr/local/bin/postfix-admin-rs

# Create configuration
sudo mkdir -p /etc/postfix-admin-rs
sudo cp config.example.toml /etc/postfix-admin-rs/config.toml
sudo chmod 600 /etc/postfix-admin-rs/config.toml

# Initial setup
postfix-admin-rs setup

# Start
postfix-admin-rs serve
```

### 2. Debian/Ubuntu package

```bash
sudo dpkg -i postfix-admin-rs_1.0.0_amd64.deb
sudo systemctl enable --now postfix-admin-rs
```

The package installs:
- `/usr/bin/postfix-admin-rs` — Binary
- `/etc/postfix-admin-rs/config.toml` — Configuration
- `/lib/systemd/system/postfix-admin-rs.service` — Systemd service

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

## Systemd service

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

# Security
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

## Initial setup

```bash
# 1. Apply migrations
postfix-admin-rs migrate

# 2. Create first superadmin
postfix-admin-rs setup
# Interactive: asks for username and password

# 3. Verify
postfix-admin-rs config check
```

## Monitoring

### Health check

```
GET /health → 200 OK {"status": "healthy", "database": "connected"}
```

### Metrics (optional)

If Prometheus metrics are enabled:
```
GET /metrics → metrics in Prometheus format
```

Exposed metrics:
- `par_http_requests_total` — HTTP requests by method/route/status
- `par_http_request_duration_seconds` — Request durations
- `par_db_pool_connections` — Database pool connections
- `par_auth_login_total` — Login attempts (success/failure)

---

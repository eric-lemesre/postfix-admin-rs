> **Language:** [English](../../en/deployment/SECURITY-HARDENING.md) | Francais

---

# Guide de durcissement securite — Deploiement expose sur internet

Ce guide couvre les mesures de securite necessaires lors de l'exposition de
postfix-admin-rs sur internet. Il complete le [guide de deploiement](DEPLOYMENT.md)
avec des configurations de durcissement.

> **Important :** postfix-admin-rs gere l'infrastructure email. Une compromission
> donne a un attaquant le controle total du routage des mails, des alias et des
> boites aux lettres. Traitez-le comme un service critique.

---

## 1. Durcissement TLS/SSL

Imposer TLS 1.2+ avec des suites de chiffrement robustes et le Perfect Forward Secrecy (PFS).

### Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name mail-admin.example.com;

    # TLS 1.2+ uniquement — desactiver SSLv3, TLSv1.0, TLSv1.1
    ssl_protocols TLSv1.2 TLSv1.3;

    # Suites de chiffrement robustes avec PFS (ECDHE) — pas d'echange de cle RSA
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305';
    ssl_prefer_server_ciphers on;

    # Courbe ECDH
    ssl_ecdh_curve X25519:secp384r1;

    # Agrafage OCSP — reduit la latence de verification du certificat
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/mail-admin.example.com/chain.pem;
    resolver 1.1.1.1 8.8.8.8 valid=300s;
    resolver_timeout 5s;

    # Reprise de session
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    ssl_session_tickets off;

    # Certificats
    ssl_certificate /etc/letsencrypt/live/mail-admin.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mail-admin.example.com/privkey.pem;

    # Parametres DH (generer avec : openssl dhparam -out /etc/ssl/dhparam.pem 4096)
    ssl_dhparam /etc/ssl/dhparam.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Redirection HTTP vers HTTPS
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

# Cache d'agrafage OCSP (placer dans la config globale)
SSLStaplingCache shmcb:/var/run/ocsp(128000)

# Redirection HTTP vers HTTPS
<VirtualHost *:80>
    ServerName mail-admin.example.com
    Redirect permanent / https://mail-admin.example.com/
</VirtualHost>
```

### Validation

```bash
# Tester la configuration TLS
openssl s_client -connect mail-admin.example.com:443 -tls1_2
openssl s_client -connect mail-admin.example.com:443 -tls1_3

# Verifier l'agrafage OCSP
openssl s_client -connect mail-admin.example.com:443 -status

# Test en ligne (recommande)
# https://www.ssllabs.com/ssltest/ — viser la note A+
```

---

## 2. En-tetes de securite HTTP

### Nginx (ajouter dans le bloc `server`)

```nginx
# Content Security Policy — restreindre les origines des ressources
add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'" always;

# HSTS — forcer HTTPS pendant 1 an, inclure les sous-domaines, autoriser la liste preload
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;

# Empecher le clickjacking
add_header X-Frame-Options "DENY" always;

# Empecher le sniffing de type MIME
add_header X-Content-Type-Options "nosniff" always;

# Controler les informations de referrer
add_header Referrer-Policy "strict-origin-when-cross-origin" always;

# Restreindre les fonctionnalites du navigateur
add_header Permissions-Policy "camera=(), microphone=(), geolocation=(), payment=()" always;
```

### Configuration applicative (config.toml)

```toml
[security.headers]
csp_enabled = true
hsts_enabled = true
hsts_max_age = 31536000
```

---

## 3. Politique CORS pour l'API REST

Autoriser uniquement les requetes provenant d'origines connues. Ne jamais utiliser `*` en production.

### Configuration applicative (config.toml)

```toml
[api.cors]
allowed_origins = ["https://mail-admin.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]
allowed_headers = ["Content-Type", "Authorization", "X-CSRF-Token"]
allow_credentials = true
max_age = 3600
```

Si l'API est consommee par des clients externes, ajouter leurs origines specifiques :

```toml
allowed_origins = [
    "https://mail-admin.example.com",
    "https://monitoring.example.com",
]
```

---

## 4. Mitigation DDoS

### Limitation de debit Nginx

```nginx
# Definir les zones de limitation (placer dans le bloc http)
limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;
limit_req_zone $binary_remote_addr zone=api:10m rate=30r/s;
limit_req_zone $binary_remote_addr zone=web:10m rate=10r/s;
limit_conn_zone $binary_remote_addr zone=addr:10m;

server {
    # Limite globale de connexions par IP
    limit_conn addr 20;

    # Interface web — 10 req/s avec burst
    location / {
        limit_req zone=web burst=20 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }

    # Endpoint de login — limitation stricte (5 req/min)
    location /login {
        limit_req zone=login burst=3 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }

    # API — 30 req/s avec burst
    location /api/ {
        limit_req zone=api burst=50 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }

    # Limite de taille du corps de requete
    client_max_body_size 1m;

    # Timeouts
    proxy_connect_timeout 5s;
    proxy_read_timeout 30s;
    proxy_send_timeout 10s;
}
```

### Limitation de debit nftables

```bash
#!/usr/sbin/nft -f
table inet filter {
    chain input {
        type filter hook input priority 0; policy drop;

        # Autoriser les connexions etablies
        ct state established,related accept

        # Autoriser le loopback
        iif lo accept

        # Limiter les nouvelles connexions HTTPS : 25/seconde par IP
        tcp dport 443 ct state new limit rate over 25/second drop
        tcp dport 443 ct state new accept

        # SSH (admin uniquement — restreindre a votre IP)
        tcp dport 22 ip saddr 203.0.113.0/24 accept

        # Rejeter tout le reste
        drop
    }
}
```

---

## 5. Regles de pare-feu et segmentation reseau

### Principe : exposition minimale

- L'application ecoute sur `127.0.0.1:8080` (localhost uniquement)
- Seul le reverse proxy (Nginx/Apache) est expose sur les ports 80/443
- La base de donnees n'est accessible que depuis le serveur applicatif
- Le port gRPC (si utilise) n'est pas expose sur internet

### Configuration nftables de production

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

        # HTTP (redirection uniquement)
        tcp dport 80 accept

        # SSH (IP admin uniquement)
        tcp dport 22 ip saddr { 203.0.113.10, 203.0.113.11 } accept

        # ICMP (ping)
        ip protocol icmp accept
        ip6 nexthdr icmpv6 accept

        # Logger et rejeter tout le reste
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

### Isolation de la base de donnees

Si la base de donnees tourne sur un hote separe :

```bash
# Sur le serveur de BDD — accepter uniquement les connexions du serveur applicatif
nft add rule inet filter input ip saddr 10.0.1.10 tcp dport 5432 accept
nft add rule inet filter input tcp dport 5432 drop
```

Si vous utilisez Docker Compose, utilisez un reseau interne :

```yaml
services:
  app:
    networks:
      - frontend
      - backend

  db:
    networks:
      - backend   # Non expose au frontend

networks:
  frontend:
  backend:
    internal: true   # Pas d'acces externe
```

---

## 6. WAF (Web Application Firewall)

### ModSecurity avec OWASP Core Rule Set (Nginx)

```bash
# Installation (Debian/Ubuntu)
sudo apt install libmodsecurity3 libnginx-mod-http-modsecurity

# Activer OWASP CRS
sudo git clone https://github.com/coreruleset/coreruleset /etc/modsecurity/crs
sudo cp /etc/modsecurity/crs/crs-setup.conf.example /etc/modsecurity/crs/crs-setup.conf
```

Configuration Nginx :

```nginx
server {
    modsecurity on;
    modsecurity_rules_file /etc/modsecurity/main.conf;

    # ...
}
```

`/etc/modsecurity/main.conf` :

```
Include /etc/modsecurity/modsecurity.conf
Include /etc/modsecurity/crs/crs-setup.conf
Include /etc/modsecurity/crs/rules/*.conf

# Exclusions specifiques a l'application (si necessaire)
SecRuleRemoveById 920350   # Exemple : exclure une regle causant des faux positifs
```

### Alternative : Cloudflare ou AWS WAF

Pour des solutions WAF gerees, placez le service devant votre reverse proxy
et configurez la protection de l'IP d'origine pour empecher le contournement.

---

## 7. Gestion des secrets

### Generation de secrets robustes

```bash
# Cle secrete de session (64 octets, encodee en base64)
openssl rand -base64 64

# Mot de passe de base de donnees
openssl rand -base64 32

# Cle de signature JWT
openssl rand -base64 48

# Cle maitre CSRF
openssl rand -base64 32
```

### Variables d'environnement (recommande)

Ne jamais stocker les secrets dans `config.toml`. Utiliser des variables d'environnement :

```bash
# /etc/postfix-admin-rs/env (chmod 600, proprietaire root)
PAR_DATABASE__URL="postgresql://postfix:MOT_DE_PASSE_ROBUSTE@127.0.0.1:5432/postfix"
PAR_SERVER__SECRET_KEY="cle-base64-64-octets-generee"
PAR_AUTH__JWT_SECRET="cle-base64-48-octets-generee"
PAR_AUTH__MASTER_KEY="cle-base64-32-octets-generee"
```

Integration systemd :

```ini
[Service]
EnvironmentFile=/etc/postfix-admin-rs/env
```

### Regles

- **Ne jamais** commiter de secrets dans git (ajouter les patterns au `.gitignore`)
- **Ne jamais** logger de secrets (l'application l'impose)
- **Effectuer une rotation** des secrets periodiquement (au moins annuellement)
- **Utiliser des secrets differents** pour chaque environnement (dev, staging, production)

### Pattern d'integration HashiCorp Vault

Pour les organisations utilisant Vault :

```bash
# Stocker les secrets
vault kv put secret/postfix-admin-rs \
    database_url="postgresql://..." \
    secret_key="..." \
    jwt_secret="..."

# Recuperer au demarrage (script wrapper)
#!/bin/bash
export PAR_DATABASE__URL=$(vault kv get -field=database_url secret/postfix-admin-rs)
export PAR_SERVER__SECRET_KEY=$(vault kv get -field=secret_key secret/postfix-admin-rs)
export PAR_AUTH__JWT_SECRET=$(vault kv get -field=jwt_secret secret/postfix-admin-rs)
exec /usr/bin/postfix-admin-rs serve
```

---

## 8. Gestion des certificats

### Let's Encrypt avec certbot

```bash
# Installation
sudo apt install certbot

# Obtenir un certificat (methode webroot)
sudo certbot certonly --webroot \
    -w /var/www/html \
    -d mail-admin.example.com \
    --email admin@example.com \
    --agree-tos \
    --no-eff-email

# Ou avec le plugin Nginx
sudo certbot --nginx -d mail-admin.example.com
```

### Renouvellement automatique

```bash
# Tester le renouvellement
sudo certbot renew --dry-run

# Timer systemd (generalement installe par certbot)
sudo systemctl enable --now certbot.timer
```

Hook de renouvellement pour recharger le reverse proxy :

```bash
# /etc/letsencrypt/renewal-hooks/deploy/reload-nginx.sh
#!/bin/bash
systemctl reload nginx
```

### Surveillance des certificats

```bash
# Tache cron pour alerter sur les certificats expirant (14 jours avant)
0 8 * * * /usr/bin/openssl s_client -connect mail-admin.example.com:443 -servername mail-admin.example.com 2>/dev/null | openssl x509 -noout -checkend 1209600 || echo "Certificat expire bientot" | mail -s "ALERTE: certificat TLS" admin@example.com
```

---

## 9. Authentification par certificat client (mTLS) pour les administrateurs

Les certificats clients fournissent un facteur d'authentification supplementaire robuste pour les comptes a privileges. Le reverse proxy gere la verification des certificats clients TLS et transmet les informations d'identite a l'application via des en-tetes HTTP.

### Generation d'une CA et de certificats clients

```bash
# Creer une CA pour les certificats clients admin
openssl genrsa -out admin-ca.key 4096
openssl req -new -x509 -days 3650 -key admin-ca.key \
    -out admin-ca.crt -subj "/CN=PostfixAdmin Admin CA/O=Example Inc"

# Generer un certificat client pour un administrateur
openssl genrsa -out admin-client.key 2048
openssl req -new -key admin-client.key \
    -out admin-client.csr -subj "/emailAddress=admin@example.com/CN=Admin User/O=Example Inc"
openssl x509 -req -days 365 -in admin-client.csr \
    -CA admin-ca.crt -CAkey admin-ca.key -CAcreateserial \
    -out admin-client.crt

# Creer un bundle PKCS#12 pour import navigateur
openssl pkcs12 -export -out admin-client.p12 \
    -inkey admin-client.key -in admin-client.crt -certfile admin-ca.crt
```

### Configuration Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name mail-admin.example.com;

    # ... config TLS existante ...

    # Verification du certificat client
    ssl_client_certificate /etc/ssl/admin-ca.crt;
    ssl_verify_client optional;    # optional = ne pas exiger pour tous les utilisateurs

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Transmettre les infos du certificat client a l'application
        proxy_set_header X-SSL-Client-Verify $ssl_client_verify;
        proxy_set_header X-SSL-Client-S-DN $ssl_client_s_dn;
        proxy_set_header X-SSL-Client-Serial $ssl_client_serial;
    }
}
```

### Configuration Apache

```apache
<VirtualHost *:443>
    ServerName mail-admin.example.com

    # ... config TLS existante ...

    # Verification du certificat client
    SSLCACertificateFile /etc/ssl/admin-ca.crt
    SSLVerifyClient optional
    SSLVerifyDepth 2

    # Transmettre les infos du certificat client
    RequestHeader set X-SSL-Client-Verify "%{SSL_CLIENT_VERIFY}s"
    RequestHeader set X-SSL-Client-S-DN "%{SSL_CLIENT_S_DN}s"
    RequestHeader set X-SSL-Client-Serial "%{SSL_CLIENT_M_SERIAL}s"

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
</VirtualHost>
```

### Configuration applicative

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

### Considerations de securite

- **Usurpation d'en-tetes :** Le reverse proxy DOIT supprimer tout en-tete `X-SSL-Client-*` defini par le client avant la transmission. Nginx le fait automatiquement pour les variables `ssl_client_*` ; Apache necessite des directives `RequestHeader unset` pour les requetes non fiables.
- **Revocation de certificats :** Utiliser CRL ou OCSP pour gerer les certificats revoques. Configurer `ssl_crl` (Nginx) ou `SSLCARevocationFile` (Apache).
- **Validite des certificats :** Les certificats clients doivent avoir une duree de validite courte (90-365 jours) et etre renouveles regulierement.
- **Securite de la CA :** La cle privee de la CA admin doit etre stockee hors ligne ou dans un HSM. Ne jamais la stocker sur le serveur web.

---

## 10. Securite gRPC

Si l'API gRPC est activee, appliquer ces mesures :

### TLS mutuel (mTLS)

```toml
# config.toml
[grpc]
enabled = true
bind = "127.0.0.1:50051"       # Localhost uniquement — ne jamais exposer directement
tls_enabled = true
tls_cert_path = "/etc/postfix-admin-rs/grpc-server.crt"
tls_key_path = "/etc/postfix-admin-rs/grpc-server.key"
tls_ca_cert_path = "/etc/postfix-admin-rs/ca.crt"   # CA client pour mTLS
require_client_cert = true
```

### Desactiver la reflection en production

La reflection gRPC expose le schema de votre API. La desactiver en production :

```toml
[grpc]
reflection_enabled = false    # Activer uniquement en developpement
```

### Isolation reseau

Le port gRPC ne doit **jamais** etre expose sur internet. Si un acces distant
est necessaire, utiliser un VPN ou un tunnel SSH :

```bash
# Tunnel SSH pour acces gRPC distant
ssh -L 50051:127.0.0.1:50051 admin@mail-admin.example.com
```

---

## 11. Monitoring de securite et journalisation

### Journalisation d'audit

```toml
# config.toml
[logging]
level = "info"
format = "json"                    # Logs structures pour l'agregation
audit_enabled = true
audit_retention_days = 365

[logging.audit]
log_authentication = true          # Tentatives de connexion (succes + echec)
log_authorization = true           # Refus de permission
log_data_changes = true            # Operations CRUD domaine/boite/alias
log_admin_actions = true           # Operations de niveau admin
```

### Integration fail2ban

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

### Agregation de logs

Pour les environnements de production, rediriger les logs structures vers un systeme centralise :

- **ELK Stack** (Elasticsearch + Logstash + Kibana) — utiliser Filebeat comme agent
- **Loki + Grafana** — alternative legere avec LogQL
- **Graylog** — gestion centralisee des logs avec alerting

Exemple de configuration Filebeat :

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

### Regles d'alerte

Configurer des alertes pour :

- Plus de 10 tentatives de connexion echouees depuis la meme IP en 5 minutes
- Toute connexion reussie depuis une IP inconnue
- Changements de configuration
- Echecs de connexion a la base de donnees
- Expiration de certificat dans les 14 jours
- Redemarrages de l'application

---

## 12. Protection des donnees et RGPD

### Chiffrement au repos

#### Base de donnees

```sql
-- PostgreSQL : activer l'extension pgcrypto
CREATE EXTENSION IF NOT EXISTS pgcrypto;
```

Utiliser le chiffrement complet du disque (LUKS) sur le volume de la base :

```bash
# Chiffrer la partition de la base de donnees
sudo cryptsetup luksFormat /dev/sdb1
sudo cryptsetup open /dev/sdb1 pgdata
sudo mkfs.ext4 /dev/mapper/pgdata
sudo mount /dev/mapper/pgdata /var/lib/postgresql
```

#### Chiffrement des sauvegardes

```bash
# Sauvegarde chiffree avec GPG
pg_dump -U postfix postfix | gpg --symmetric --cipher-algo AES256 \
    --output /backups/postfix-$(date +%Y%m%d).sql.gpg

# Restauration
gpg --decrypt /backups/postfix-20250101.sql.gpg | psql -U postfix postfix
```

### Retention des donnees

```toml
# config.toml
[data_retention]
audit_log_days = 365               # Conserver les logs d'audit pendant 1 an
session_data_days = 30             # Purger les sessions expirees apres 30 jours
deleted_mailbox_days = 30          # Delai de grace avant suppression definitive
```

### Droit a l'effacement (RGPD Art. 17)

La CLI fournit des commandes pour l'effacement des donnees :

```bash
# Supprimer toutes les donnees associees a un utilisateur
postfix-admin-rs user purge --email user@example.com --confirm

# Exporter les donnees utilisateur (RGPD Art. 15 — droit d'acces)
postfix-admin-rs user export --email user@example.com --format json
```

### Considerations vie privee

- Les fichiers de log ne doivent pas contenir le contenu des emails
- Les adresses IP dans les logs sont considerees comme des donnees personnelles au sens du RGPD
- Implementer la rotation des logs avec suppression automatique apres la periode de retention
- Documenter vos activites de traitement des donnees (RGPD Art. 30)

---

## 13. Checklist de deploiement production

### TLS/SSL
- [ ] TLS 1.2+ impose, TLS 1.0/1.1 desactives
- [ ] Suites de chiffrement robustes avec Perfect Forward Secrecy
- [ ] Agrafage OCSP active
- [ ] Redirection HTTP vers HTTPS en place
- [ ] Score SSL Labs : A+

### Securite HTTP
- [ ] Tous les en-tetes de securite configures (CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)
- [ ] Soumission a la liste de preload HSTS (si applicable)
- [ ] Politique CORS restreint les origines aux domaines connus

### Reseau
- [ ] L'application ecoute sur localhost uniquement (127.0.0.1)
- [ ] Seuls les ports 80/443 sont exposes sur internet
- [ ] Le port de la base de donnees n'est pas accessible depuis internet
- [ ] Le port gRPC n'est pas accessible depuis internet
- [ ] Regles de pare-feu appliquees et testees
- [ ] Limitation de debit DDoS configuree

### Authentification
- [ ] Identifiants par defaut modifies
- [ ] Politique de mots de passe robuste imposee
- [ ] Limitation de debit sur les endpoints de login
- [ ] 2FA (TOTP) active pour les comptes admin
- [ ] Certificats clients (mTLS) actives pour les comptes superadmin
- [ ] Cookies de session : HttpOnly, Secure, SameSite=Strict

### Secrets
- [ ] Tous les secrets generes avec un generateur cryptographique
- [ ] Secrets stockes dans des variables d'environnement (pas dans les fichiers de config)
- [ ] Aucun secret commite dans git
- [ ] Secrets differents pour chaque environnement

### Certificats
- [ ] Certificats Let's Encrypt (ou equivalent) utilises
- [ ] Renouvellement automatique configure et teste
- [ ] Surveillance de l'expiration des certificats en place

### Monitoring
- [ ] Journalisation d'audit activee
- [ ] fail2ban (ou equivalent) configure
- [ ] Agregation de logs mise en place
- [ ] Regles d'alerte definies
- [ ] Endpoint de health check surveille

### Durcissement systeme
- [ ] Service systemd avec options de securite (NoNewPrivileges, ProtectSystem, etc.)
- [ ] L'application tourne sous un utilisateur non-privilegie
- [ ] Permissions fichiers : config `600`, binaire `755`
- [ ] WAF active (ModSecurity ou solution geree)
- [ ] `cargo audit` execute — aucune vulnerabilite connue

### Certificats clients (mTLS)
- [ ] CA admin generee et stockee de maniere securisee (hors ligne/HSM)
- [ ] Certificats clients emis pour tous les comptes admin
- [ ] Reverse proxy configure pour verifier les certificats clients
- [ ] Les en-tetes X-SSL-Client-* ne peuvent pas etre usurpes par les clients
- [ ] Mecanisme de revocation de certificats en place (CRL/OCSP)

### Protection des donnees
- [ ] Chiffrement de la base de donnees au repos (LUKS ou equivalent)
- [ ] Chiffrement des sauvegardes configure
- [ ] Politique de retention des donnees definie
- [ ] Commandes d'export/suppression RGPD testees
- [ ] Rotation des logs configuree

---

Voir aussi : [Guide de deploiement](DEPLOYMENT.md) | [Architecture](../../fr/architecture/ARCHITECTURE.md)

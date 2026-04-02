# Guide de migration — PostfixAdmin PHP vers postfix-admin-rs

## Prerequis

- PostfixAdmin PHP version 3.3+ (schema de base de donnees recent)
- Sauvegarde complete de la base de donnees
- Acces en lecture a la configuration PostfixAdmin PHP (`config.local.php`)
- Le serveur Postfix/Dovecot continue de fonctionner pendant la migration

## Strategie de migration

La migration est conçue pour etre **non-destructive** et **progressive** :

1. postfix-admin-rs se connecte a la base existante
2. Les migrations SQL ajoutent les nouvelles colonnes (sans casser les anciennes)
3. PostfixAdmin PHP peut continuer a fonctionner en parallele pendant la transition
4. Les mots de passe sont rehashes transparemment lors des connexions

## Etapes

### 1. Sauvegarde

```bash
# PostgreSQL
pg_dump -Fc postfix > postfix_backup_$(date +%Y%m%d).dump

# MySQL
mysqldump --single-transaction postfix > postfix_backup_$(date +%Y%m%d).sql

# SQLite
cp /path/to/postfixadmin.db /path/to/postfixadmin.db.backup
```

### 2. Installation de postfix-admin-rs

```bash
# Depuis le binaire
wget https://github.com/eric-lemesre/PostfixAdminRust/releases/latest/download/postfix-admin-rs-linux-amd64
chmod +x postfix-admin-rs-linux-amd64
sudo mv postfix-admin-rs-linux-amd64 /usr/local/bin/postfix-admin-rs

# Depuis les packages
sudo dpkg -i postfix-admin-rs_1.0.0_amd64.deb
```

### 3. Configuration

Creer `/etc/postfix-admin-rs/config.toml` a partir de la configuration PHP existante :

| PostfixAdmin PHP (`config.local.php`) | postfix-admin-rs (`config.toml`) |
|---------------------------------------|----------------------------------|
| `$CONF['database_type'] = 'pgsql'` | `database.url = "postgresql://..."` |
| `$CONF['database_host']` | Inclus dans `database.url` |
| `$CONF['database_user']` | Inclus dans `database.url` |
| `$CONF['database_password']` | Inclus dans `database.url` |
| `$CONF['database_name']` | Inclus dans `database.url` |
| `$CONF['encrypt'] = 'dovecot:BLF-CRYPT'` | `auth.password_scheme = "argon2id"` |
| `$CONF['page_size']` | `ui.page_size` |
| `$CONF['vacation_domain']` | `vacation.domain` |

### 4. Verification de la configuration

```bash
postfix-admin-rs config check
```

### 5. Migration du schema

```bash
# Voir l'etat des migrations
postfix-admin-rs migrate --status

# Appliquer les migrations
postfix-admin-rs migrate
```

Les migrations ajoutent :
- Colonnes manquantes (avec valeurs par defaut)
- Nouveaux index
- Nouvelles tables (si necessaire)
- Elles ne suppriment aucune colonne existante

### 6. Test en parallele

Demarrer postfix-admin-rs sur un port different :

```bash
PAR_SERVER__PORT=8081 postfix-admin-rs serve
```

Verifier :
- La liste des domaines est correcte
- Les boites mail sont accessibles
- Les alias sont corrects
- La connexion admin fonctionne (le mot de passe est rehash automatiquement)

### 7. Basculement

```bash
# Arreter PostfixAdmin PHP (desactiver le vhost Apache/Nginx)
sudo a2dissite postfixadmin
sudo systemctl reload apache2

# Demarrer postfix-admin-rs sur le port de production
sudo systemctl enable --now postfix-admin-rs
```

### 8. Configuration Postfix/Dovecot

Les requetes SQL de Postfix et Dovecot n'ont **pas besoin d'etre modifiees**
car le schema de base est compatible. Les tables `domain`, `mailbox`, `alias`
gardent les memes colonnes et les memes index.

## Compatibilite des mots de passe

| Format PostfixAdmin PHP | Support postfix-admin-rs |
|-------------------------|--------------------------|
| `{BLF-CRYPT}$2y$...` | Lecture + rehash auto |
| `$2y$...` (bcrypt brut) | Lecture + rehash auto |
| `{SHA512-CRYPT}$6$...` | Lecture + rehash auto |
| `{MD5-CRYPT}$1$...` | Lecture + rehash auto |
| `{PLAIN-MD5}...` | Lecture + rehash auto |
| `{CRYPT}...` | Lecture + rehash auto |

Lors de la premiere connexion reussie d'un utilisateur, son mot de passe
est automatiquement rehash en argon2id (schema par defaut).

## Rollback

En cas de probleme :

```bash
# Arreter postfix-admin-rs
sudo systemctl stop postfix-admin-rs

# Restaurer la sauvegarde (si les migrations ont cause des problemes)
pg_restore -d postfix postfix_backup_YYYYMMDD.dump

# Reactiver PostfixAdmin PHP
sudo a2ensite postfixadmin
sudo systemctl reload apache2
```

## Points d'attention

- **Prefixe de tables** : Si PostfixAdmin PHP utilise un prefixe de table,
  le configurer dans `database.table_prefix`
- **Vacation** : Le domaine de vacation (`autoreply.example.com`) doit rester le meme
- **Fetchmail** : Les mots de passe distants sont stockes differemment (chiffrement AES-256-GCM
  au lieu du texte quasi-clair de PHP). Une migration specifique les convertit
- **TOTP** : Si le 2FA etait deja actif dans PostfixAdmin PHP, les secrets TOTP
  sont preserves et re-chiffres avec la nouvelle cle

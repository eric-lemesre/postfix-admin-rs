> **Language:** [English](../en/features/11-cli/cli-administration.md) | Francais

# SPEC-11.1 — Interface en ligne de commande (CLI)

## Résumé

CLI d'administration pour la gestion du serveur mail sans interface web.
Basé sur `clap`, il offre une interface ergonomique avec sous-commandes,
auto-complétion et sortie formatée (table, JSON, CSV).

## Commandes

### Structure générale

```
postfix-admin-rs <SOUS-COMMANDE> <ACTION> [OPTIONS]
```

### Gestion des domaines

```
postfix-admin-rs domain list [--active] [--format json|table|csv]
postfix-admin-rs domain show <DOMAIN>
postfix-admin-rs domain add <DOMAIN> [--description "..."] [--aliases 0] [--mailboxes 0]
                            [--maxquota 0] [--quota 0] [--transport "virtual:"]
                            [--backupmx] [--no-active]
postfix-admin-rs domain edit <DOMAIN> [--description "..."] [--aliases N] ...
postfix-admin-rs domain delete <DOMAIN> [--force]
postfix-admin-rs domain toggle <DOMAIN>
```

### Gestion des boîtes mail

```
postfix-admin-rs mailbox list [--domain <DOMAIN>] [--active] [--format json|table|csv]
postfix-admin-rs mailbox show <USERNAME>
postfix-admin-rs mailbox add <USERNAME> --password <PASS> [--name "..."] [--quota N]
                             [--no-active]
postfix-admin-rs mailbox edit <USERNAME> [--name "..."] [--quota N]
postfix-admin-rs mailbox delete <USERNAME> [--force]
postfix-admin-rs mailbox toggle <USERNAME>
postfix-admin-rs mailbox password <USERNAME> --password <NEW_PASS>
postfix-admin-rs mailbox quota <USERNAME>
```

### Gestion des alias

```
postfix-admin-rs alias list [--domain <DOMAIN>] [--format json|table|csv]
postfix-admin-rs alias show <ADDRESS>
postfix-admin-rs alias add <ADDRESS> --goto <DEST1,DEST2,...>
postfix-admin-rs alias edit <ADDRESS> --goto <DEST1,DEST2,...>
postfix-admin-rs alias delete <ADDRESS> [--force]
```

### Gestion des admins

```
postfix-admin-rs admin list [--format json|table|csv]
postfix-admin-rs admin show <USERNAME>
postfix-admin-rs admin add <USERNAME> --password <PASS> [--superadmin]
                           [--domains domain1,domain2]
postfix-admin-rs admin edit <USERNAME> [--superadmin] [--no-superadmin]
postfix-admin-rs admin delete <USERNAME> [--force]
postfix-admin-rs admin password <USERNAME> --password <NEW_PASS>
postfix-admin-rs admin domains <USERNAME> --add <DOMAIN>
postfix-admin-rs admin domains <USERNAME> --remove <DOMAIN>
```

### Utilitaires

```
postfix-admin-rs setup                          # Setup initial (premier superadmin)
postfix-admin-rs migrate                        # Exécuter les migrations BDD
postfix-admin-rs migrate --status               # Voir l'état des migrations
postfix-admin-rs config check                   # Vérifier la configuration
postfix-admin-rs config show                    # Afficher la configuration active
postfix-admin-rs hash-password <PASSWORD>        # Hasher un mot de passe
postfix-admin-rs verify-password <HASH> <PASS>   # Vérifier un hash
postfix-admin-rs export [--domain <DOMAIN>]      # Export JSON complet
postfix-admin-rs import <FILE>                   # Import depuis JSON/CSV
postfix-admin-rs log list [--domain <D>] [--action <A>] [--last N]
postfix-admin-rs stats [--domain <DOMAIN>]       # Statistiques
postfix-admin-rs completion <SHELL>              # Génération auto-complétion
```

## Options globales

| Option | Court | Description |
|--------|-------|-------------|
| `--config` | `-c` | Chemin du fichier de configuration |
| `--database-url` | | URL de connexion BDD (override config) |
| `--format` | `-f` | Format de sortie : `table` (défaut), `json`, `csv` |
| `--quiet` | `-q` | Sortie minimale |
| `--verbose` | `-v` | Sortie détaillée (-vv pour debug) |
| `--yes` | `-y` | Confirmer automatiquement les actions destructives |
| `--color` | | Forcer/désactiver les couleurs (`auto`, `always`, `never`) |

## Format de sortie

### Table (défaut)
```
DOMAIN          DESCRIPTION     ALIASES  MAILBOXES  QUOTA   ACTIVE
example.com     Example Domain  10/50    5/100      2G/10G  yes
other.com       Other Domain    3/0      12/0       0/0     yes
```

### JSON
```json
[
  {
    "domain": "example.com",
    "description": "Example Domain",
    "aliases_count": 10,
    "aliases_limit": 50,
    "active": true
  }
]
```

### CSV
```
domain,description,aliases_count,aliases_limit,active
example.com,Example Domain,10,50,true
```

## Configuration CLI

Le CLI utilise le même fichier de configuration que le serveur web.
Il peut aussi se connecter directement à la base de données sans passer
par l'API (mode direct).

```toml
# Priorité de résolution :
# 1. --database-url en ligne de commande
# 2. Variable d'environnement DATABASE_URL
# 3. Fichier de configuration (--config ou /etc/postfix-admin-rs/config.toml)
```

## Codes de sortie

| Code | Signification |
|------|--------------|
| `0` | Succès |
| `1` | Erreur générale |
| `2` | Erreur de configuration |
| `3` | Erreur de connexion BDD |
| `4` | Erreur de validation |
| `5` | Ressource non trouvée |
| `10` | Action annulée par l'utilisateur |

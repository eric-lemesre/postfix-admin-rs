> **Language:** [English](../en/guidelines/GUIDELINES-SQL.md) | Francais

# Guidelines SQL — postfix-admin-rs

## Philosophie

Le projet supporte trois backends (PostgreSQL, MySQL, SQLite). Les migrations
et les requêtes doivent être compatibles ou fournies en variantes par backend.

---

## 1. Conventions de nommage

| Élément           | Convention                                    | Exemple                                        |
|-------------------|-----------------------------------------------|------------------------------------------------|
| Table             | snake_case, singulier                         | `domain`, `mailbox`, `alias`                   |
| Colonne           | snake_case                                    | `created_at`, `domain_name`                    |
| Clé primaire      | Nom descriptif ou `id`                        | `domain` (PK naturelle), `id` (PK synthétique) |
| Clé étrangère     | `{table_référencée}_{colonne}` ou nom logique | `domain` (FK vers domain.domain)               |
| Index             | `idx_{table}_{colonnes}`                      | `idx_mailbox_domain`                           |
| Contrainte unique | `uq_{table}_{colonnes}`                       | `uq_alias_domain_alias`                        |

### Préfixe de table

Le préfixe de table est configurable (`database.table_prefix` dans la config).
Toutes les requêtes doivent utiliser le nom logique, le préfixe est appliqué
au runtime.

---

## 2. Migrations

### Conventions

- Un fichier par migration : `YYYYMMDDHHMMSS_description.sql`
- Description courte en snake_case : `20240101120000_create_domain_table.sql`
- Chaque migration est idempotente quand possible (`IF NOT EXISTS`)
- Les migrations destructives (DROP, ALTER DROP COLUMN) doivent être réversibles

### Structure

```sql
-- migrations/20240101120000_create_domain_table.sql

CREATE TABLE IF NOT EXISTS domain (
    domain       VARCHAR(255)   NOT NULL,
    description  VARCHAR(255)   NOT NULL DEFAULT '',
    aliases      INTEGER        NOT NULL DEFAULT 0,
    mailboxes    INTEGER        NOT NULL DEFAULT 0,
    maxquota     BIGINT         NOT NULL DEFAULT 0,
    quota        BIGINT         NOT NULL DEFAULT 0,
    transport    VARCHAR(255)   DEFAULT NULL,
    backupmx     BOOLEAN        NOT NULL DEFAULT FALSE,
    active       BOOLEAN        NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ    NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_domain PRIMARY KEY (domain)
);

CREATE INDEX IF NOT EXISTS idx_domain_active ON domain (domain, active);
```

### Gestion multi-backend

Pour les différences de syntaxe entre backends, fournir des fichiers séparés
si nécessaire :

```
migrations/
├── 20240101120000_create_domain_table.sql           # PostgreSQL (défaut)
├── 20240101120000_create_domain_table.mysql.sql      # MySQL
└── 20240101120000_create_domain_table.sqlite.sql     # SQLite
```

Différences notables :

| Fonctionnalité | PostgreSQL              | MySQL                     | SQLite                            |
|----------------|-------------------------|---------------------------|-----------------------------------|
| Auto-increment | `SERIAL` / `GENERATED`  | `AUTO_INCREMENT`          | `INTEGER PRIMARY KEY` (implicite) |
| Boolean        | `BOOLEAN` natif         | `TINYINT(1)`              | `INTEGER` (0/1)                   |
| Timestamp      | `TIMESTAMPTZ`           | `DATETIME`                | `TEXT` (ISO 8601)                 |
| Texte illimité | `TEXT`                  | `TEXT` / `LONGTEXT`       | `TEXT`                            |
| JSON           | `JSONB`                 | `JSON`                    | `TEXT`                            |
| Upsert         | `ON CONFLICT DO UPDATE` | `ON DUPLICATE KEY UPDATE` | `ON CONFLICT DO UPDATE`           |

---

## 3. Requêtes

### Paramétrage

```rust
// TOUJOURS utiliser des requêtes paramétrées
// Bon :
sqlx::query!("SELECT * FROM domain WHERE domain = $1", name)

// INTERDIT :
sqlx::query(&format!("SELECT * FROM domain WHERE domain = '{}'", name))
```

### Style SQL

```sql
-- Mots-clés SQL en MAJUSCULES
-- Indentation cohérente
-- Une clause par ligne pour les requêtes complexes

SELECT
    d.domain,
    d.description,
    d.active,
    COUNT(m.username) AS mailbox_count,
    COALESCE(SUM(q.bytes), 0) AS total_usage
FROM domain d
LEFT JOIN mailbox m ON m.domain = d.domain AND m.active = TRUE
LEFT JOIN quota2 q ON q.username = m.username
WHERE d.active = TRUE
GROUP BY d.domain, d.description, d.active
ORDER BY d.domain ASC
LIMIT $1 OFFSET $2;
```

### Pagination

```sql
-- Toujours utiliser LIMIT + OFFSET
-- Toujours accompagner d'un ORDER BY déterministe
-- Le comptage total est une requête séparée (COUNT)

-- Requête de données
SELECT * FROM domain
ORDER BY domain ASC
LIMIT $1 OFFSET $2;

-- Requête de comptage
SELECT COUNT(*) AS total FROM domain
WHERE active = TRUE;
```

### Index

Les index doivent être créés pour :
- Les colonnes dans les clauses `WHERE` fréquentes
- Les colonnes dans les `ORDER BY`
- Les clés étrangères (non automatiques dans certains SGBD)
- Les colonnes dans les `JOIN`

```sql
-- Index composites : l'ordre des colonnes compte
-- La colonne la plus sélective en premier
CREATE INDEX idx_mailbox_domain_active ON mailbox (domain, active);
```

---

## 4. Sécurité

- Jamais de concaténation de chaînes dans les requêtes SQL
- Les mots de passe ne sont JAMAIS stockés en clair
- Les secrets (clés TOTP, mots de passe fetchmail) sont chiffrés avant stockage
- L'utilisateur de la BDD a les privilèges minimaux nécessaires :
  - `SELECT, INSERT, UPDATE, DELETE` sur les tables de l'application
  - Pas de `DROP`, `CREATE TABLE`, `ALTER TABLE` en runtime (seulement pour les migrations)

---

## 5. Performance

### Requêtes à éviter

```sql
-- Éviter SELECT * en production (lister les colonnes)
-- Mauvais :
SELECT * FROM mailbox WHERE domain = $1;

-- Bon :
SELECT username, name, quota, active, created_at
FROM mailbox
WHERE domain = $1;
```

```sql
-- Éviter les sous-requêtes corrélées, préférer les JOIN
-- Mauvais :
SELECT d.domain,
    (SELECT COUNT(*) FROM mailbox WHERE domain = d.domain) AS count
FROM domain d;

-- Bon :
SELECT d.domain, COUNT(m.username) AS count
FROM domain d
LEFT JOIN mailbox m ON m.domain = d.domain
GROUP BY d.domain;
```

### Pool de connexions

- Dimensionner le pool selon la charge attendue
- `min_connections` : 2 (garder des connexions chaudes)
- `max_connections` : 10-20 (selon le SGBD et le nombre de workers)
- `idle_timeout` : 5 minutes

---

## 6. Compatibilité PostfixAdmin PHP

Pour la migration depuis une base existante :

- Le schéma de base est compatible avec les tables PostfixAdmin PHP
- Les nouvelles colonnes sont ajoutées avec des valeurs par défaut
- Les colonnes renommées sont gérées par des migrations progressives
- L'application détecte la version du schéma et applique les migrations manquantes

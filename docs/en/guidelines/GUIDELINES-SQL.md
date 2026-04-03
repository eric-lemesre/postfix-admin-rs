> **Language:** English | [Francais](../fr/guidelines/GUIDELINES-SQL.md)

# SQL Guidelines — postfix-admin-rs

## Philosophy

The project supports three backends (PostgreSQL, MySQL, SQLite). Migrations and queries must be compatible or provided in variants by backend.

---

## 1. Naming Conventions

| Element           | Convention                                    | Example                                    |
|-------------------|-----------------------------------------------|--------------------------------------------|
| Table             | snake_case, singular                          | `domain`, `mailbox`, `alias`               |
| Column            | snake_case                                    | `created_at`, `domain_name`                |
| Primary Key       | Descriptive name or `id`                      | `domain` (natural PK), `id` (synthetic PK) |
| Foreign Key       | `{referenced_table}_{column}` or logical name | `domain` (FK to domain.domain)             |
| Index             | `idx_{table}_{columns}`                       | `idx_mailbox_domain`                       |
| Unique Constraint | `uq_{table}_{columns}`                        | `uq_alias_domain_alias`                    |

### Table Prefix

The table prefix is configurable (`database.table_prefix` in the config). All queries must use the logical name; the prefix is applied at runtime.

---

## 2. Migrations

### Conventions

- One file per migration: `YYYYMMDDHHMMSS_description.sql`
- Short description in snake_case: `20240101120000_create_domain_table.sql`
- Each migration should be idempotent where possible (`IF NOT EXISTS`)
- Destructive migrations (DROP, ALTER DROP COLUMN) must be reversible

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

### Multi-backend Management

For syntax differences between backends, provide separate files if necessary:

```
migrations/
├── 20240101120000_create_domain_table.sql           # PostgreSQL (default)
├── 20240101120000_create_domain_table.mysql.sql      # MySQL
└── 20240101120000_create_domain_table.sqlite.sql     # SQLite
```

Notable differences:

| Feature         | PostgreSQL              | MySQL                     | SQLite                            |
|-----------------|-------------------------|---------------------------|-----------------------------------|
| Auto-increment  | `SERIAL` / `GENERATED`  | `AUTO_INCREMENT`          | `INTEGER PRIMARY KEY` (implicit)  |
| Boolean         | Native `BOOLEAN`        | `TINYINT(1)`              | `INTEGER` (0/1)                   |
| Timestamp       | `TIMESTAMPTZ`           | `DATETIME`                | `TEXT` (ISO 8601)                 |
| Unlimited text  | `TEXT`                  | `TEXT` / `LONGTEXT`       | `TEXT`                            |
| JSON            | `JSONB`                 | `JSON`                    | `TEXT`                            |
| Upsert          | `ON CONFLICT DO UPDATE` | `ON DUPLICATE KEY UPDATE` | `ON CONFLICT DO UPDATE`           |

---

## 3. Queries

### Parameterization

```rust
// ALWAYS use parameterized queries
// Good:
sqlx::query!("SELECT * FROM domain WHERE domain = $1", name)

// FORBIDDEN:
sqlx::query(&format!("SELECT * FROM domain WHERE domain = '{}'", name))
```

### SQL Style

```sql
-- SQL keywords in UPPERCASE
-- Consistent indentation
-- One clause per line for complex queries

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
-- Always use LIMIT + OFFSET
-- Always accompanied by a deterministic ORDER BY
-- Total count is a separate query (COUNT)

-- Data query
SELECT * FROM domain
ORDER BY domain ASC
LIMIT $1 OFFSET $2;

-- Count query
SELECT COUNT(*) AS total FROM domain
WHERE active = TRUE;
```

### Indexes

Indexes should be created for:
- Columns in frequent `WHERE` clauses
- Columns in `ORDER BY`
- Foreign keys (not automatic in some DBMS)
- Columns in `JOIN`

```sql
-- Composite indexes: column order matters
-- Most selective column first
CREATE INDEX idx_mailbox_domain_active ON mailbox (domain, active);
```

---

## 4. Security

- Never concatenate strings in SQL queries
- Passwords are NEVER stored in plain text
- Secrets (TOTP keys, fetchmail passwords) are encrypted before storage
- The database user has minimal required privileges:
  - `SELECT, INSERT, UPDATE, DELETE` on application tables
  - No `DROP`, `CREATE TABLE`, or `ALTER TABLE` at runtime (only for migrations)

---

## 5. Performance

### Queries to Avoid

```sql
-- Avoid SELECT * in production (list columns)
-- Bad:
SELECT * FROM mailbox WHERE domain = $1;

-- Good:
SELECT username, name, quota, active, created_at
FROM mailbox
WHERE domain = $1;
```

```sql
-- Avoid correlated subqueries, prefer JOINs
-- Bad:
SELECT d.domain,
    (SELECT COUNT(*) FROM mailbox WHERE domain = d.domain) AS count
FROM domain d;

-- Good:
SELECT d.domain, COUNT(m.username) AS count
FROM domain d
LEFT JOIN mailbox m ON m.domain = d.domain
GROUP BY d.domain;
```

### Connection Pool

- Size the pool according to expected load
- `min_connections`: 2 (keep warm connections)
- `max_connections`: 10-20 (depending on DBMS and number of workers)
- `idle_timeout`: 5 minutes

---

## 6. Compatibility with PostfixAdmin PHP

For migration from an existing database:

- The base schema is compatible with PostfixAdmin PHP tables
- New columns are added with default values
- Renamed columns are handled by progressive migrations
- The application detects the schema version and applies missing migrations

---

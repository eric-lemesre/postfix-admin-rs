-- PostfixAdmin-RS: PostgreSQL initial schema
-- Creates all 15 tables with proper FK ordering

-- 1. domain (no dependencies)
CREATE TABLE IF NOT EXISTS domain (
    domain       VARCHAR(255) NOT NULL,
    description  VARCHAR(255) NOT NULL DEFAULT '',
    aliases      INTEGER      NOT NULL DEFAULT 0,
    mailboxes    INTEGER      NOT NULL DEFAULT 0,
    maxquota     BIGINT       NOT NULL DEFAULT 0,
    quota        BIGINT       NOT NULL DEFAULT 0,
    transport    VARCHAR(255),
    backupmx     BOOLEAN      NOT NULL DEFAULT FALSE,
    password_expiry INTEGER   NOT NULL DEFAULT 0,
    active       BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (domain)
);

-- 2. admin (no dependencies)
CREATE TABLE IF NOT EXISTS admin (
    username     VARCHAR(255) NOT NULL,
    password     VARCHAR(255) NOT NULL DEFAULT '',
    superadmin   BOOLEAN      NOT NULL DEFAULT FALSE,
    totp_secret  VARCHAR(255),
    active       BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (username)
);

-- 3. domain_admins (FK -> admin, domain)
CREATE TABLE IF NOT EXISTS domain_admins (
    username     VARCHAR(255) NOT NULL,
    domain       VARCHAR(255) NOT NULL,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (username, domain),
    FOREIGN KEY (username) REFERENCES admin(username) ON DELETE CASCADE,
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
);

-- 4. mailbox (FK -> domain)
CREATE TABLE IF NOT EXISTS mailbox (
    username       VARCHAR(255) NOT NULL,
    password       VARCHAR(255) NOT NULL DEFAULT '',
    name           VARCHAR(255) NOT NULL DEFAULT '',
    maildir        VARCHAR(255) NOT NULL DEFAULT '',
    quota          BIGINT       NOT NULL DEFAULT 0,
    local_part     VARCHAR(255) NOT NULL DEFAULT '',
    domain         VARCHAR(255) NOT NULL,
    password_expiry TIMESTAMPTZ,
    totp_secret    VARCHAR(255),
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (username),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
);

-- 5. alias (FK -> domain)
CREATE TABLE IF NOT EXISTS alias (
    address      VARCHAR(255) NOT NULL,
    goto         TEXT         NOT NULL DEFAULT '',
    domain       VARCHAR(255) NOT NULL,
    active       BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (address),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
);

-- 6. alias_domain (FK -> domain)
CREATE TABLE IF NOT EXISTS alias_domain (
    alias_domain   VARCHAR(255) NOT NULL,
    target_domain  VARCHAR(255) NOT NULL,
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (alias_domain),
    FOREIGN KEY (alias_domain) REFERENCES domain(domain) ON DELETE CASCADE,
    FOREIGN KEY (target_domain) REFERENCES domain(domain) ON DELETE CASCADE
);

-- 7. vacation (FK -> domain)
CREATE TABLE IF NOT EXISTS vacation (
    email          VARCHAR(255) NOT NULL,
    subject        VARCHAR(255) NOT NULL DEFAULT '',
    body           TEXT         NOT NULL DEFAULT '',
    domain         VARCHAR(255) NOT NULL,
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    active_from    TIMESTAMPTZ,
    active_until   TIMESTAMPTZ,
    interval_time  INTEGER      NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (email),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
);

-- 8. vacation_notification (FK -> vacation, CASCADE)
CREATE TABLE IF NOT EXISTS vacation_notification (
    on_vacation    VARCHAR(255) NOT NULL,
    notified       VARCHAR(255) NOT NULL DEFAULT '',
    notified_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (on_vacation, notified),
    FOREIGN KEY (on_vacation) REFERENCES vacation(email) ON DELETE CASCADE
);

-- 9. fetchmail (FK -> domain, mailbox)
CREATE TABLE IF NOT EXISTS fetchmail (
    id             UUID         NOT NULL,
    domain         VARCHAR(255) NOT NULL,
    mailbox        VARCHAR(255) NOT NULL,
    src_server     VARCHAR(255) NOT NULL DEFAULT '',
    src_auth       VARCHAR(255) NOT NULL DEFAULT 'password',
    src_user       VARCHAR(255) NOT NULL DEFAULT '',
    src_password   VARCHAR(255) NOT NULL DEFAULT '',
    src_folder     VARCHAR(255) NOT NULL DEFAULT '',
    poll_time      INTEGER      NOT NULL DEFAULT 10,
    fetchall       BOOLEAN      NOT NULL DEFAULT FALSE,
    keep           BOOLEAN      NOT NULL DEFAULT TRUE,
    protocol       VARCHAR(255) NOT NULL DEFAULT 'IMAP',
    usessl         BOOLEAN      NOT NULL DEFAULT TRUE,
    sslcertck      BOOLEAN      NOT NULL DEFAULT TRUE,
    extra_options  TEXT,
    mda            VARCHAR(255) NOT NULL DEFAULT '',
    returned_text  TEXT,
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    date           TIMESTAMPTZ,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE,
    FOREIGN KEY (mailbox) REFERENCES mailbox(username) ON DELETE CASCADE
);

-- 10. dkim_key (FK -> domain, CASCADE)
CREATE TABLE IF NOT EXISTS dkim_key (
    id             UUID         NOT NULL,
    domain_name    VARCHAR(255) NOT NULL,
    description    VARCHAR(255) NOT NULL DEFAULT '',
    selector       VARCHAR(63)  NOT NULL DEFAULT 'default',
    private_key    TEXT         NOT NULL DEFAULT '',
    public_key     TEXT         NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    FOREIGN KEY (domain_name) REFERENCES domain(domain) ON DELETE CASCADE
);

-- 11. dkim_signing (FK -> dkim_key, CASCADE)
CREATE TABLE IF NOT EXISTS dkim_signing (
    id             UUID         NOT NULL,
    author         VARCHAR(255) NOT NULL DEFAULT '',
    dkim_id        UUID         NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    FOREIGN KEY (dkim_id) REFERENCES dkim_key(id) ON DELETE CASCADE
);

-- 12. mailbox_app_password (FK -> mailbox)
CREATE TABLE IF NOT EXISTS mailbox_app_password (
    id             UUID         NOT NULL,
    username       VARCHAR(255) NOT NULL,
    description    VARCHAR(255) NOT NULL DEFAULT '',
    password_hash  VARCHAR(255) NOT NULL DEFAULT '',
    last_used      TIMESTAMPTZ,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    FOREIGN KEY (username) REFERENCES mailbox(username) ON DELETE CASCADE
);

-- 13. totp_exception_address (no FK)
CREATE TABLE IF NOT EXISTS totp_exception_address (
    id             UUID         NOT NULL,
    ip             VARCHAR(46)  NOT NULL,
    username       VARCHAR(255),
    description    VARCHAR(255),
    PRIMARY KEY (id)
);

-- 14. log (no FK)
CREATE TABLE IF NOT EXISTS log (
    id             UUID         NOT NULL,
    timestamp      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    username       VARCHAR(255) NOT NULL DEFAULT '',
    domain         VARCHAR(255) NOT NULL DEFAULT '',
    action         VARCHAR(255) NOT NULL DEFAULT '',
    data           TEXT         NOT NULL DEFAULT '',
    ip_address     VARCHAR(46),
    user_agent     TEXT,
    PRIMARY KEY (id)
);

-- 15. quota2 (FK -> mailbox)
CREATE TABLE IF NOT EXISTS quota2 (
    username       VARCHAR(255) NOT NULL,
    bytes          BIGINT       NOT NULL DEFAULT 0,
    messages       INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (username),
    FOREIGN KEY (username) REFERENCES mailbox(username) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_domain_active ON domain(domain, active);
CREATE INDEX IF NOT EXISTS idx_mailbox_username_active ON mailbox(username, active);
CREATE INDEX IF NOT EXISTS idx_mailbox_domain ON mailbox(domain);
CREATE INDEX IF NOT EXISTS idx_alias_address_active ON alias(address, active);
CREATE INDEX IF NOT EXISTS idx_alias_domain_col ON alias(domain);
CREATE INDEX IF NOT EXISTS idx_alias_domain_active ON alias_domain(alias_domain, active);
CREATE INDEX IF NOT EXISTS idx_alias_domain_target ON alias_domain(target_domain);
CREATE INDEX IF NOT EXISTS idx_dkim_key_domain ON dkim_key(domain_name);
CREATE INDEX IF NOT EXISTS idx_dkim_signing_author ON dkim_signing(author);
CREATE INDEX IF NOT EXISTS idx_log_timestamp ON log(timestamp);
CREATE INDEX IF NOT EXISTS idx_log_domain ON log(domain);
CREATE INDEX IF NOT EXISTS idx_log_username ON log(username);

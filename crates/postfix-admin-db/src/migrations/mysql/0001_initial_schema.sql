-- PostfixAdmin-RS: MySQL initial schema
-- Creates all 15 tables with proper FK ordering

-- 1. domain (no dependencies)
CREATE TABLE IF NOT EXISTS domain (
    domain       VARCHAR(255) NOT NULL,
    description  VARCHAR(255) NOT NULL DEFAULT '',
    aliases      INT          NOT NULL DEFAULT 0,
    mailboxes    INT          NOT NULL DEFAULT 0,
    maxquota     BIGINT       NOT NULL DEFAULT 0,
    quota        BIGINT       NOT NULL DEFAULT 0,
    transport    VARCHAR(255) DEFAULT NULL,
    backupmx     BOOLEAN      NOT NULL DEFAULT FALSE,
    password_expiry INT       NOT NULL DEFAULT 0,
    active       BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (domain)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 2. admin (no dependencies)
CREATE TABLE IF NOT EXISTS admin (
    username     VARCHAR(255) NOT NULL,
    password     VARCHAR(255) NOT NULL DEFAULT '',
    superadmin   BOOLEAN      NOT NULL DEFAULT FALSE,
    totp_secret  VARCHAR(255) DEFAULT NULL,
    active       BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (username)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 3. domain_admins (FK -> admin, domain)
CREATE TABLE IF NOT EXISTS domain_admins (
    username     VARCHAR(255) NOT NULL,
    domain       VARCHAR(255) NOT NULL,
    created_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (username, domain),
    FOREIGN KEY (username) REFERENCES admin(username) ON DELETE CASCADE,
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 4. mailbox (FK -> domain)
CREATE TABLE IF NOT EXISTS mailbox (
    username       VARCHAR(255) NOT NULL,
    password       VARCHAR(255) NOT NULL DEFAULT '',
    name           VARCHAR(255) NOT NULL DEFAULT '',
    maildir        VARCHAR(255) NOT NULL DEFAULT '',
    quota          BIGINT       NOT NULL DEFAULT 0,
    local_part     VARCHAR(255) NOT NULL DEFAULT '',
    domain         VARCHAR(255) NOT NULL,
    password_expiry TIMESTAMP   DEFAULT NULL,
    totp_secret    VARCHAR(255) DEFAULT NULL,
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (username),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 5. alias (FK -> domain)
CREATE TABLE IF NOT EXISTS alias (
    address      VARCHAR(255) NOT NULL,
    goto         TEXT         NOT NULL,
    domain       VARCHAR(255) NOT NULL,
    active       BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (address),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 6. alias_domain (FK -> domain)
CREATE TABLE IF NOT EXISTS alias_domain (
    alias_domain   VARCHAR(255) NOT NULL,
    target_domain  VARCHAR(255) NOT NULL,
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (alias_domain),
    FOREIGN KEY (alias_domain) REFERENCES domain(domain) ON DELETE CASCADE,
    FOREIGN KEY (target_domain) REFERENCES domain(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 7. vacation (FK -> domain)
CREATE TABLE IF NOT EXISTS vacation (
    email          VARCHAR(255) NOT NULL,
    subject        VARCHAR(255) NOT NULL DEFAULT '',
    body           TEXT         NOT NULL,
    domain         VARCHAR(255) NOT NULL,
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    active_from    TIMESTAMP    DEFAULT NULL,
    active_until   TIMESTAMP    DEFAULT NULL,
    interval_time  INT          NOT NULL DEFAULT 0,
    created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (email),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 8. vacation_notification (FK -> vacation, CASCADE)
CREATE TABLE IF NOT EXISTS vacation_notification (
    on_vacation    VARCHAR(255) NOT NULL,
    notified       VARCHAR(255) NOT NULL DEFAULT '',
    notified_at    TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (on_vacation, notified),
    FOREIGN KEY (on_vacation) REFERENCES vacation(email) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 9. fetchmail (FK -> domain, mailbox)
CREATE TABLE IF NOT EXISTS fetchmail (
    id             BINARY(16)   NOT NULL,
    domain         VARCHAR(255) NOT NULL,
    mailbox        VARCHAR(255) NOT NULL,
    src_server     VARCHAR(255) NOT NULL DEFAULT '',
    src_auth       VARCHAR(255) NOT NULL DEFAULT 'password',
    src_user       VARCHAR(255) NOT NULL DEFAULT '',
    src_password   VARCHAR(255) NOT NULL DEFAULT '',
    src_folder     VARCHAR(255) NOT NULL DEFAULT '',
    poll_time      INT          NOT NULL DEFAULT 10,
    fetchall       BOOLEAN      NOT NULL DEFAULT FALSE,
    keep_mail      BOOLEAN      NOT NULL DEFAULT TRUE,
    protocol       VARCHAR(255) NOT NULL DEFAULT 'IMAP',
    usessl         BOOLEAN      NOT NULL DEFAULT TRUE,
    sslcertck      BOOLEAN      NOT NULL DEFAULT TRUE,
    extra_options  TEXT         DEFAULT NULL,
    mda            VARCHAR(255) NOT NULL DEFAULT '',
    returned_text  TEXT         DEFAULT NULL,
    active         BOOLEAN      NOT NULL DEFAULT TRUE,
    date           TIMESTAMP    DEFAULT NULL,
    created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (domain) REFERENCES domain(domain) ON DELETE CASCADE,
    FOREIGN KEY (mailbox) REFERENCES mailbox(username) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 10. dkim_key (FK -> domain, CASCADE)
CREATE TABLE IF NOT EXISTS dkim_key (
    id             BINARY(16)   NOT NULL,
    domain_name    VARCHAR(255) NOT NULL,
    description    VARCHAR(255) NOT NULL DEFAULT '',
    selector       VARCHAR(63)  NOT NULL DEFAULT 'default',
    private_key    TEXT         NOT NULL,
    public_key     TEXT         NOT NULL,
    created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (domain_name) REFERENCES domain(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 11. dkim_signing (FK -> dkim_key, CASCADE)
CREATE TABLE IF NOT EXISTS dkim_signing (
    id             BINARY(16)   NOT NULL,
    author         VARCHAR(255) NOT NULL DEFAULT '',
    dkim_id        BINARY(16)   NOT NULL,
    created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (dkim_id) REFERENCES dkim_key(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 12. mailbox_app_password (FK -> mailbox)
CREATE TABLE IF NOT EXISTS mailbox_app_password (
    id             BINARY(16)   NOT NULL,
    username       VARCHAR(255) NOT NULL,
    description    VARCHAR(255) NOT NULL DEFAULT '',
    password_hash  VARCHAR(255) NOT NULL DEFAULT '',
    last_used      TIMESTAMP    DEFAULT NULL,
    created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (username) REFERENCES mailbox(username) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 13. totp_exception_address (no FK)
CREATE TABLE IF NOT EXISTS totp_exception_address (
    id             BINARY(16)   NOT NULL,
    ip             VARCHAR(46)  NOT NULL,
    username       VARCHAR(255) DEFAULT NULL,
    description    VARCHAR(255) DEFAULT NULL,
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 14. log (no FK)
CREATE TABLE IF NOT EXISTS log (
    id             BINARY(16)   NOT NULL,
    timestamp      TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    username       VARCHAR(255) NOT NULL DEFAULT '',
    domain         VARCHAR(255) NOT NULL DEFAULT '',
    action         VARCHAR(255) NOT NULL DEFAULT '',
    data           TEXT         NOT NULL,
    ip_address     VARCHAR(46)  DEFAULT NULL,
    user_agent     TEXT         DEFAULT NULL,
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 15. quota2 (FK -> mailbox)
CREATE TABLE IF NOT EXISTS quota2 (
    username       VARCHAR(255) NOT NULL,
    bytes          BIGINT       NOT NULL DEFAULT 0,
    messages       INT          NOT NULL DEFAULT 0,
    PRIMARY KEY (username),
    FOREIGN KEY (username) REFERENCES mailbox(username) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Indexes
CREATE INDEX idx_domain_active ON domain(domain, active);
CREATE INDEX idx_mailbox_username_active ON mailbox(username, active);
CREATE INDEX idx_mailbox_domain ON mailbox(domain);
CREATE INDEX idx_alias_address_active ON alias(address, active);
CREATE INDEX idx_alias_domain_col ON alias(domain);
CREATE INDEX idx_alias_domain_active ON alias_domain(alias_domain, active);
CREATE INDEX idx_alias_domain_target ON alias_domain(target_domain);
CREATE INDEX idx_dkim_key_domain ON dkim_key(domain_name);
CREATE INDEX idx_dkim_signing_author ON dkim_signing(author);
CREATE INDEX idx_log_timestamp ON log(timestamp);
CREATE INDEX idx_log_domain ON log(domain);
CREATE INDEX idx_log_username ON log(username);

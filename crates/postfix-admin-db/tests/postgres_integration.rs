#![allow(clippy::unwrap_used)]

mod common;

use postfix_admin_core::dto::{
    CreateAdmin, CreateAlias, CreateAliasDomain, CreateAppPassword, CreateDkimKey,
    CreateDkimSigning, CreateDomain, CreateFetchmail, CreateLog, CreateMailbox, LogFilter,
    UpdateAlias, UpdateDomain, UpdateMailbox, UpdateVacation,
};
use postfix_admin_core::pagination::PageRequest;
use postfix_admin_core::repository::{
    AdminRepository, AliasDomainRepository, AliasRepository, AppPasswordRepository, DkimRepository,
    DomainRepository, FetchmailRepository, LogRepository, MailboxRepository, VacationRepository,
};
use postfix_admin_core::{DomainName, EmailAddress, Password};

use postfix_admin_db::{
    pg_transaction, DbError, PgAdminRepository, PgAliasDomainRepository, PgAliasRepository,
    PgAppPasswordRepository, PgDkimRepository, PgDomainRepository, PgFetchmailRepository,
    PgLogRepository, PgMailboxRepository, PgVacationRepository,
};

#[tokio::test]
async fn pg_domain_create_valid_returns_response() {
    let (_container, pool) = common::setup_pg().await;
    let repo = PgDomainRepository::new(pool);

    let dto = CreateDomain {
        domain: DomainName::new("example.com").unwrap(),
        description: Some("Test domain".to_string()),
        aliases: Some(100),
        mailboxes: Some(50),
        maxquota: Some(10240),
        quota: Some(20480),
        transport: None,
        backupmx: Some(false),
        password_expiry: Some(0),
        active: Some(true),
    };

    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.domain.as_str(), "example.com");
    assert_eq!(resp.description, "Test domain");
    assert_eq!(resp.aliases, 100);
    assert!(resp.active);
}

#[tokio::test]
async fn pg_domain_find_by_name_existing_returns_some() {
    let (_container, pool) = common::setup_pg().await;
    let repo = PgDomainRepository::new(pool);

    let dto = CreateDomain {
        domain: DomainName::new("find-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    repo.create(&dto).await.unwrap();

    let name = DomainName::new("find-test.com").unwrap();
    let found = repo.find_by_name(&name).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().domain.as_str(), "find-test.com");
}

#[tokio::test]
async fn pg_domain_update_changes_description() {
    let (_container, pool) = common::setup_pg().await;
    let repo = PgDomainRepository::new(pool);

    let dto = CreateDomain {
        domain: DomainName::new("update-test.com").unwrap(),
        description: Some("Original".to_string()),
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    repo.create(&dto).await.unwrap();

    let name = DomainName::new("update-test.com").unwrap();
    let update = UpdateDomain {
        description: Some("Updated".to_string()),
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    let resp = repo.update(&name, &update).await.unwrap();
    assert_eq!(resp.description, "Updated");
}

#[tokio::test]
async fn pg_domain_delete_removes_domain() {
    let (_container, pool) = common::setup_pg().await;
    let repo = PgDomainRepository::new(pool);

    let dto = CreateDomain {
        domain: DomainName::new("delete-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    repo.create(&dto).await.unwrap();

    let name = DomainName::new("delete-test.com").unwrap();
    repo.delete(&name).await.unwrap();

    let found = repo.find_by_name(&name).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn pg_domain_count_returns_correct_value() {
    let (_container, pool) = common::setup_pg().await;
    let repo = PgDomainRepository::new(pool);

    assert_eq!(repo.count().await.unwrap(), 0);

    let dto = CreateDomain {
        domain: DomainName::new("count-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    repo.create(&dto).await.unwrap();
    assert_eq!(repo.count().await.unwrap(), 1);
}

#[tokio::test]
async fn pg_admin_create_and_find_by_username() {
    let (_container, pool) = common::setup_pg().await;
    let repo = PgAdminRepository::new(pool);

    let dto = CreateAdmin {
        username: EmailAddress::new("admin@example.com").unwrap(),
        password: Password::new("securepass123").unwrap(),
        superadmin: Some(true),
        active: Some(true),
    };
    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.username.as_str(), "admin@example.com");
    assert!(resp.superadmin);

    let username = EmailAddress::new("admin@example.com").unwrap();
    let found = repo.find_by_username(&username).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn pg_mailbox_create_and_find_by_domain() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let repo = PgMailboxRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("mailbox-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let dto = CreateMailbox {
        username: EmailAddress::new("user@mailbox-test.com").unwrap(),
        password: Password::new("securepass123").unwrap(),
        name: Some("Test User".to_string()),
        quota: Some(1024),
        active: Some(true),
    };
    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.username.as_str(), "user@mailbox-test.com");
    assert_eq!(resp.domain.as_str(), "mailbox-test.com");
    assert_eq!(resp.local_part, "user");

    let domain = DomainName::new("mailbox-test.com").unwrap();
    let page = PageRequest::new(1, 25);
    let page_resp = repo.find_by_domain(&domain, &page).await.unwrap();
    assert_eq!(page_resp.items.len(), 1);
}

#[tokio::test]
async fn pg_mailbox_update_changes_name() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let repo = PgMailboxRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("mbox-upd.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let dto = CreateMailbox {
        username: EmailAddress::new("user@mbox-upd.com").unwrap(),
        password: Password::new("securepass123").unwrap(),
        name: Some("Old Name".to_string()),
        quota: None,
        active: None,
    };
    repo.create(&dto).await.unwrap();

    let username = EmailAddress::new("user@mbox-upd.com").unwrap();
    let update = UpdateMailbox {
        password: None,
        name: Some("New Name".to_string()),
        quota: None,
        active: None,
    };
    let resp = repo.update(&username, &update).await.unwrap();
    assert_eq!(resp.name, "New Name");
}

#[tokio::test]
async fn pg_alias_create_and_find() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let repo = PgAliasRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("alias-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let dto = CreateAlias {
        address: EmailAddress::new("info@alias-test.com").unwrap(),
        goto: "user@alias-test.com".to_string(),
        active: Some(true),
    };
    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.address.as_str(), "info@alias-test.com");
    assert_eq!(resp.goto, "user@alias-test.com");

    let address = EmailAddress::new("info@alias-test.com").unwrap();
    let found = repo.find_by_address(&address).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn pg_alias_update_changes_goto() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let repo = PgAliasRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("alias-upd.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let dto = CreateAlias {
        address: EmailAddress::new("info@alias-upd.com").unwrap(),
        goto: "old@alias-upd.com".to_string(),
        active: None,
    };
    repo.create(&dto).await.unwrap();

    let address = EmailAddress::new("info@alias-upd.com").unwrap();
    let update = UpdateAlias {
        goto: Some("new@alias-upd.com".to_string()),
        active: None,
    };
    let resp = repo.update(&address, &update).await.unwrap();
    assert_eq!(resp.goto, "new@alias-upd.com");
}

#[tokio::test]
async fn pg_alias_domain_create_and_find() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let repo = PgAliasDomainRepository::new(pool);

    for name in &["source-ad.com", "target-ad.com"] {
        let dto = CreateDomain {
            domain: DomainName::new(*name).unwrap(),
            description: None,
            aliases: None,
            mailboxes: None,
            maxquota: None,
            quota: None,
            transport: None,
            backupmx: None,
            password_expiry: None,
            active: None,
        };
        domain_repo.create(&dto).await.unwrap();
    }

    let dto = CreateAliasDomain {
        alias_domain: DomainName::new("source-ad.com").unwrap(),
        target_domain: DomainName::new("target-ad.com").unwrap(),
        active: Some(true),
    };
    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.alias_domain.as_str(), "source-ad.com");
    assert_eq!(resp.target_domain.as_str(), "target-ad.com");

    let target = DomainName::new("target-ad.com").unwrap();
    let found = repo.find_by_target(&target).await.unwrap();
    assert_eq!(found.len(), 1);
}

#[tokio::test]
async fn pg_vacation_upsert_and_find() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let repo = PgVacationRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("vacation-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let email = EmailAddress::new("user@vacation-test.com").unwrap();
    let dto = UpdateVacation {
        subject: Some("Out of office".to_string()),
        body: Some("I am away".to_string()),
        active: Some(true),
        active_from: None,
        active_until: None,
        interval_time: Some(86400),
    };
    let resp = repo.upsert(&email, &dto).await.unwrap();
    assert_eq!(resp.subject, "Out of office");
    assert_eq!(resp.interval_time, 86400);

    let found = repo.find_by_email(&email).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn pg_dkim_create_key_and_signing() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let repo = PgDkimRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("dkim-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let key_dto = CreateDkimKey {
        domain_name: DomainName::new("dkim-test.com").unwrap(),
        description: Some("Test key".to_string()),
        selector: Some("default".to_string()),
        private_key: "private-key-data".to_string(),
        public_key: "public-key-data".to_string(),
    };
    let key_resp = repo.create_key(&key_dto).await.unwrap();
    assert_eq!(key_resp.domain_name.as_str(), "dkim-test.com");
    assert_eq!(key_resp.public_key, "public-key-data");

    let signing_dto = CreateDkimSigning {
        author: "dkim-test.com".to_string(),
        dkim_id: key_resp.id,
    };
    let signing_resp = repo.create_signing(&signing_dto).await.unwrap();
    assert_eq!(signing_resp.dkim_id, key_resp.id);

    let signings = repo.find_signings_by_key_id(key_resp.id).await.unwrap();
    assert_eq!(signings.len(), 1);
}

#[tokio::test]
async fn pg_fetchmail_create_and_find() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let mailbox_repo = PgMailboxRepository::new(pool.clone());
    let repo = PgFetchmailRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("fetch-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let mailbox_dto = CreateMailbox {
        username: EmailAddress::new("user@fetch-test.com").unwrap(),
        password: Password::new("securepass123").unwrap(),
        name: None,
        quota: None,
        active: None,
    };
    mailbox_repo.create(&mailbox_dto).await.unwrap();

    let dto = CreateFetchmail {
        mailbox: EmailAddress::new("user@fetch-test.com").unwrap(),
        src_server: "imap.external.com".to_string(),
        src_auth: None,
        src_user: "external_user".to_string(),
        src_password: "external_pass".to_string(),
        src_folder: None,
        poll_time: None,
        fetchall: None,
        keep: None,
        protocol: None,
        usessl: None,
        sslcertck: None,
        extra_options: None,
        mda: None,
        active: None,
    };
    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.src_server, "imap.external.com");
    assert_eq!(resp.src_user, "external_user");

    let found = repo.find_by_id(resp.id).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn pg_log_create_and_find() {
    let (_container, pool) = common::setup_pg().await;
    let repo = PgLogRepository::new(pool);

    let dto = CreateLog {
        username: "admin@example.com".to_string(),
        domain: "example.com".to_string(),
        action: "create_domain".to_string(),
        data: Some("test data".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: None,
    };
    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.action, "create_domain");

    let filter = LogFilter {
        domain: Some("example.com".to_string()),
        username: None,
        action: None,
        from: None,
        until: None,
    };
    let page = PageRequest::new(1, 25);
    let page_resp = repo.find_all(&filter, &page).await.unwrap();
    assert_eq!(page_resp.items.len(), 1);
}

#[tokio::test]
async fn pg_app_password_create_and_find() {
    let (_container, pool) = common::setup_pg().await;
    let domain_repo = PgDomainRepository::new(pool.clone());
    let mailbox_repo = PgMailboxRepository::new(pool.clone());
    let repo = PgAppPasswordRepository::new(pool);

    let domain_dto = CreateDomain {
        domain: DomainName::new("app-pwd-test.com").unwrap(),
        description: None,
        aliases: None,
        mailboxes: None,
        maxquota: None,
        quota: None,
        transport: None,
        backupmx: None,
        password_expiry: None,
        active: None,
    };
    domain_repo.create(&domain_dto).await.unwrap();

    let mailbox_dto = CreateMailbox {
        username: EmailAddress::new("user@app-pwd-test.com").unwrap(),
        password: Password::new("securepass123").unwrap(),
        name: None,
        quota: None,
        active: None,
    };
    mailbox_repo.create(&mailbox_dto).await.unwrap();

    let dto = CreateAppPassword {
        username: EmailAddress::new("user@app-pwd-test.com").unwrap(),
        description: "Thunderbird".to_string(),
        password_hash: "$argon2id$hash".to_string(),
    };
    let resp = repo.create(&dto).await.unwrap();
    assert_eq!(resp.description, "Thunderbird");

    let username = EmailAddress::new("user@app-pwd-test.com").unwrap();
    let found = repo.find_by_username(&username).await.unwrap();
    assert_eq!(found.len(), 1);
}

#[tokio::test]
async fn pg_transaction_commit_persists_data() {
    let (_container, pool) = common::setup_pg().await;

    pg_transaction(&pool, |conn| {
        Box::pin(async move {
            sqlx::query("INSERT INTO domain (domain) VALUES ($1)")
                .bind("tx-commit.com")
                .execute(&mut *conn)
                .await?;
            Ok(())
        })
    })
    .await
    .unwrap();

    let repo = PgDomainRepository::new(pool);
    let name = DomainName::new("tx-commit.com").unwrap();
    let found = repo.find_by_name(&name).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn pg_transaction_error_rolls_back() {
    let (_container, pool) = common::setup_pg().await;

    let result: Result<(), DbError> = pg_transaction(&pool, |conn| {
        Box::pin(async move {
            sqlx::query("INSERT INTO domain (domain) VALUES ($1)")
                .bind("tx-rollback.com")
                .execute(&mut *conn)
                .await?;
            Err(DbError::not_found("test", "forced-error"))
        })
    })
    .await;

    assert!(result.is_err());

    let repo = PgDomainRepository::new(pool);
    let name = DomainName::new("tx-rollback.com").unwrap();
    let found = repo.find_by_name(&name).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn pg_transaction_multiple_operations_atomic() {
    let (_container, pool) = common::setup_pg().await;

    pg_transaction(&pool, |conn| {
        Box::pin(async move {
            sqlx::query("INSERT INTO domain (domain) VALUES ($1)")
                .bind("tx-multi.com")
                .execute(&mut *conn)
                .await?;
            sqlx::query("INSERT INTO alias (address, goto, domain) VALUES ($1, $2, $3)")
                .bind("info@tx-multi.com")
                .bind("admin@tx-multi.com")
                .bind("tx-multi.com")
                .execute(&mut *conn)
                .await?;
            Ok(())
        })
    })
    .await
    .unwrap();

    let domain_repo = PgDomainRepository::new(pool.clone());
    let alias_repo = PgAliasRepository::new(pool);
    let name = DomainName::new("tx-multi.com").unwrap();
    assert!(domain_repo.find_by_name(&name).await.unwrap().is_some());
    let addr = EmailAddress::new("info@tx-multi.com").unwrap();
    assert!(alias_repo.find_by_address(&addr).await.unwrap().is_some());
}

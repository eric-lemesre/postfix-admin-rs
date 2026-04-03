use sqlx::{MySqlPool, PgPool};
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::postgres::Postgres;

use postfix_admin_db::run_pg_migrations;

#[allow(dead_code, clippy::expect_used)]
pub async fn setup_pg() -> (ContainerAsync<Postgres>, PgPool) {
    let container = Postgres::default()
        .start()
        .await
        .expect("failed to start postgres container");

    let host = container
        .get_host()
        .await
        .expect("failed to get postgres host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get postgres port");

    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let pool = postfix_admin_db::create_pg_pool(&url, 5)
        .await
        .expect("failed to create pg pool");

    run_pg_migrations(&pool)
        .await
        .expect("failed to run pg migrations");

    (container, pool)
}

#[allow(dead_code, clippy::expect_used)]
pub async fn setup_mysql() -> (ContainerAsync<Mysql>, MySqlPool) {
    let container = Mysql::default()
        .start()
        .await
        .expect("failed to start mysql container");

    let host = container
        .get_host()
        .await
        .expect("failed to get mysql host");
    let port = container
        .get_host_port_ipv4(3306)
        .await
        .expect("failed to get mysql port");

    let url = format!("mysql://root@{host}:{port}/test");
    let pool = postfix_admin_db::create_mysql_pool(&url, 5)
        .await
        .expect("failed to create mysql pool");

    postfix_admin_db::run_mysql_migrations(&pool)
        .await
        .expect("failed to run mysql migrations");

    (container, pool)
}

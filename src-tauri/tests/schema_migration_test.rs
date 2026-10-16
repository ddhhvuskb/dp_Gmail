//! `storage::schema` 的版本与迁移测试。
//!
//! 覆盖 P0-07 引入的 `secret_encrypted` 列，以及 `PRAGMA user_version` 迁移机制。

use dp_gmail::storage::{init_schema, SCHEMA_VERSION};
use rusqlite::Connection;

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1"
        ))
        .expect("应能准备查询");
    stmt.query_row([column], |row| row.get::<_, i64>(0))
        .expect("应能执行查询")
        > 0
}

fn user_version(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("应能读取 user_version")
}

#[test]
fn fresh_database_is_stamped_with_current_version() {
    let mut conn = Connection::open_in_memory().expect("应能打开内存库");
    init_schema(&mut conn).expect("建表应成功");

    assert_eq!(user_version(&conn), SCHEMA_VERSION);
}

#[test]
fn fresh_database_has_encrypted_secret_column() {
    let mut conn = Connection::open_in_memory().expect("应能打开内存库");
    init_schema(&mut conn).expect("建表应成功");

    assert!(
        column_exists(&conn, "accounts", "secret_encrypted"),
        "全新库的 accounts 就应带上 secret_encrypted"
    );
}

#[test]
fn migrates_legacy_database_missing_the_column() {
    // 手工造一个 P0-03 时期的库：accounts 存在但没有 secret_encrypted，user_version = 0
    let mut conn = Connection::open_in_memory().expect("应能打开内存库");
    conn.execute_batch(
        "CREATE TABLE accounts (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             email        TEXT    NOT NULL UNIQUE,
             display_name TEXT,
             provider_id  TEXT    NOT NULL,
             created_at   INTEGER NOT NULL
         );",
    )
    .expect("应能造出旧库");

    assert!(!column_exists(&conn, "accounts", "secret_encrypted"));
    assert_eq!(user_version(&conn), 0);

    init_schema(&mut conn).expect("迁移应成功");

    assert!(
        column_exists(&conn, "accounts", "secret_encrypted"),
        "老库应被补上 secret_encrypted 列"
    );
    assert_eq!(user_version(&conn), SCHEMA_VERSION);
    assert!(
        column_exists(&conn, "messages", "uid"),
        "迁移同时应补齐其余缺失的表"
    );
}

#[test]
fn migration_is_idempotent() {
    let mut conn = Connection::open_in_memory().expect("应能打开内存库");
    conn.execute_batch(
        "CREATE TABLE accounts (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             email        TEXT    NOT NULL UNIQUE,
             display_name TEXT,
             provider_id  TEXT    NOT NULL,
             created_at   INTEGER NOT NULL
         );",
    )
    .expect("应能造出旧库");

    init_schema(&mut conn).expect("首次迁移");
    init_schema(&mut conn).expect("二次调用不应重复 ALTER");
    init_schema(&mut conn).expect("三次调用也不应失败");

    assert_eq!(user_version(&conn), SCHEMA_VERSION);
}

#[test]
fn preserves_legacy_rows_during_migration() {
    let mut conn = Connection::open_in_memory().expect("应能打开内存库");
    conn.execute_batch(
        "CREATE TABLE accounts (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             email        TEXT    NOT NULL UNIQUE,
             display_name TEXT,
             provider_id  TEXT    NOT NULL,
             created_at   INTEGER NOT NULL
         );
         INSERT INTO accounts (email, provider_id, created_at) VALUES ('old@qq.com', 'qq', 1);",
    )
    .expect("应能造出旧库并塞一行");

    init_schema(&mut conn).expect("迁移应成功");

    let (email, secret): (String, Option<Vec<u8>>) = conn
        .query_row(
            "SELECT email, secret_encrypted FROM accounts WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("老数据应还在");

    assert_eq!(email, "old@qq.com", "迁移不得丢数据");
    assert!(secret.is_none(), "老行的新列应为 NULL");
}

//! `storage` 模块（打开数据库 / PRAGMA / 级联）的单元测试。
//!
//! 覆盖 `docs/tasks/P0-03-sqlite-schema.md` §2 的 P0-03b。

use std::path::PathBuf;

use dp_gmail::storage::open_database;
use rusqlite::Connection;

/// 每个用例用独立的临时库文件，避免并发互相干扰。
fn temp_db_path(tag: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("dp_gmail_test_{tag}_{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .expect("应能统计行数")
}

fn seed_chain(conn: &Connection) {
    conn.execute(
        "INSERT INTO accounts (email, provider_id, created_at) VALUES ('a@qq.com', 'qq', 0)",
        [],
    )
    .expect("应能插入 account");
    conn.execute(
        "INSERT INTO folders (account_id, name) VALUES (1, 'INBOX')",
        [],
    )
    .expect("应能插入 folder");
    conn.execute(
        "INSERT INTO messages (folder_id, uid, date_unix, fetched_at) VALUES (1, 7, 0, 0)",
        [],
    )
    .expect("应能插入 message");
    conn.execute(
        "INSERT INTO attachments (message_id, filename, content_type, size) \
         VALUES (1, 'a.pdf', 'application/pdf', 3)",
        [],
    )
    .expect("应能插入 attachment");
}

#[test]
fn open_database_creates_schema() {
    let path = temp_db_path("creates");
    let conn = open_database(&path).expect("应能打开并建库");

    let tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .expect("应能统计表数量");
    assert_eq!(tables, 4, "应建出 P0 的四张表");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn enables_foreign_key_enforcement() {
    let path = temp_db_path("fk_on");
    let conn = open_database(&path).expect("应能开库");

    let enabled: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("应能读取 PRAGMA");
    assert_eq!(enabled, 1, "foreign_keys 必须为 ON，否则级联是空话");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn rejects_orphan_message() {
    let path = temp_db_path("orphan");
    let conn = open_database(&path).expect("应能开库");

    let err = conn
        .execute(
            "INSERT INTO messages (folder_id, uid, date_unix, fetched_at) VALUES (999, 1, 0, 0)",
            [],
        )
        .expect_err("引用不存在的 folder 应被外键拒绝");
    assert!(
        err.to_string().to_lowercase().contains("foreign key"),
        "应为外键约束错误，实际：{err}"
    );

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn cascades_delete_from_account_down_to_attachments() {
    let path = temp_db_path("cascade");
    let conn = open_database(&path).expect("应能开库");
    seed_chain(&conn);

    conn.execute("DELETE FROM accounts WHERE id = 1", [])
        .expect("应能删除 account");

    assert_eq!(count(&conn, "folders"), 0, "folder 应级联删除");
    assert_eq!(count(&conn, "messages"), 0, "message 应级联删除");
    assert_eq!(count(&conn, "attachments"), 0, "attachment 应级联删除");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn persists_across_reopen() {
    let path = temp_db_path("persist");

    {
        let conn = open_database(&path).expect("应能开库");
        seed_chain(&conn);
    }

    let conn = open_database(&path).expect("应能重新打开");
    assert_eq!(count(&conn, "messages"), 1, "重开后数据应仍在");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

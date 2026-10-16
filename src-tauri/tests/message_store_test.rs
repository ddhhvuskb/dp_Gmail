//! `storage::message_store` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-03-sqlite-schema.md` §2 的 P0-03c（UID 去重）。

use dp_gmail::storage::{open_database, upsert_message, NewAttachment, NewMessage};
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    dp_gmail::storage::init_schema(&mut conn).expect("建表应成功");
    conn.execute(
        "INSERT INTO accounts (email, provider_id, created_at) VALUES ('a@qq.com', 'qq', 0)",
        [],
    )
    .expect("应能插入 account");
    for name in ["INBOX", "Sent"] {
        conn.execute(
            "INSERT INTO folders (account_id, name) VALUES (1, ?1)",
            [name],
        )
        .expect("应能插入 folder");
    }
    conn
}

fn message(uid: i64, subject: &str) -> NewMessage {
    NewMessage {
        uid,
        message_id: Some(format!("<m{uid}@qq.com>")),
        subject: Some(subject.to_owned()),
        from_name: Some("Alice".to_owned()),
        from_address: Some("alice@qq.com".to_owned()),
        to_addresses: Some("bob@qq.com".to_owned()),
        date_unix: 1_700_000_000,
        text_body: Some("hello".to_owned()),
        html_body: None,
        attachments: vec![],
    }
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0))
        .expect("应能统计行数")
}

fn subject_of(conn: &Connection, id: i64) -> String {
    conn.query_row("SELECT subject FROM messages WHERE id = ?1", [id], |row| {
        row.get(0)
    })
    .expect("应能取到 subject")
}

#[test]
fn inserts_new_message() {
    let mut conn = memory_db();
    let id = upsert_message(&mut conn, 1, &message(42, "first")).expect("应能写入");

    assert!(id > 0);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM messages"), 1);
    assert_eq!(subject_of(&conn, id), "first");
}

#[test]
fn resyncing_same_uid_updates_instead_of_duplicating() {
    let mut conn = memory_db();
    let first_id = upsert_message(&mut conn, 1, &message(42, "first")).expect("首次写入");
    let second_id = upsert_message(&mut conn, 1, &message(42, "updated")).expect("再次写入");

    assert_eq!(first_id, second_id, "同一 UID 应复用同一行");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM messages"), 1);
    assert_eq!(subject_of(&conn, second_id), "updated");
}

#[test]
fn preserves_local_is_seen_across_resync() {
    let mut conn = memory_db();
    let id = upsert_message(&mut conn, 1, &message(42, "first")).expect("首次写入");
    conn.execute("UPDATE messages SET is_seen = 1 WHERE id = ?1", [id])
        .expect("应能标记已读");

    upsert_message(&mut conn, 1, &message(42, "updated")).expect("再次写入");

    let is_seen: i64 = conn
        .query_row("SELECT is_seen FROM messages WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .expect("应能读取 is_seen");
    assert_eq!(is_seen, 1, "同步不应覆盖本地已读状态");
}

#[test]
fn replaces_attachments_on_resync() {
    let mut conn = memory_db();

    let mut first = message(42, "first");
    first.attachments = vec![
        NewAttachment {
            filename: "old.txt".to_owned(),
            content_type: "text/plain".to_owned(),
            size: 3,
        },
        NewAttachment {
            filename: "stale.txt".to_owned(),
            content_type: "text/plain".to_owned(),
            size: 4,
        },
    ];
    let id = upsert_message(&mut conn, 1, &first).expect("首次写入");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM attachments"), 2);

    let mut second = message(42, "second");
    second.attachments = vec![NewAttachment {
        filename: "new.pdf".to_owned(),
        content_type: "application/pdf".to_owned(),
        size: 5,
    }];
    upsert_message(&mut conn, 1, &second).expect("再次写入");

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM attachments"), 1);
    let filename: String = conn
        .query_row(
            "SELECT filename FROM attachments WHERE message_id = ?1",
            [id],
            |row| row.get(0),
        )
        .expect("应能取到附件名");
    assert_eq!(filename, "new.pdf");
}

#[test]
fn allows_same_uid_in_different_folders() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(42, "inbox copy")).expect("INBOX 写入");
    upsert_message(&mut conn, 2, &message(42, "sent copy")).expect("Sent 写入");

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM messages"), 2);
}

#[test]
fn rolls_back_when_attachment_insert_fails() {
    let mut conn = memory_db();

    // 先占住 uid=42，制造一次注定失败的写入：附件表在事务内插入。
    upsert_message(&mut conn, 1, &message(42, "ok")).expect("首次写入");

    // 用一个非法 folder_id 触发外键失败（open_database 才开 FK，这里手动打开）
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("应能开启外键");

    let mut bad = message(99, "should roll back");
    bad.attachments = vec![NewAttachment {
        filename: "x".to_owned(),
        content_type: "text/plain".to_owned(),
        size: 1,
    }];
    let err = upsert_message(&mut conn, 999, &bad).expect_err("非法 folder_id 应失败");
    assert!(
        err.to_string().to_lowercase().contains("foreign key"),
        "应为外键错误，实际：{err}"
    );

    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM messages WHERE uid = 99"),
        0,
        "失败的写入必须整体回滚"
    );
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM attachments"),
        0,
        "附件也不应残留"
    );
}

#[test]
fn open_database_round_trip() {
    let mut path = std::env::temp_dir();
    path.push(format!("dp_gmail_store_{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);

    let mut conn = open_database(&path).expect("应能开库");
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

    let id = upsert_message(&mut conn, 1, &message(42, "persisted")).expect("应能写入");
    drop(conn);

    let conn = open_database(&path).expect("应能重开");
    assert_eq!(subject_of(&conn, id), "persisted");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

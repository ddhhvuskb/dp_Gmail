//! `sync_folder` 支撑层的单元测试。
//!
//! 覆盖 `ensure_folder` / `upsert_messages` / `load_account_credentials` /
//! `to_new_message` / `normalize_batch`。

use dp_gmail::commands::sync::{normalize_batch, MAX_SYNC_BATCH};
use dp_gmail::mail::parser::parse_message;
use dp_gmail::mail::sync::to_new_message;
use dp_gmail::storage::{
    create_account, ensure_folder, init_schema, load_account_credentials, upsert_messages,
    NewAttachment, NewMessage,
};
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    init_schema(&mut conn).expect("建表应成功");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("应能开启外键");
    conn
}

fn message(uid: i64, subject: &str) -> NewMessage {
    NewMessage {
        uid,
        message_id: None,
        subject: Some(subject.to_owned()),
        from_name: None,
        from_address: None,
        to_addresses: None,
        date_unix: 1_700_000_000,
        text_body: None,
        html_body: None,
        attachments: vec![],
    }
}

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .expect("应能统计")
}

// ── ensure_folder ────────────────────────────────────────────────

#[test]
fn ensure_folder_creates_then_reuses() {
    let conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");

    let first = ensure_folder(&conn, 1, "INBOX").expect("首次应建");
    let second = ensure_folder(&conn, 1, "INBOX").expect("再次应复用");

    assert_eq!(first, second, "同名文件夹必须复用同一行");
    assert_eq!(count(&conn, "folders"), 1);
}

#[test]
fn ensure_folder_isolates_by_account() {
    let conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");
    create_account(&conn, "b@qq.com", None, "qq", None).expect("建账户");

    let a = ensure_folder(&conn, 1, "INBOX").expect("账户 1");
    let b = ensure_folder(&conn, 2, "INBOX").expect("账户 2");

    assert_ne!(a, b, "不同账户的同名文件夹应是两行");
    assert_eq!(count(&conn, "folders"), 2);
}

// ── upsert_messages ──────────────────────────────────────────────

#[test]
fn upsert_messages_stores_whole_batch() {
    let mut conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");
    let folder = ensure_folder(&conn, 1, "INBOX").expect("建文件夹");

    let batch: Vec<NewMessage> = (1..=5).map(|uid| message(uid, "x")).collect();
    let stored = upsert_messages(&mut conn, folder, &batch).expect("批量写入");

    assert_eq!(stored, 5);
    assert_eq!(count(&conn, "messages"), 5);
}

#[test]
fn upsert_messages_is_idempotent_per_uid() {
    let mut conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");
    let folder = ensure_folder(&conn, 1, "INBOX").expect("建文件夹");

    let batch: Vec<NewMessage> = (1..=3).map(|uid| message(uid, "first")).collect();
    upsert_messages(&mut conn, folder, &batch).expect("首次");

    let batch2: Vec<NewMessage> = (1..=3).map(|uid| message(uid, "second")).collect();
    upsert_messages(&mut conn, folder, &batch2).expect("再次");

    assert_eq!(count(&conn, "messages"), 3, "同 UID 不应产生重复行");
    let subject: String = conn
        .query_row("SELECT subject FROM messages WHERE uid = 2", [], |row| {
            row.get(0)
        })
        .expect("应能读回");
    assert_eq!(subject, "second", "应更新为最新内容");
}

#[test]
fn upsert_messages_rolls_back_whole_batch_on_failure() {
    let mut conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");
    let folder = ensure_folder(&conn, 1, "INBOX").expect("建文件夹");

    // 批次里混入一封属于不存在文件夹的邮件：整批必须回滚
    let good: Vec<NewMessage> = (1..=3).map(|uid| message(uid, "ok")).collect();
    upsert_messages(&mut conn, folder, &good).expect("先写入 3 封");

    let mut bad = message(99, "boom");
    bad.attachments = vec![NewAttachment {
        filename: "x".to_owned(),
        content_type: "text/plain".to_owned(),
        size: 1,
    }];

    let err = upsert_messages(&mut conn, 999, std::slice::from_ref(&bad))
        .expect_err("非法 folder_id 应失败");
    assert!(
        err.to_string().to_lowercase().contains("foreign key"),
        "应为外键错误，实际：{err}"
    );

    assert_eq!(count(&conn, "messages"), 3, "失败的批次不得留下任何行");
    assert_eq!(count(&conn, "attachments"), 0);
}

// ── load_account_credentials ─────────────────────────────────────

#[test]
fn load_account_credentials_returns_identity_and_blob() {
    let conn = memory_db();
    let id =
        create_account(&conn, "a@qq.com", Some("主号"), "qq", Some(b"cipher")).expect("建账户");

    let loaded = load_account_credentials(&conn, id)
        .expect("查询不应报错")
        .expect("应命中");

    assert_eq!(loaded.email, "a@qq.com");
    assert_eq!(loaded.provider_id, "qq");
    assert_eq!(loaded.secret_encrypted.as_deref(), Some(&b"cipher"[..]));
}

#[test]
fn load_account_credentials_none_for_missing() {
    let conn = memory_db();
    assert!(load_account_credentials(&conn, 404)
        .expect("查询不应报错")
        .is_none());
}

// ── to_new_message ───────────────────────────────────────────────

#[test]
fn to_new_message_maps_parsed_fields() {
    let raw = b"Subject: hello\r\n\
From: Alice <alice@qq.com>\r\n\
To: a@x.com, b@y.com\r\n\
Date: Mon, 13 Oct 2025 10:00:00 +0800\r\n\
\r\n\
body text\r\n";
    let parsed = parse_message(raw).expect("应能解析");

    let new = to_new_message(42, &parsed);

    assert_eq!(new.uid, 42);
    assert_eq!(new.subject.as_deref(), Some("hello"));
    assert_eq!(new.from_name.as_deref(), Some("Alice"));
    assert_eq!(new.from_address.as_deref(), Some("alice@qq.com"));
    assert_eq!(new.to_addresses.as_deref(), Some("a@x.com, b@y.com"));
    assert_eq!(new.date_unix, 1_760_320_800);
    assert!(new.text_body.is_some());
}

#[test]
fn to_new_message_defaults_missing_date_to_zero() {
    let parsed = parse_message(b"Subject: no date\r\n\r\nbody\r\n").expect("应能解析");
    let new = to_new_message(1, &parsed);

    assert_eq!(new.date_unix, 0, "缺 Date 头时记 0，列表里排到最后");
    assert!(new.to_addresses.is_none());
}

#[test]
fn to_new_message_maps_attachment_sizes() {
    let raw = b"Subject: att\r\n\
Content-Type: multipart/mixed; boundary=\"B\"\r\n\
\r\n\
--B\r\n\
Content-Type: text/plain\r\n\
\r\n\
hi\r\n\
--B\r\n\
Content-Type: application/pdf; name=\"r.pdf\"\r\n\
Content-Disposition: attachment; filename=\"r.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
SGVsbG8=\r\n\
--B--\r\n";
    let parsed = parse_message(raw).expect("应能解析");
    let new = to_new_message(7, &parsed);

    assert_eq!(new.attachments.len(), 1);
    assert_eq!(new.attachments[0].filename, "r.pdf");
    assert_eq!(new.attachments[0].size, 5);
}

// ── normalize_batch ──────────────────────────────────────────────

#[test]
fn normalize_batch_clamps_to_bounds() {
    assert_eq!(normalize_batch(0), 1, "0 应归一为 1");
    assert_eq!(normalize_batch(u32::MAX), MAX_SYNC_BATCH);
    assert_eq!(normalize_batch(10), 10);
    assert_eq!(normalize_batch(MAX_SYNC_BATCH), MAX_SYNC_BATCH);
}

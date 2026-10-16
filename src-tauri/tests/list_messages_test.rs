//! `storage::list_messages` 的单元测试。

use dp_gmail::storage::{list_messages, upsert_message, NewMessage};
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    dp_gmail::storage::init_schema(&mut conn).expect("建表应成功");
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
    conn
}

fn message(uid: i64, subject: &str, date_unix: i64) -> NewMessage {
    NewMessage {
        uid,
        message_id: None,
        subject: Some(subject.to_owned()),
        from_name: Some("Alice".to_owned()),
        from_address: Some("alice@qq.com".to_owned()),
        to_addresses: None,
        date_unix,
        text_body: None,
        html_body: None,
        attachments: vec![],
    }
}

#[test]
fn returns_empty_for_unknown_folder() {
    let conn = memory_db();
    let rows = list_messages(&conn, 999, 50, 0).expect("应能查询");
    assert!(rows.is_empty());
}

#[test]
fn orders_by_date_descending() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "oldest", 1000)).expect("写入");
    upsert_message(&mut conn, 1, &message(2, "newest", 3000)).expect("写入");
    upsert_message(&mut conn, 1, &message(3, "middle", 2000)).expect("写入");

    let rows = list_messages(&conn, 1, 50, 0).expect("应能查询");
    let subjects: Vec<&str> = rows
        .iter()
        .map(|r| r.subject.as_deref().unwrap_or(""))
        .collect();
    assert_eq!(subjects, vec!["newest", "middle", "oldest"]);
}

#[test]
fn paginates_without_gaps_or_duplicates() {
    let mut conn = memory_db();
    // 故意让日期大量重复，检验次序键是否稳定
    for uid in 1..=10 {
        upsert_message(&mut conn, 1, &message(uid, &format!("m{uid}"), 5000)).expect("写入");
    }

    let first = list_messages(&conn, 1, 4, 0).expect("第一页");
    let second = list_messages(&conn, 1, 4, 4).expect("第二页");
    let third = list_messages(&conn, 1, 4, 8).expect("第三页");

    let mut ids: Vec<i64> = first
        .iter()
        .chain(second.iter())
        .chain(third.iter())
        .map(|r| r.id)
        .collect();
    assert_eq!(ids.len(), 10, "三页合计应恰好 10 条，不重不漏");

    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 10, "不应出现重复行");
}

#[test]
fn counts_attachments_per_message() {
    let mut conn = memory_db();

    let mut with_files = message(1, "has files", 2000);
    with_files.attachments = vec![
        dp_gmail::storage::NewAttachment {
            filename: "a.pdf".to_owned(),
            content_type: "application/pdf".to_owned(),
            size: 1,
        },
        dp_gmail::storage::NewAttachment {
            filename: "b.pdf".to_owned(),
            content_type: "application/pdf".to_owned(),
            size: 2,
        },
    ];
    upsert_message(&mut conn, 1, &with_files).expect("写入");
    upsert_message(&mut conn, 1, &message(2, "no files", 1000)).expect("写入");

    let rows = list_messages(&conn, 1, 50, 0).expect("应能查询");
    assert_eq!(rows[0].attachment_count, 2);
    assert_eq!(rows[1].attachment_count, 0);
}

#[test]
fn reflects_is_seen_flag() {
    let mut conn = memory_db();
    let id = upsert_message(&mut conn, 1, &message(1, "x", 1000)).expect("写入");
    conn.execute("UPDATE messages SET is_seen = 1 WHERE id = ?1", [id])
        .expect("标记已读");

    let rows = list_messages(&conn, 1, 50, 0).expect("应能查询");
    assert!(rows[0].is_seen);
}

#[test]
fn isolates_folders() {
    let mut conn = memory_db();
    conn.execute(
        "INSERT INTO folders (account_id, name) VALUES (1, 'Sent')",
        [],
    )
    .expect("应能插入第二个 folder");

    upsert_message(&mut conn, 1, &message(1, "inbox", 1000)).expect("写入");
    upsert_message(&mut conn, 2, &message(2, "sent", 2000)).expect("写入");

    let inbox = list_messages(&conn, 1, 50, 0).expect("应能查询");
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].subject.as_deref(), Some("inbox"));
}

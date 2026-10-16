//! `storage::load_message_detail` 的单元测试。

use dp_gmail::storage::{
    create_account, ensure_folder, init_schema, load_message_detail, upsert_message, NewAttachment,
    NewMessage,
};
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    init_schema(&mut conn).expect("建表应成功");
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");
    ensure_folder(&conn, 1, "INBOX").expect("建文件夹");
    conn
}

fn message(uid: i64) -> NewMessage {
    NewMessage {
        uid,
        message_id: Some(format!("<m{uid}@qq.com>")),
        subject: Some("主题".to_owned()),
        from_name: Some("Alice".to_owned()),
        from_address: Some("alice@qq.com".to_owned()),
        to_addresses: Some("bob@qq.com".to_owned()),
        date_unix: 1_700_000_000,
        text_body: Some("纯文本正文".to_owned()),
        html_body: Some("<p>HTML 正文</p>".to_owned()),
        attachments: vec![],
    }
}

#[test]
fn returns_none_for_missing_message() {
    let conn = memory_db();
    assert!(load_message_detail(&conn, 999)
        .expect("查询不应报错")
        .is_none());
}

#[test]
fn loads_all_detail_fields() {
    let mut conn = memory_db();
    let id = upsert_message(&mut conn, 1, &message(42)).expect("写入");

    let detail = load_message_detail(&conn, id)
        .expect("查询不应报错")
        .expect("应命中");

    assert_eq!(detail.id, id);
    assert_eq!(detail.uid, 42);
    assert_eq!(detail.subject.as_deref(), Some("主题"));
    assert_eq!(detail.from_name.as_deref(), Some("Alice"));
    assert_eq!(detail.from_address.as_deref(), Some("alice@qq.com"));
    assert_eq!(detail.to_addresses.as_deref(), Some("bob@qq.com"));
    assert_eq!(detail.date_unix, 1_700_000_000);
    assert!(!detail.is_seen);
    assert_eq!(detail.text_body.as_deref(), Some("纯文本正文"));
    // storage 层返回的是**原始** HTML，净化是 security 的职责
    assert_eq!(detail.html_body.as_deref(), Some("<p>HTML 正文</p>"));
}

#[test]
fn loads_attachments_ordered_by_id() {
    let mut conn = memory_db();
    let mut msg = message(7);
    msg.attachments = vec![
        NewAttachment {
            filename: "first.pdf".to_owned(),
            content_type: "application/pdf".to_owned(),
            size: 11,
        },
        NewAttachment {
            filename: "second.png".to_owned(),
            content_type: "image/png".to_owned(),
            size: 22,
        },
    ];
    let id = upsert_message(&mut conn, 1, &msg).expect("写入");

    let detail = load_message_detail(&conn, id)
        .expect("查询不应报错")
        .expect("应命中");

    let names: Vec<&str> = detail
        .attachments
        .iter()
        .map(|a| a.filename.as_str())
        .collect();
    assert_eq!(names, vec!["first.pdf", "second.png"]);
    assert_eq!(detail.attachments[0].size, 11);
    assert_eq!(detail.attachments[1].content_type, "image/png");
}

#[test]
fn reflects_updated_content_after_resync() {
    let mut conn = memory_db();
    let id = upsert_message(&mut conn, 1, &message(42)).expect("首次写入");

    let mut updated = message(42);
    updated.subject = Some("改过的主题".to_owned());
    updated.html_body = None;
    upsert_message(&mut conn, 1, &updated).expect("再次写入");

    let detail = load_message_detail(&conn, id)
        .expect("查询不应报错")
        .expect("应命中");

    assert_eq!(detail.subject.as_deref(), Some("改过的主题"));
    assert!(detail.html_body.is_none(), "重同步后正文应被更新");
}

#[test]
fn handles_null_bodies() {
    let mut conn = memory_db();
    let mut msg = message(1);
    msg.text_body = None;
    msg.html_body = None;
    let id = upsert_message(&mut conn, 1, &msg).expect("写入");

    let detail = load_message_detail(&conn, id)
        .expect("查询不应报错")
        .expect("应命中");

    assert!(detail.text_body.is_none());
    assert!(detail.html_body.is_none());
    assert!(detail.attachments.is_empty());
}

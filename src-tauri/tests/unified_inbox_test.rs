//! 多账户 / 统一收件箱的查询测试。

use dp_gmail::storage::{
    create_account, ensure_folder, init_schema, list_folders, list_unified_messages,
    upsert_message, NewMessage,
};
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库");
    init_schema(&mut conn).expect("建表");
    conn
}

fn message(uid: i64, subject: &str, date_unix: i64) -> NewMessage {
    NewMessage {
        uid,
        message_id: None,
        subject: Some(subject.to_owned()),
        from_name: None,
        from_address: Some("someone@qq.com".to_owned()),
        to_addresses: None,
        date_unix,
        text_body: None,
        html_body: None,
        attachments: vec![],
    }
}

/// 造两个账户，各自一个 INBOX 与一个 Sent，各塞若干邮件。
fn seed_two_accounts(conn: &mut Connection) {
    create_account(conn, "a@qq.com", Some("甲"), "qq", None).expect("账户 A");
    create_account(conn, "b@qq.com", Some("乙"), "qq", None).expect("账户 B");

    let a_inbox = ensure_folder(conn, 1, "INBOX").expect("A/INBOX");
    let a_sent = ensure_folder(conn, 1, "Sent").expect("A/Sent");
    let b_inbox = ensure_folder(conn, 2, "INBOX").expect("B/INBOX");

    upsert_message(conn, a_inbox, &message(1, "A 收件 旧", 1000)).expect("写入");
    upsert_message(conn, a_inbox, &message(2, "A 收件 新", 3000)).expect("写入");
    upsert_message(conn, a_sent, &message(3, "A 已发送", 2000)).expect("写入");
    upsert_message(conn, b_inbox, &message(4, "B 收件", 4000)).expect("写入");
}

fn subjects(hits: &[dp_gmail::storage::UnifiedMessageSummary]) -> Vec<&str> {
    hits.iter()
        .map(|m| m.subject.as_deref().unwrap_or(""))
        .collect()
}

// ── list_folders ────────────────────────────────────────────────

#[test]
fn lists_all_folders_when_account_is_none() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    let folders = list_folders(&conn, None).expect("查询");
    let names: Vec<&str> = folders.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["INBOX", "Sent", "INBOX"],
        "应按账户再按名称排序"
    );
    assert_eq!(folders.len(), 3);
}

#[test]
fn scopes_folders_to_one_account() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    let folders = list_folders(&conn, Some(1)).expect("查询");
    assert_eq!(folders.len(), 2);
    assert!(folders.iter().all(|f| f.account_id == 1));
}

#[test]
fn counts_messages_and_unread_per_folder() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    let folders = list_folders(&conn, Some(1)).expect("查询");
    let inbox = folders
        .iter()
        .find(|f| f.name == "INBOX")
        .expect("应有 INBOX");
    let sent = folders
        .iter()
        .find(|f| f.name == "Sent")
        .expect("应有 Sent");

    assert_eq!(inbox.message_count, 2);
    assert_eq!(inbox.unread_count, 2, "新建邮件默认未读");
    assert_eq!(sent.message_count, 1);

    // 标记一封已读后，未读数应变化而总数不变
    conn.execute(
        "UPDATE messages SET is_seen = 1 WHERE folder_id = ?1 AND uid = 1",
        [inbox.id],
    )
    .expect("标记已读");
    let folders = list_folders(&conn, Some(1)).expect("查询");
    let inbox = folders
        .iter()
        .find(|f| f.name == "INBOX")
        .expect("应有 INBOX");
    assert_eq!(inbox.message_count, 2);
    assert_eq!(inbox.unread_count, 1);
}

#[test]
fn returns_empty_for_account_without_folders() {
    let conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("账户");
    assert!(list_folders(&conn, Some(1)).expect("查询").is_empty());
}

// ── list_unified_messages ───────────────────────────────────────

#[test]
fn unifies_messages_across_accounts() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    let hits = list_unified_messages(&conn, None, None, 50, 0).expect("查询");
    assert_eq!(
        subjects(&hits),
        vec!["B 收件", "A 收件 新", "A 已发送", "A 收件 旧"],
        "应跨账户跨文件夹按时间倒序合并"
    );
}

#[test]
fn carries_source_folder_and_account() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    let hits = list_unified_messages(&conn, None, None, 50, 0).expect("查询");
    let newest = &hits[0];

    assert_eq!(newest.folder_name, "INBOX");
    assert_eq!(newest.account_email, "b@qq.com");
    assert_eq!(newest.account_id, 2);
    assert_eq!(newest.folder_id, 3);
}

#[test]
fn scopes_to_one_account() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    let hits = list_unified_messages(&conn, Some(1), None, 50, 0).expect("查询");
    assert_eq!(subjects(&hits), vec!["A 收件 新", "A 已发送", "A 收件 旧"]);
}

#[test]
fn scopes_to_inbox_across_accounts() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    // 典型的「统一收件箱」：各账户 INBOX 合并，排除 Sent
    let hits = list_unified_messages(&conn, None, Some("INBOX"), 50, 0).expect("查询");
    assert_eq!(subjects(&hits), vec!["B 收件", "A 收件 新", "A 收件 旧"]);
    assert!(hits.iter().all(|m| m.folder_name == "INBOX"));
}

#[test]
fn mailbox_filter_is_case_insensitive() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    for spelling in ["INBOX", "inbox", "InBoX"] {
        let hits = list_unified_messages(&conn, None, Some(spelling), 50, 0).expect("查询");
        assert_eq!(hits.len(), 3, "[{spelling}] 应命中同样结果");
    }
}

#[test]
fn combines_account_and_mailbox_filters() {
    let mut conn = memory_db();
    seed_two_accounts(&mut conn);

    let hits = list_unified_messages(&conn, Some(1), Some("INBOX"), 50, 0).expect("查询");
    assert_eq!(subjects(&hits), vec!["A 收件 新", "A 收件 旧"]);
}

#[test]
fn paginates_unified_view_without_gaps() {
    let mut conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("账户");
    let folder = ensure_folder(&conn, 1, "INBOX").expect("文件夹");
    for uid in 1..=10 {
        upsert_message(&mut conn, folder, &message(uid, "x", 5000)).expect("写入");
    }

    let first = list_unified_messages(&conn, None, None, 4, 0).expect("第一页");
    let second = list_unified_messages(&conn, None, None, 4, 4).expect("第二页");
    let third = list_unified_messages(&conn, None, None, 4, 8).expect("第三页");

    let mut ids: Vec<i64> = first
        .iter()
        .chain(second.iter())
        .chain(third.iter())
        .map(|m| m.id)
        .collect();
    assert_eq!(ids.len(), 10, "三页合计应恰好 10 条");

    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 10, "不应有重复行");
}

#[test]
fn returns_empty_on_empty_database() {
    let conn = memory_db();
    assert!(list_unified_messages(&conn, None, None, 50, 0)
        .expect("查询")
        .is_empty());
}

#[test]
fn counts_attachments_in_unified_view() {
    let mut conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("账户");
    let folder = ensure_folder(&conn, 1, "INBOX").expect("文件夹");

    let mut with_attachment = message(1, "带附件", 2000);
    with_attachment.attachments = vec![dp_gmail::storage::NewAttachment {
        filename: "a.pdf".to_owned(),
        content_type: "application/pdf".to_owned(),
        size: 9,
    }];
    upsert_message(&mut conn, folder, &with_attachment).expect("写入");
    upsert_message(&mut conn, folder, &message(2, "无附件", 1000)).expect("写入");

    let hits = list_unified_messages(&conn, None, None, 50, 0).expect("查询");
    assert_eq!(hits[0].attachment_count, 1);
    assert_eq!(hits[1].attachment_count, 0);
}

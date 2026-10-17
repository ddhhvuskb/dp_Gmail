//! 本地「已发送」归档的存储层测试。
//!
//! 守的是用户报的那条缺陷：**邮件发出去了，但应用里没有记录**。
//! `send_mail` 只走 SMTP，投递成功后本地什么都不留；
//! `record_sent` 命令负责补一份本地归档，本文件测它用到的那几个存储动作。

use dp_gmail::storage::{
    create_account, ensure_folder, init_schema, list_messages, mark_seen, upsert_message, NewMessage,
};
use rusqlite::Connection;

const SENT_FOLDER: &str = "已发送";

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    conn.execute_batch("PRAGMA foreign_keys = ON;").expect("应能开外键");
    init_schema(&mut conn).expect("建表应成功");
    conn
}

fn sent_message(uid: i64, to: &str) -> NewMessage {
    NewMessage {
        uid,
        message_id: None,
        subject: Some("测试主题".to_owned()),
        from_name: None,
        from_address: Some("me@163.com".to_owned()),
        to_addresses: Some(to.to_owned()),
        date_unix: 1_700_000_000,
        text_body: Some("正文".to_owned()),
        html_body: None,
        attachments: Vec::new(),
    }
}

#[test]
fn sent_message_lands_in_the_sent_folder_and_is_readable() {
    let mut conn = memory_db();
    let account = create_account(&conn, "me@163.com", None, "netease163", None).expect("建账户");

    let folder = ensure_folder(&conn, account, SENT_FOLDER).expect("建已发送文件夹");
    let id = upsert_message(&mut conn, folder, &sent_message(1, "you@qq.com")).expect("写入");

    let rows = list_messages(&conn, folder, 50, 0).expect("应能列出");
    assert_eq!(rows.len(), 1, "已发送里应该有这一封");
    assert_eq!(rows[0].id, id);
    assert_eq!(rows[0].subject.as_deref(), Some("测试主题"));
    assert_eq!(
        rows[0].from_address.as_deref(),
        Some("me@163.com"),
        "发件人应是账户本身"
    );
}

#[test]
fn sent_folder_is_reused_not_duplicated() {
    let mut conn = memory_db();
    let account = create_account(&conn, "me@163.com", None, "netease163", None).expect("建账户");

    let first = ensure_folder(&conn, account, SENT_FOLDER).expect("第一次");
    let second = ensure_folder(&conn, account, SENT_FOLDER).expect("第二次");
    assert_eq!(first, second, "同名文件夹应复用，不能每次发信都建一个");
}

#[test]
fn mark_seen_clears_the_unread_flag() {
    let mut conn = memory_db();
    let account = create_account(&conn, "me@163.com", None, "netease163", None).expect("建账户");
    let folder = ensure_folder(&conn, account, SENT_FOLDER).expect("建文件夹");
    let id = upsert_message(&mut conn, folder, &sent_message(1, "you@qq.com")).expect("写入");

    // 刚写进去默认未读
    let before = list_messages(&conn, folder, 50, 0).expect("列出");
    assert!(!before[0].is_seen, "刚写入时默认未读");

    mark_seen(&conn, id).expect("标记已读");

    let after = list_messages(&conn, folder, 50, 0).expect("列出");
    assert!(after[0].is_seen, "自己发的信必须立刻是已读，否则发件箱永远挂着未读");
}

#[test]
fn uid_increments_within_the_sent_folder() {
    let mut conn = memory_db();
    let account = create_account(&conn, "me@163.com", None, "netease163", None).expect("建账户");
    let folder = ensure_folder(&conn, account, SENT_FOLDER).expect("建文件夹");

    upsert_message(&mut conn, folder, &sent_message(1, "a@qq.com")).expect("第一封");
    upsert_message(&mut conn, folder, &sent_message(2, "b@qq.com")).expect("第二封");

    let rows = list_messages(&conn, folder, 50, 0).expect("列出");
    assert_eq!(rows.len(), 2, "两封都在，说明 uid 没有撞车");
}

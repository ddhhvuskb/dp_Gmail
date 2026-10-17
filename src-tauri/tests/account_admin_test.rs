//! 账户「编辑 / 删除」的单元测试。
//!
//! 重点：
//! 1. 删除必须**连带**清掉文件夹、邮件、附件与 FTS 索引 —— 靠 `ON DELETE CASCADE`，
//!    所以测试库必须显式 `PRAGMA foreign_keys = ON`，否则级联静默失效、
//!    测试会「通过」但真实库里留一堆孤儿。
//! 2. 编辑不传授权码时**绝不能**把已有凭证写坏。

use dp_gmail::storage::{
    create_account, delete_account, ensure_folder, init_schema, list_accounts,
    load_encrypted_secret, update_account, NewAttachment, NewMessage,
};
use rusqlite::Connection;

const SECRET_A: &[u8] = &[1, 2, 3, 4];
const SECRET_B: &[u8] = &[9, 9, 9];

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    // 与 storage::open_database 保持一致：事务外打开外键约束
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("应能打开外键约束");
    init_schema(&mut conn).expect("建表应成功");
    conn
}

fn demo_message(uid: i64) -> NewMessage {
    NewMessage {
        uid,
        message_id: Some(format!("<m{uid}@test>")),
        subject: Some("测试主题".to_owned()),
        from_name: Some("发件人".to_owned()),
        from_address: Some("from@qq.com".to_owned()),
        to_addresses: Some("to@qq.com".to_owned()),
        date_unix: 1_700_000_000,
        text_body: Some("正文".to_owned()),
        html_body: None,
        attachments: vec![NewAttachment {
            filename: "a.txt".to_owned(),
            content_type: "text/plain".to_owned(),
            size: 3,
        }],
    }
}

/// 给账户造一封带附件的邮件，返回 `messages.id`。
fn seed_mail(conn: &mut Connection, account_id: i64, uid: i64) -> i64 {
    let folder_id = ensure_folder(conn, account_id, "INBOX").expect("建文件夹应成功");
    dp_gmail::storage::upsert_message(conn, folder_id, &demo_message(uid)).expect("写入邮件应成功")
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).expect("计数应成功")
}

#[test]
fn delete_removes_account_and_everything_below_it() {
    let mut conn = memory_db();
    let account_id = create_account(&conn, "a@qq.com", Some("主号"), "qq", Some(SECRET_A))
        .expect("建账户应成功");
    seed_mail(&mut conn, account_id, 1);
    seed_mail(&mut conn, account_id, 2);

    let impact = delete_account(&mut conn, account_id)
        .expect("删除应成功")
        .expect("账户存在，应返回影响面");

    assert_eq!(impact.messages, 2, "应报告删掉 2 封邮件");
    assert_eq!(impact.attachments, 2, "应报告删掉 2 个附件");

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM accounts"), 0);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM folders"), 0, "文件夹应级联删除");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM messages"), 0, "邮件应级联删除");
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM attachments"),
        0,
        "附件应级联删除"
    );
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM messages_fts"),
        0,
        "FTS 索引应由触发器跟着删"
    );
}

#[test]
fn delete_unknown_account_reports_none_and_deletes_nothing() {
    let mut conn = memory_db();
    let keep = create_account(&conn, "keep@qq.com", None, "qq", Some(SECRET_A))
        .expect("建账户应成功");
    seed_mail(&mut conn, keep, 1);

    let impact = delete_account(&mut conn, 999).expect("查询不应报错");
    assert!(impact.is_none(), "不存在的账户应返回 None");

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM accounts"), 1);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM messages"), 1);
}

#[test]
fn delete_only_touches_the_targeted_account() {
    let mut conn = memory_db();
    let gone = create_account(&conn, "gone@qq.com", None, "qq", Some(SECRET_A))
        .expect("建账户应成功");
    let keep = create_account(&conn, "keep@qq.com", None, "qq", Some(SECRET_A))
        .expect("建账户应成功");
    seed_mail(&mut conn, gone, 1);
    seed_mail(&mut conn, keep, 1);

    delete_account(&mut conn, gone).expect("删除应成功");

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM accounts"), 1);
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM messages"),
        1,
        "另一个账户的邮件不能被牵连"
    );
    let remaining = list_accounts(&conn).expect("列表应成功");
    assert_eq!(remaining[0].id, keep);
}

#[test]
fn update_renames_without_touching_the_secret() {
    let conn = memory_db();
    let id = create_account(&conn, "a@qq.com", Some("旧名"), "qq", Some(SECRET_A))
        .expect("建账户应成功");

    let found = update_account(&conn, id, "a@qq.com", Some("新名"), "qq", None).expect("更新应成功");
    assert!(found);

    let name: Option<String> = conn
        .query_row("SELECT display_name FROM accounts WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .expect("应能读回");
    assert_eq!(name.as_deref(), Some("新名"));
    assert_eq!(
        load_encrypted_secret(&conn, id).expect("应能读凭证"),
        Some(SECRET_A.to_vec()),
        "没传新授权码时，已有凭证必须原样保留"
    );
}

#[test]
fn update_replaces_the_secret_when_given() {
    let conn = memory_db();
    let id = create_account(&conn, "a@qq.com", None, "qq", Some(SECRET_A)).expect("建账户应成功");

    update_account(&conn, id, "a@qq.com", None, "qq", Some(SECRET_B)).expect("更新应成功");

    assert_eq!(
        load_encrypted_secret(&conn, id).expect("应能读凭证"),
        Some(SECRET_B.to_vec())
    );
}

#[test]
fn update_changes_email_and_provider_together() {
    let conn = memory_db();
    let id = create_account(&conn, "typo@qq.con", None, "qq", Some(SECRET_A))
        .expect("建账户应成功");

    // 这就是用户报的场景：邮箱打错了，改对时服务商要跟着重算
    update_account(&conn, id, "right@163.com", None, "netease163", None).expect("更新应成功");

    let (email, provider): (String, String) = conn
        .query_row("SELECT email, provider_id FROM accounts WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .expect("应能读回");
    assert_eq!(email, "right@163.com");
    assert_eq!(provider, "netease163");
}

#[test]
fn update_reports_false_for_missing_account() {
    let conn = memory_db();
    let found = update_account(&conn, 999, "a@qq.com", None, "qq", None).expect("查询不应报错");
    assert!(!found, "不存在的账户应返回 false");
}

#[test]
fn update_rejects_duplicate_email() {
    let conn = memory_db();
    create_account(&conn, "taken@qq.com", None, "qq", Some(SECRET_A)).expect("首个应成功");
    let id = create_account(&conn, "mine@qq.com", None, "qq", Some(SECRET_A)).expect("第二个应成功");

    let err = update_account(&conn, id, "taken@qq.com", None, "qq", None)
        .expect_err("改成已存在的邮箱应被拒");
    assert!(
        err.to_string().to_lowercase().contains("unique"),
        "应为唯一约束冲突，实际：{err}"
    );
}

/// 级联删除依赖 `PRAGMA foreign_keys`。这条测试守的就是它。
///
/// 原本想写「不开 pragma 就会留孤儿」的阴性对照，**实测发现构造不出来**：
/// rusqlite 的 `bundled` SQLite 编译期带 `SQLITE_DEFAULT_FOREIGN_KEYS=1`，
/// 新建连接**默认就是 1**（实测 SQLite 3.53.2，未设置时查询得 1）。
///
/// 所以改成断言真实的不变量 —— 这条测试会在下列情况下变红，从而提醒复核：
/// - 有人把 `rusqlite` 从 `bundled` 改成系统库（默认值可能变成 0，级联静默失效）
/// - 有人删掉 `open_database` 里那句 `PRAGMA foreign_keys = ON`
///
/// 换句话说：**「默认就是开的」是当前构建的偶然事实，不是 SQLite 的承诺。**
#[test]
fn app_connections_always_have_foreign_keys_on() {
    let dir = std::env::temp_dir().join(format!("dp_gmail_fk_{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&dir);

    let conn = dp_gmail::storage::open_database(&dir).expect("应能打开文件库");

    let fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("应能查询 pragma");
    assert_eq!(fk, 1, "应用连接必须开着外键约束，否则级联删除会静默失效");

    drop(conn);
    let _ = std::fs::remove_file(&dir);
    let _ = std::fs::remove_file(dir.with_extension("sqlite-wal"));
    let _ = std::fs::remove_file(dir.with_extension("sqlite-shm"));
}

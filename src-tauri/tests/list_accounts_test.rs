//! `storage::list_accounts` 的单元测试。

use dp_gmail::storage::{create_account, init_schema, list_accounts};
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    init_schema(&mut conn).expect("建表应成功");
    conn
}

#[test]
fn returns_empty_when_no_accounts() {
    let conn = memory_db();
    assert!(list_accounts(&conn).expect("查询不应报错").is_empty());
}

#[test]
fn lists_accounts_in_creation_order() {
    let conn = memory_db();
    create_account(&conn, "a@qq.com", Some("主号"), "qq", Some(b"cipher")).expect("建账户");
    create_account(&conn, "b@qq.com", None, "qq", None).expect("建账户");

    let accounts = list_accounts(&conn).expect("查询不应报错");

    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].email, "a@qq.com");
    assert_eq!(accounts[0].display_name.as_deref(), Some("主号"));
    assert_eq!(accounts[1].email, "b@qq.com");
    assert!(accounts[0].id < accounts[1].id, "应按创建顺序");
}

#[test]
fn reports_has_secret_without_exposing_the_blob() {
    let conn = memory_db();
    create_account(&conn, "with@qq.com", None, "qq", Some(b"SECRET-CIPHERTEXT")).expect("建账户");
    create_account(&conn, "without@qq.com", None, "qq", None).expect("建账户");

    let accounts = list_accounts(&conn).expect("查询不应报错");

    assert!(accounts[0].has_secret);
    assert!(!accounts[1].has_secret);
}

#[test]
fn reflects_deleted_accounts() {
    let conn = memory_db();
    let id = create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");
    assert_eq!(list_accounts(&conn).expect("查询").len(), 1);

    conn.execute("DELETE FROM accounts WHERE id = ?1", [id])
        .expect("应能删除");

    assert!(list_accounts(&conn).expect("查询").is_empty());
}

#[test]
fn carries_provider_id_through() {
    let conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");

    let accounts = list_accounts(&conn).expect("查询不应报错");
    assert_eq!(accounts[0].provider_id, "qq");
}

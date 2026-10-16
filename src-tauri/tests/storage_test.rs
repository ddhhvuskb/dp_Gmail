//! `storage::schema` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-03-sqlite-schema.md` §6 约定的用例。

use dp_gmail::storage::init_schema;
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    init_schema(&mut conn).expect("建表应成功");
    conn
}

fn table_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .expect("应能准备查询");
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .expect("应能执行查询");
    rows.map(|r| r.expect("应能取到表名")).collect()
}

fn seed_account_and_folder(conn: &Connection) {
    conn.execute(
        "INSERT INTO accounts (email, provider_id, created_at) VALUES (?1, 'qq', 0)",
        ["a@qq.com"],
    )
    .expect("应能插入 account");
    conn.execute(
        "INSERT INTO folders (account_id, name) VALUES (1, 'INBOX')",
        [],
    )
    .expect("应能插入 folder");
}

#[test]
fn creates_all_p0_tables() {
    let conn = memory_db();
    let names = table_names(&conn);

    // 用「包含」而非「等于」：P1 之后 sqlite_master 里还会多出
    // messages_fts 及其 4 张影子表（_data/_idx/_docsize/_config），
    // 断言精确列表会跟着 FTS 实现细节一起碎掉。
    for expected in ["accounts", "attachments", "folders", "messages"] {
        assert!(
            names.iter().any(|name| name == expected),
            "缺少表 {expected}：{names:?}"
        );
    }
}

#[test]
fn creates_fts_index_table() {
    let conn = memory_db();
    assert!(
        table_names(&conn).iter().any(|name| name == "messages_fts"),
        "P1 的全文检索虚表应随建表一起创建"
    );
}

#[test]
fn creates_expected_indexes() {
    let conn = memory_db();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_%' ORDER BY name")
        .expect("应能准备查询");
    let names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .expect("应能执行查询")
        .map(|r| r.expect("应能取到索引名"))
        .collect();

    assert_eq!(
        names,
        vec!["idx_attachments_message", "idx_messages_folder_date"]
    );
}

#[test]
fn init_schema_is_idempotent() {
    let mut conn = memory_db();
    let before = table_names(&conn).len();

    init_schema(&mut conn).expect("重复建表不应失败");
    init_schema(&mut conn).expect("第三次建表也不应失败");

    assert_eq!(table_names(&conn).len(), before, "重复调用不得新增任何表");
}

#[test]
fn rejects_duplicate_uid_within_same_folder() {
    let conn = memory_db();
    seed_account_and_folder(&conn);

    let insert = "INSERT INTO messages (folder_id, uid, date_unix, fetched_at) \
                  VALUES (1, 42, 1700000000, 1700000000)";
    conn.execute(insert, []).expect("首个 UID 应能插入");

    let err = conn
        .execute(insert, [])
        .expect_err("同一文件夹内重复 UID 应被唯一索引拒绝");
    assert!(
        err.to_string().to_lowercase().contains("unique"),
        "应为唯一约束冲突，实际：{err}"
    );
}

#[test]
fn allows_same_uid_in_different_folders() {
    let conn = memory_db();
    seed_account_and_folder(&conn);
    conn.execute(
        "INSERT INTO folders (account_id, name) VALUES (1, 'Sent')",
        [],
    )
    .expect("应能插入第二个 folder");

    let insert = "INSERT INTO messages (folder_id, uid, date_unix, fetched_at) \
                  VALUES (?1, 42, 1700000000, 1700000000)";
    conn.execute(insert, [1]).expect("INBOX 中插入 UID 42");
    conn.execute(insert, [2])
        .expect("另一文件夹中的同号 UID 应被允许");
}

#[test]
fn rejects_duplicate_account_email() {
    let conn = memory_db();
    let insert = "INSERT INTO accounts (email, provider_id, created_at) VALUES (?1, 'qq', 0)";
    conn.execute(insert, ["a@qq.com"])
        .expect("首个邮箱应能插入");

    let err = conn
        .execute(insert, ["a@qq.com"])
        .expect_err("重复邮箱应被唯一索引拒绝");
    assert!(
        err.to_string().to_lowercase().contains("unique"),
        "应为唯一约束冲突，实际：{err}"
    );
}

#[test]
fn rejects_duplicate_folder_name_within_account() {
    let conn = memory_db();
    seed_account_and_folder(&conn);

    let err = conn
        .execute(
            "INSERT INTO folders (account_id, name) VALUES (1, 'INBOX')",
            [],
        )
        .expect_err("同一账户下重复文件夹名应被唯一索引拒绝");
    assert!(
        err.to_string().to_lowercase().contains("unique"),
        "应为唯一约束冲突，实际：{err}"
    );
}

#[test]
fn stores_timestamps_as_integers() {
    let conn = memory_db();
    // date_unix 列声明为 INTEGER；写入非整数应被 SQLite 类型亲和性转为整数或报错，
    // 此处直接核对列声明类型，防止后续被改成 TEXT/DATETIME。
    let mut stmt = conn
        .prepare("SELECT type FROM pragma_table_info('messages') WHERE name = 'date_unix'")
        .expect("应能准备查询");
    let col_type: String = stmt
        .query_row([], |row| row.get(0))
        .expect("应能取到列类型");
    assert_eq!(col_type, "INTEGER");
}

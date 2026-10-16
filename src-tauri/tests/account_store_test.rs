//! `storage::account_store` 的单元测试。
//!
//! 重点验证「明文密码不落库」这条安全性质 —— 用真实的加密/解密穿一遍全链路。

use dp_gmail::security::{decrypt_secret, encrypt_secret, KEY_LEN};
use dp_gmail::storage::{create_account, init_schema, load_encrypted_secret};
use rusqlite::types::Value;
use rusqlite::Connection;

const PLAINTEXT: &str = "qq-auth-code-abcdef";

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库应能打开");
    init_schema(&mut conn).expect("建表应成功");
    conn
}

fn demo_key() -> [u8; KEY_LEN] {
    [0x42; KEY_LEN]
}

#[test]
fn creates_account_and_returns_id() {
    let conn = memory_db();
    let id = create_account(&conn, "a@qq.com", Some("主号"), "qq", None).expect("应能建账户");

    assert!(id > 0);
    let (email, provider): (String, String) = conn
        .query_row(
            "SELECT email, provider_id FROM accounts WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("应能读回");
    assert_eq!(email, "a@qq.com");
    assert_eq!(provider, "qq");
}

#[test]
fn rejects_duplicate_email() {
    let conn = memory_db();
    create_account(&conn, "a@qq.com", None, "qq", None).expect("首个应成功");

    let err = create_account(&conn, "a@qq.com", None, "qq", None).expect_err("重复邮箱应被拒");
    assert!(
        err.to_string().to_lowercase().contains("unique"),
        "应为唯一约束冲突，实际：{err}"
    );
}

#[test]
fn returns_none_for_missing_account_or_secret() {
    let conn = memory_db();

    assert!(
        load_encrypted_secret(&conn, 999)
            .expect("查询不应报错")
            .is_none(),
        "账户不存在应返回 None"
    );

    let id = create_account(&conn, "a@qq.com", None, "qq", None).expect("应能建账户");
    assert!(
        load_encrypted_secret(&conn, id)
            .expect("查询不应报错")
            .is_none(),
        "未设置凭证应返回 None"
    );
}

#[test]
fn encrypted_secret_round_trips_through_the_database() {
    let conn = memory_db();
    let key = demo_key();

    let blob = encrypt_secret(PLAINTEXT, &key).expect("应能加密");
    let id = create_account(&conn, "a@qq.com", None, "qq", Some(&blob)).expect("应能建账户");

    let loaded = load_encrypted_secret(&conn, id)
        .expect("查询不应报错")
        .expect("应有密文");

    assert_eq!(loaded, blob, "存进去和读出来应一致");
    assert_eq!(
        decrypt_secret(&loaded, &key).expect("应能解密"),
        PLAINTEXT,
        "全链路应能还原出原始授权码"
    );
}

#[test]
fn stored_blob_is_not_the_plaintext() {
    let conn = memory_db();
    let key = demo_key();

    let blob = encrypt_secret(PLAINTEXT, &key).expect("应能加密");
    let id = create_account(&conn, "a@qq.com", None, "qq", Some(&blob)).expect("应能建账户");

    // 逐列扫一遍整行，明文不得出现在任何列里
    let mut stmt = conn
        .prepare("SELECT * FROM accounts WHERE id = ?1")
        .expect("应能准备查询");
    let column_count = stmt.column_count();

    let mut rows = stmt.query([id]).expect("应能执行查询");
    let row = rows.next().expect("应有一行").expect("应能取行");

    for index in 0..column_count {
        // 各列类型不同（INTEGER / TEXT / BLOB / NULL），必须分类型取字节后比对。
        let cell: Value = row.get(index).expect("应能取单元格");
        let bytes = match cell {
            Value::Null => continue,
            Value::Integer(value) => value.to_le_bytes().to_vec(),
            Value::Real(value) => value.to_le_bytes().to_vec(),
            Value::Text(value) => value.into_bytes(),
            Value::Blob(value) => value,
        };

        assert!(
            !bytes
                .windows(PLAINTEXT.len())
                .any(|w| w == PLAINTEXT.as_bytes()),
            "第 {index} 列里出现了明文密码"
        );
    }
}

#[test]
fn wrong_key_cannot_open_stored_secret() {
    let conn = memory_db();
    let blob = encrypt_secret(PLAINTEXT, &demo_key()).expect("应能加密");
    let id = create_account(&conn, "a@qq.com", None, "qq", Some(&blob)).expect("应能建账户");

    let loaded = load_encrypted_secret(&conn, id)
        .expect("查询不应报错")
        .expect("应有密文");

    assert!(
        decrypt_secret(&loaded, &[0x43; KEY_LEN]).is_err(),
        "换一把密钥必须解不开"
    );
}

//! 表结构定义。
//!
//! 时间字段一律 `INTEGER` 存 **Unix 秒**（`DESIGN.md` P0-03 约束）。
//! 跨表引用一律 `ON DELETE CASCADE`。禁止引入 ORM。

use rusqlite::Connection;

use super::error::StorageError;

/// P0 全部 DDL。全部 `IF NOT EXISTS`，可重复执行。
///
/// 注意：`PRAGMA foreign_keys` **不能**放在这里 —— 它在事务内是空操作，
/// 必须在事务外设置（见 `storage::open_database`）。
const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS accounts (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    email            TEXT    NOT NULL UNIQUE,
    display_name     TEXT,
    provider_id      TEXT    NOT NULL,
    created_at       INTEGER NOT NULL,
    -- P0-07：AES-256-GCM 密文，布局 nonce(12) || ciphertext || tag(16)。
    -- 明文密码绝不允许出现在这里或任何其他列（AGENTS.md 禁止项）。
    secret_encrypted BLOB
);

CREATE TABLE IF NOT EXISTS folders (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    uid_validity INTEGER,
    uid_next     INTEGER,
    last_sync_at INTEGER,
    UNIQUE (account_id, name)
);

CREATE TABLE IF NOT EXISTS messages (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_id    INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    uid          INTEGER NOT NULL,
    message_id   TEXT,
    subject      TEXT,
    from_name    TEXT,
    from_address TEXT,
    to_addresses TEXT,
    date_unix    INTEGER NOT NULL,
    text_body    TEXT,
    html_body    TEXT,
    is_seen      INTEGER NOT NULL DEFAULT 0,
    fetched_at   INTEGER NOT NULL,
    UNIQUE (folder_id, uid)
);

CREATE TABLE IF NOT EXISTS attachments (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id   INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename     TEXT    NOT NULL,
    content_type TEXT    NOT NULL,
    size         INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_folder_date ON messages (folder_id, date_unix DESC);
CREATE INDEX IF NOT EXISTS idx_attachments_message  ON attachments (message_id);
";

/// 当前 schema 版本，存在 SQLite 内建的 `PRAGMA user_version` 里。
///
/// - `1`：P0-03 的初始四表
/// - `2`：P0-07，`accounts` 增加 `secret_encrypted`
pub const SCHEMA_VERSION: i64 = 2;

/// 逐版本迁移语句，**只对已存在的库**执行。
///
/// 全新库不走这里 —— `CREATE TABLE` 本身就是最新结构，
/// 再补一次 `ALTER TABLE` 会因为列已存在而报错。
const MIGRATIONS: &[(i64, &str)] = &[(2, "ALTER TABLE accounts ADD COLUMN secret_encrypted BLOB;")];

/// 在**一个事务**内建表建索引，并按 `user_version` 补齐历史库缺的列。
///
/// 可重复调用：全新库只建表，老库只跑未执行过的迁移。
pub fn init_schema(conn: &mut Connection) -> Result<(), StorageError> {
    let tx = conn.transaction()?;

    // 建表前先探测：这是全新库，还是历史上已经建过表的库？
    let is_fresh: bool = tx.query_row(
        "SELECT COUNT(*) = 0 FROM sqlite_master WHERE type = 'table' AND name = 'accounts'",
        [],
        |row| row.get(0),
    )?;

    tx.execute_batch(SCHEMA_SQL)?;

    // 全新库的 CREATE TABLE 已是最新结构，起点直接算作最新，跳过全部迁移。
    let from = if is_fresh {
        SCHEMA_VERSION
    } else {
        tx.pragma_query_value(None, "user_version", |row| row.get(0))?
    };

    for (version, sql) in MIGRATIONS {
        if *version > from {
            tx.execute_batch(sql)?;
        }
    }

    tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    tx.commit()?;
    Ok(())
}

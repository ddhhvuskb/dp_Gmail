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
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    email        TEXT    NOT NULL UNIQUE,
    display_name TEXT,
    provider_id  TEXT    NOT NULL,
    created_at   INTEGER NOT NULL
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

/// 在**一个事务**内建表建索引。可重复调用。
pub fn init_schema(conn: &mut Connection) -> Result<(), StorageError> {
    let tx = conn.transaction()?;
    tx.execute_batch(SCHEMA_SQL)?;
    tx.commit()?;
    Ok(())
}

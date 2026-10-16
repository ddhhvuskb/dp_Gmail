//! 文件夹读写。

use rusqlite::{params, Connection};

use super::error::StorageError;

/// 取或建文件夹行，返回 `folders.id`。
///
/// 用 `ON CONFLICT DO NOTHING` + 回查，而不是「先 SELECT 再 INSERT」——
/// 后者在并发下会撞唯一约束，且要多一次往返。
pub fn ensure_folder(conn: &Connection, account_id: i64, name: &str) -> Result<i64, StorageError> {
    conn.execute(
        "INSERT INTO folders (account_id, name) VALUES (?1, ?2)
         ON CONFLICT(account_id, name) DO NOTHING",
        params![account_id, name],
    )?;

    conn.query_row(
        "SELECT id FROM folders WHERE account_id = ?1 AND name = ?2",
        params![account_id, name],
        |row| row.get(0),
    )
    .map_err(StorageError::from)
}

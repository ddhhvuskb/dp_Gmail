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

/// 文件夹记录（供侧栏展示）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FolderRecord {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub uid_validity: Option<i64>,
    /// Unix 秒。
    pub last_sync_at: Option<i64>,
    pub message_count: i64,
    pub unread_count: i64,
}

/// 列出文件夹，按账户与名称排序。
///
/// `account_id` 为 `None` 时列出**全部账户**的文件夹；
/// 用 `?1 IS NULL OR …` 一条 SQL 覆盖两种情况，避免维护两份语句。
pub fn list_folders(
    conn: &Connection,
    account_id: Option<i64>,
) -> Result<Vec<FolderRecord>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.account_id, f.name, f.uid_validity, f.last_sync_at,
                (SELECT COUNT(*) FROM messages m WHERE m.folder_id = f.id),
                (SELECT COUNT(*) FROM messages m WHERE m.folder_id = f.id AND m.is_seen = 0)
         FROM folders f
         WHERE ?1 IS NULL OR f.account_id = ?1
         ORDER BY f.account_id, f.name",
    )?;

    let rows = stmt.query_map(params![account_id], |row| {
        Ok(FolderRecord {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            uid_validity: row.get(3)?,
            last_sync_at: row.get(4)?,
            message_count: row.get(5)?,
            unread_count: row.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

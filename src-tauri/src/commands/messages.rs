//! 邮件读取命令。

use crate::state::AppState;
use crate::storage::{self, MessageSummary};

/// 单页最大条数。
///
/// 上限是**参数校验**职责的一部分：前端如果传 `u32::MAX`，
/// SQLite 会试图把整个文件夹读进内存。
pub const MAX_PAGE_SIZE: u32 = 200;

/// 归一化分页参数：`limit` 收敛到 `1..=MAX_PAGE_SIZE`，`offset` 原样透传。
pub fn normalize_paging(limit: u32, offset: u32) -> (u32, u32) {
    (limit.clamp(1, MAX_PAGE_SIZE), offset)
}

/// 按收件时间倒序分页读取某文件夹的邮件摘要。
///
/// 返回的 `MessageSummary` 是 `storage` 定义的**摘要 DTO**，
/// 不是数据库行 —— `DESIGN.md` P0-04 明确禁止命令层返回原始 DB 行。
#[tauri::command]
pub fn get_messages(
    state: tauri::State<'_, AppState>,
    folder_id: i64,
    limit: u32,
    offset: u32,
) -> Result<Vec<MessageSummary>, String> {
    let (limit, offset) = normalize_paging(limit, offset);

    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    storage::list_messages(&conn, folder_id, limit, offset).map_err(|err| err.to_string())
}

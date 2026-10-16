//! 全文检索命令。

use crate::state::AppState;
use crate::storage::{self, MessageSummary};

/// 检索邮件。`limit` 的上限由 `storage::search_messages` 再钳一次。
///
/// 分词细节（trigram + 短查询 LIKE 回退）全在 storage 层，
/// 命令层不关心 —— 否则中文分词的坑会漏到桥接层。
#[tauri::command]
pub fn search_messages(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: u32,
) -> Result<Vec<MessageSummary>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    storage::search_messages(&conn, &query, limit).map_err(|err| err.to_string())
}

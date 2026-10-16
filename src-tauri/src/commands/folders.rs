//! 文件夹读取命令。

use crate::state::AppState;
use crate::storage::{self, FolderRecord};

/// 列出文件夹。`account_id` 为 `null` 时列出全部账户的。
#[tauri::command]
pub fn list_folders(
    state: tauri::State<'_, AppState>,
    account_id: Option<i64>,
) -> Result<Vec<FolderRecord>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    storage::list_folders(&conn, account_id).map_err(|err| err.to_string())
}

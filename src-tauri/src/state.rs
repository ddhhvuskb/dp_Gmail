//! 应用全局状态。
//!
//! 由 `run()` 在 `setup` 阶段注入，命令层用 `tauri::State<'_, AppState>` 取用。

use std::sync::Mutex;

use rusqlite::Connection;

/// 应用状态。目前只有数据库连接。
pub struct AppState {
    /// `rusqlite::Connection` 是 `Send` 但**不是** `Sync`，共享必须加锁。
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self { db: Mutex::new(db) }
    }
}
